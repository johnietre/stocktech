use super::types::*;

use jtutils::atomic_value::{Ordering, AAV, NEAV};
use std::error::Error;
use std::fmt;
use std::io::{prelude::*, Error as IoError, ErrorKind as IoErrorKind};
use std::net::{Shutdown, TcpStream, ToSocketAddrs};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, Instant};

pub type ArcClientError = Arc<ClientError>;

#[derive(Debug)]
pub enum ClientError {
    LoggedOut,
    ServerTimedOut,
    LoginRejected(LoginReject),
    UnexpectedPacket(Packet),
    PacketParse(PacketParseError),
    Io(IoError),
}

impl From<IoError> for ClientError {
    fn from(e: IoError) -> Self {
        ClientError::Io(e)
    }
}

impl From<PacketParseError> for ClientError {
    fn from(e: PacketParseError) -> Self {
        match e {
            PacketParseError::Io(e) => ClientError::Io(e),
            e => ClientError::PacketParse(e),
        }
    }
}

impl fmt::Display for ClientError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            ClientError::LoggedOut => write!(f, "logged out"),
            ClientError::ServerTimedOut => write!(f, "server timed out"),
            ClientError::LoginRejected(r) => write!(f, "login rejected: {r}"),
            // TODO: print payload?
            ClientError::UnexpectedPacket(ref p) => write!(
                f,
                "unexpected packet (packet type: {:?}, payload len: {})",
                p.packet_type(),
                p.payload().len()
            ),
            ClientError::PacketParse(ref e) => write!(f, "packet parse error: {e}"),
            ClientError::Io(ref e) => write!(f, "io error: {e}"),
        }
    }
}

impl Error for ClientError {}

pub const DEFAULT_SERVER_TIMEOUT: Duration = Duration::from_secs(15);
pub(crate) const CLIENT_HEARTBEAT: Duration = Duration::from_secs(1);

pub type ClientHandler = Arc<dyn Fn(Packet) + Send + Sync>;

#[derive(Clone)]
pub struct ClientOptions {
    session: SessionId,
    sequence_number: SequenceNumber,
    username: Username,
    password: Password,
    server_timeout: Duration,
    deadline: Option<Instant>,
}

impl Default for ClientOptions {
    fn default() -> Self {
        Self {
            session: SessionId::default(),
            sequence_number: SequenceNumber::default(),
            username: Username::default(),
            password: Password::default(),
            server_timeout: DEFAULT_SERVER_TIMEOUT,
            deadline: None,
        }
    }
}

impl ClientOptions {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_session(mut self, session: SessionId) -> Self {
        self.session = session;
        self
    }

    pub fn with_sequence_number(mut self, seq_num: SequenceNumber) -> Self {
        self.sequence_number = seq_num;
        self
    }

    pub fn with_username(mut self, username: Username) -> Self {
        self.username = username;
        self
    }

    pub fn with_password(mut self, password: Password) -> Self {
        self.password = password;
        self
    }

    pub fn with_server_timeout(mut self, timeout: Duration) -> Self {
        self.server_timeout = timeout;
        self
    }

    pub fn with_deadline(mut self, deadline: Option<Instant>) -> Self {
        self.deadline = deadline;
        self
    }

    pub fn session(&self) -> SessionId {
        self.session
    }

    pub fn sequence_number(&self) -> SequenceNumber {
        self.sequence_number
    }

    pub fn username(&self) -> Username {
        self.username
    }

    pub fn password(&self) -> Password {
        self.password
    }

    pub fn server_timeout(&self) -> Duration {
        self.server_timeout
    }

    pub fn deadline(&self) -> Option<Instant> {
        self.deadline
    }

    pub fn connect<A: ToSocketAddrs>(
        self,
        addr: A,
        handler: Option<ClientHandler>,
    ) -> Result<Client, ClientError> {
        #[inline(always)]
        fn map_deadline(deadline: Option<Instant>) -> Option<Duration> {
            const NANO: Duration = Duration::from_nanos(1);
            // TODO: what to return when after deadline
            deadline.map(|d| d.checked_duration_since(Instant::now()).unwrap_or(NANO))
        }

        #[allow(dead_code)]
        const SOME_ZERO: Option<Duration> = Some(Duration::ZERO);

        let mut stream = if self.deadline.is_some() {
            let mut res = Err(IoError::new(
                IoErrorKind::InvalidInput, "no valid address specified",
            ));
            for addr in addr.to_socket_addrs()? {
                let timeout = map_deadline(self.deadline).unwrap();
                res = TcpStream::connect_timeout(&addr, timeout);
                if res.is_ok() {
                    break;
                }
            }
            res?
        } else {
            TcpStream::connect(addr)?
        };

        let packet = Packet::login_request(
            self.username,
            self.password,
            self.session,
            self.sequence_number,
        );
        if self.deadline.is_some() {
            stream.set_write_timeout(map_deadline(self.deadline))?;
        }
        stream.write_all(&packet)?;

        if self.deadline.is_some() {
            stream.set_read_timeout(map_deadline(self.deadline))?;
        }
        let packet = Packet::read_from(&mut stream)?;
        match packet.packet_type() {
            PacketType::LoginAccepted => (),
            PacketType::LoginReject => match packet.reject_reason() {
                Some(reason) => return Err(ClientError::LoginRejected(reason)),
                None => return Err(ClientError::UnexpectedPacket(packet)),
            },
            _ => return Err(ClientError::UnexpectedPacket(packet)),
        };

        if self.deadline.is_some() {
            stream.set_read_timeout(None)?;
            stream.set_write_timeout(None)?;
        }
        let stream = Arc::new(stream);
        let now = Instant::now();
        let inner = Arc::new(InnerClient {
            read_stream: Mutex::new(Some(Arc::clone(&stream))),
            write_stream: Mutex::new(Some(Arc::clone(&stream))),
            opts: self,
            handler,

            last_server_heartbeat: NEAV::new(now),
            last_client_heartbeat: NEAV::new(now),

            close_err: AAV::empty(),
        });
        if inner.handler.is_some() {
            assert!(
                Arc::clone(&inner).listen_packets_and_heartbeats(),
                "did not start listening to packets",
            );
        } else {
            Arc::clone(&inner).check_heartbeats();
        }
        Ok(Client(inner))
    }
}

#[derive(Clone)]
pub struct Client(Arc<InnerClient>);

impl Client {
    pub fn connect<A: ToSocketAddrs>(
        addr: A,
        username: Username,
        password: Password,
        handler: Option<ClientHandler>,
    ) -> Result<Self, ClientError> {
        ClientOptions::new()
            .with_username(username)
            .with_password(password)
            .connect(addr, handler)
    }

    pub fn options() -> ClientOptions {
        ClientOptions::new()
    }

    pub fn read_packet(&self) -> Option<Result<Packet, ArcClientError>> {
        self.0.read_packet()
    }

    pub fn send_unsequenced(&self, payload: Payload) -> Result<(), ArcClientError> {
        self.0.send_unsequenced(payload)
    }

    pub fn logout(&self) -> Result<(), ArcClientError> {
        self.0.logout()
    }

    pub fn close_err(&self) -> Option<ArcClientError> {
        self.0.close_err()
    }

    pub fn is_closed(&self) -> bool {
        self.0.is_closed()
    }

    pub fn opts(&self) -> &ClientOptions {
        self.0.opts()
    }

    pub fn handler(&self) -> &Option<ClientHandler> {
        self.0.handler()
    }

    pub fn last_client_heartbeat(&self) -> Instant {
        self.0.last_client_heartbeat()
    }

    pub fn last_server_heartbeat(&self) -> Instant {
        self.0.last_server_heartbeat()
    }

    pub fn set_last_server_heartbeat(&self, t: Instant) {
        self.0.set_last_server_heartbeat(t);
    }
}

struct InnerClient {
    read_stream: Mutex<Option<Arc<TcpStream>>>,
    write_stream: Mutex<Option<Arc<TcpStream>>>,
    opts: ClientOptions,
    handler: Option<ClientHandler>,

    last_client_heartbeat: NEAV<Instant>,
    last_server_heartbeat: NEAV<Instant>,

    close_err: AAV<ClientError>,
}

impl InnerClient {
    fn read_packet(&self) -> Option<Result<Packet, ArcClientError>> {
        let mut read_half_opt = self.read_stream.lock().unwrap();
        let read_half = read_half_opt.as_ref()?;
        // TODO: close?
        match Packet::read_from(&mut &**read_half) {
            Ok(packet) => Some(Ok(packet)),
            Err(e) => {
                read_half_opt.take();

                // TODO: is this be best way?
                if let Some(w) = self.write_stream.lock().unwrap().take() {
                    let _ = w.shutdown(Shutdown::Both);
                }

                Some(Err(self.close_with_err(ClientError::PacketParse(e))))
            }
        }
    }

    fn send_unsequenced(&self, payload: Payload) -> Result<(), ArcClientError> {
        self.send_packet(Packet::unsequenced_data(payload))
    }

    fn logout(&self) -> Result<(), ArcClientError> {
        self.send_packet(Packet::logout_request())?;
        let Some(write_half) = self.write_stream.lock().unwrap().take() else {
            // TODO: don't spin?
            loop {
                if let Some(e) = self.close_err.load(Ordering::Relaxed) {
                    return Err(e);
                }
            }
        };
        let err = match write_half.shutdown(Shutdown::Both) {
            Ok(_) => ClientError::LoggedOut,
            Err(e) => ClientError::Io(e),
        };
        self.close_with_err(err);
        Ok(())
    }

    fn close_err(&self) -> Option<Arc<ClientError>> {
        self.close_err.load(Ordering::Relaxed)
    }

    fn is_closed(&self) -> bool {
        self.close_err.is_empty(Ordering::Relaxed)
    }

    fn opts(&self) -> &ClientOptions {
        &self.opts
    }

    fn handler(&self) -> &Option<ClientHandler> {
        &self.handler
    }

    fn last_client_heartbeat(&self) -> Instant {
        self.last_client_heartbeat.load_copied(Ordering::Relaxed)
    }

    fn last_server_heartbeat(&self) -> Instant {
        self.last_server_heartbeat.load_copied(Ordering::Relaxed)
    }

    fn set_last_server_heartbeat(&self, t: Instant) {
        self.last_server_heartbeat.store(t, Ordering::Relaxed);
    }

    fn send_packet(&self, packet: Packet) -> Result<(), ArcClientError> {
        if let Some(err) = self.close_err() {
            return Err(err);
        }
        let mut write_half_opt = self.write_stream.lock().unwrap();
        let Some(write_half) = write_half_opt.as_mut() else {
            // TODO: don't spin?
            loop {
                if let Some(e) = self.close_err.load(Ordering::Relaxed) {
                    return Err(e);
                }
            }
        };
        // TODO: close?
        if let Err(e) = (&**write_half).write(&packet) {
            return Err(self.close_with_err(e));
        }
        self.last_client_heartbeat
            .store(Instant::now(), Ordering::Relaxed);
        Ok(())
    }

    fn listen_packets_and_heartbeats(self: Arc<Self>) -> bool {
        use jtutils::presult::{pio::*, prelude::*};

        let Some(read_stream) = self.read_stream.lock().unwrap().take() else {
            return false;
        };
        let Some(handler) = self.handler.clone() else {
            return false;
        };
        let server_timeout = self.opts.server_timeout();
        thread::spawn(move || {
            let read_half = read_stream;
            let mut read_stream = Adapter::new(&*read_half);

            let mut buf = vec![0u8; 2];
            let mut buf_pos = 0;
            let mut packet_len = 0;
            loop {
                if self.is_closed() {
                    break;
                }
                if self.last_client_heartbeat().elapsed() >= CLIENT_HEARTBEAT {
                    self.heartbeat();
                }
                if self.last_server_heartbeat().elapsed() > server_timeout {
                    self.close_with_err(ClientError::ServerTimedOut);
                    break;
                }
                if self.is_closed() {
                    break;
                }

                // Read the packet length if we haven't yet
                if buf_pos < 2 {
                    // Set timeout
                    let dur = (self.last_client_heartbeat() + CLIENT_HEARTBEAT)
                        .checked_duration_since(Instant::now());
                    if dur.unwrap_or(Duration::ZERO) == Duration::ZERO {
                        // Need to check heartbeats
                        continue;
                    }
                    if let Err(e) = read_stream.set_read_timeout(dur) {
                        self.close_with_err(ClientError::Io(e));
                        break;
                    }

                    // Read
                    match read_stream.pread_exact(&mut buf[buf_pos..2]) {
                        POk(_) => buf_pos = 2,
                        PPartial(n, e) => {
                            buf_pos += n;
                            if !is_timeout(&e) {
                                self.close_with_err(ClientError::Io(e));
                                break;
                            }
                            // Need to check heartbeats
                            continue;
                        }
                        PErr(e) => {
                            if !is_timeout(&e) {
                                self.close_with_err(ClientError::Io(e));
                                break;
                            }
                            continue;
                        }
                    }
                    packet_len = (buf[0] as usize) | ((buf[1] as usize) << 8);
                    if buf.len() < 2 + packet_len {
                        buf.resize(2 + packet_len, 0);
                    }
                    // TODO: possibly shrink buf
                }

                // Read rest of packet (packet type and payload)

                // Set timeout
                let dur = (self.last_client_heartbeat() + CLIENT_HEARTBEAT)
                    .checked_duration_since(Instant::now());
                if dur.unwrap_or(Duration::ZERO) == Duration::ZERO {
                    // Need to check heartbeats
                    continue;
                }
                if let Err(e) = read_stream.set_read_timeout(dur) {
                    self.close_with_err(ClientError::Io(e));
                    break;
                }

                // Read
                match read_stream.pread_exact(&mut buf[buf_pos..2 + packet_len]) {
                    POk(n) => buf_pos += n,
                    PPartial(n, e) => {
                        buf_pos += n;
                        if !is_timeout(&e) {
                            self.close_with_err(ClientError::Io(e));
                            break;
                        }
                        // Need to check heartbeats
                        continue;
                    }
                    PErr(e) => {
                        if !is_timeout(&e) {
                            self.close_with_err(ClientError::Io(e));
                            break;
                        }
                        continue;
                    }
                }
                // Full packet read (shouldn't need to check but will do anyway)
                if buf_pos < 2 + packet_len {
                    continue;
                }

                let packet = match Packet::parse(&buf) {
                    Ok(packet) => packet,
                    Err(e) => {
                        self.close_with_err(e);
                        if let Some(write_stream) = self.write_stream.lock().unwrap().take() {
                            let _ = write_stream.shutdown(Shutdown::Both);
                        }
                        break;
                    }
                };
                // TODO: how best to call
                (handler)(packet);
            }
        });
        true
    }

    fn check_heartbeats(self: Arc<Self>) {
        let server_timeout = self.opts.server_timeout();
        thread::spawn(move || loop {
            if self.is_closed() {
                break;
            }
            let lch = self.last_client_heartbeat();
            sleep_until(lch + CLIENT_HEARTBEAT);
            if self.is_closed() {
                break;
            }
            if self.last_server_heartbeat().elapsed() > server_timeout {
                self.close_with_err(ClientError::ServerTimedOut);
                break;
            }
            if self.last_client_heartbeat() == lch {
                self.heartbeat();
            }
        });
    }

    fn heartbeat(&self) {
        let _ = self.send_packet(Packet::client_heartbeat());
    }

    fn close_with_err(&self, err: impl Into<ClientError>) -> ArcClientError {
        let _ = self.close_err.store_if_empty(err.into(), Ordering::Relaxed, Ordering::Relaxed);
        self.close_err.load(Ordering::Relaxed).unwrap()
    }
}

fn sleep_until(deadline: Instant) {
    let now = Instant::now();
    if let Some(delay) = deadline.checked_duration_since(now) {
        thread::sleep(delay);
    }
}

fn is_timeout(e: &IoError) -> bool {
    // TODO
    match e.kind() {
        IoErrorKind::WouldBlock | IoErrorKind::TimedOut => true,
        _ => false,
    }
}
