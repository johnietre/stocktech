pub use crate::v4::client::{ArcClientError, ClientError, ClientHandler, DEFAULT_SERVER_TIMEOUT};
use crate::v4::client::CLIENT_HEARTBEAT;
use crate::v4::types::*;

use jtutils::atomic_value::{Ordering, AAV, NEAV};
use std::marker::Unpin;
use std::sync::Arc;
use std::time::Duration;
use tokio::io::{AsyncRead, AsyncReadExt, AsyncWriteExt};
use tokio::net::tcp::{OwnedReadHalf, OwnedWriteHalf};
use tokio::net::{TcpStream, ToSocketAddrs};
use tokio::sync::Mutex;
use tokio::time::{Instant, sleep_until};

#[derive(Clone)]
pub struct ClientOptions {
    session: SessionId,
    sequence_number: SequenceNumber,
    username: Username,
    password: Password,
    server_timeout: Duration,
    //deadline: Option<Instant>,
}

impl Default for ClientOptions {
    fn default() -> Self {
        Self {
            session: SessionId::default(),
            sequence_number: SequenceNumber::default(),
            username: Username::default(),
            password: Password::default(),
            server_timeout: DEFAULT_SERVER_TIMEOUT,
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

    /*
    pub fn with_deadline(mut self, deadline: Option<Instant>) -> Self {
        self.deadline = deadline;
        self
    }
    */

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

    pub async fn connect<A: ToSocketAddrs>(
        self,
        addr: A,
        handler: Option<ClientHandler>,
    ) -> Result<Client, ClientError> {
        let mut stream = TcpStream::connect(addr).await?;

        let packet = Packet::login_request(
            self.username,
            self.password,
            self.session,
            self.sequence_number,
        );
        stream.write_all(&packet).await?;

        let packet = read_packet_from(&mut stream).await?;
        match packet.packet_type() {
            PacketType::LoginAccepted => (),
            // TODO: what if no reject reason
            PacketType::LoginReject => match packet.reject_reason() {
                Some(reason) => return Err(ClientError::LoginRejected(reason)),
                None => return Err(ClientError::UnexpectedPacket(packet)),
            },
            _ => return Err(ClientError::UnexpectedPacket(packet)),
        };

        let (read, write) = stream.into_split();
        let now = Instant::now();
        let inner = Arc::new(InnerClient {
            read_half: Mutex::new(Some(read)),
            write_half: Mutex::new(Some(write)),
            opts: self,
            handler,

            last_server_heartbeat: NEAV::new(now),
            last_client_heartbeat: NEAV::new(now),

            session_number: (),

            close_err: AAV::empty(),
        });
        if inner.handler.is_some() {
            assert!(
                Arc::clone(&inner).listen_packets().await,
                "did not start listening to packets",
            );
        }
        Arc::clone(&inner).check_heartbeats().await;
        Ok(Client(inner))
    }
}

#[derive(Clone)]
pub struct Client(Arc<InnerClient>);

impl Client {
    pub async fn connect<A: ToSocketAddrs>(
        addr: A,
        username: Username,
        password: Password,
        handler: Option<ClientHandler>,
    ) -> Result<Self, ClientError> {
        ClientOptions::new()
            .with_username(username)
            .with_password(password)
            .connect(addr, handler)
            .await
    }

    pub fn options() -> ClientOptions {
        ClientOptions::new()
    }

    pub async fn read_packet(&self) -> Option<Result<Packet, ArcClientError>> {
        self.0.read_packet().await
    }

    pub async fn send_unsequenced(&self, payload: Payload) -> Result<(), ArcClientError> {
        self.0.send_unsequenced(payload).await
    }

    pub async fn logout(&self) -> Result<(), ArcClientError> {
        self.0.logout().await
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
    read_half: Mutex<Option<OwnedReadHalf>>,
    write_half: Mutex<Option<OwnedWriteHalf>>,
    opts: ClientOptions,
    handler: Option<ClientHandler>,

    last_client_heartbeat: NEAV<Instant>,
    last_server_heartbeat: NEAV<Instant>,

    #[allow(dead_code)]
    session_number: (),

    close_err: AAV<ClientError>,
}

impl InnerClient {
    async fn read_packet(&self) -> Option<Result<Packet, ArcClientError>> {
        let mut read_half_opt = self.read_half.lock().await;
        let read_half = read_half_opt.as_mut()?;
        // TODO: close?
        match read_packet_from(read_half).await {
            Ok(packet) => Some(Ok(packet)),
            Err(e) => {
                read_half_opt.take();

                // TODO: is this the best way?
                self.write_half.lock().await.take();

                Some(Err(self.close_with_err(e)))
            }
        }
    }

    async fn send_unsequenced(&self, payload: Payload) -> Result<(), ArcClientError> {
        self.send_packet(Packet::unsequenced_data(payload)).await
    }

    async fn logout(&self) -> Result<(), ArcClientError> {
        self.send_packet(Packet::logout_request()).await?;
        let Some(mut write_half) = self.write_half.lock().await.take() else {
            // TODO: don't spin?
            loop {
                if let Some(e) = self.close_err.load(Ordering::Relaxed) {
                    return Err(e);
                }
            }
        };
        let err = match write_half.shutdown().await {
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

    async fn send_packet(&self, packet: Packet) -> Result<(), ArcClientError> {
        if let Some(err) = self.close_err() {
            return Err(err);
        }
        let mut write_half_opt = self.write_half.lock().await;
        let Some(write_half) = write_half_opt.as_mut() else {
            // TODO: don't spin?
            loop {
                if let Some(e) = self.close_err.load(Ordering::Relaxed) {
                    return Err(e);
                }
            }
        };
        // TODO: close?
        if let Err(e) = write_half.write(&packet).await {
            return Err(self.close_with_err(e));
        }
        self.last_client_heartbeat
            .store(Instant::now(), Ordering::Relaxed);
        Ok(())
    }

    async fn listen_packets(self: Arc<Self>) -> bool {
        let Some(mut read_half) = self.read_half.lock().await.take() else {
            return false;
        };
        let Some(handler) = self.handler.clone() else {
            return false;
        };
        tokio::spawn(async move {
            loop {
                let packet = match read_packet_from(&mut read_half).await {
                    Ok(packet) => packet,
                    Err(e) => {
                        self.close_with_err(e);
                        self.write_half.lock().await.take();
                        break;
                    }
                };
                self.set_last_server_heartbeat(Instant::now());
                if packet.packet_type() == PacketType::ServerHeartbeat {
                    continue;
                }
                let h = handler.clone();
                tokio::spawn(async move {
                    (h)(packet);
                });
            }
        });
        true
    }

    async fn check_heartbeats(self: Arc<Self>) {
        let server_timeout = self.opts.server_timeout();
        tokio::spawn(async move {
            loop {
                if self.is_closed() {
                    break;
                }
                let lch = self.last_client_heartbeat();
                sleep_until(lch + CLIENT_HEARTBEAT).await;
                if self.is_closed() {
                    break;
                }
                if self.last_server_heartbeat().elapsed() > server_timeout {
                    self.close_with_err(ClientError::ServerTimedOut);
                    break;
                }
                if self.last_client_heartbeat() == lch {
                    self.heartbeat().await;
                }
            }
        });
    }

    async fn heartbeat(&self) {
        let _ = self.send_packet(Packet::client_heartbeat()).await;
    }

    fn close_with_err(&self, err: impl Into<ClientError>) -> ArcClientError {
        let _ = self.close_err.store_if_empty(err.into(), Ordering::Relaxed, Ordering::Relaxed);
        self.close_err.load(Ordering::Relaxed).unwrap()
    }
}

async fn read_packet_from<R: AsyncRead + Unpin>(r: &mut R) -> Result<Packet, PacketParseError> {
    let mut buf = [0u8; 3];
    r.read_exact(&mut buf).await?;
    let payload_len = match (buf[0] as usize) | ((buf[1] as usize) << 8) {
        0 => return Err(PacketParseError::MismatchLen { want: 1, got: 0 }),
        pl => pl - 1,
    };
    let packet_type = match PacketType::from_u8(buf[2]) {
        Ok(pt) => pt,
        Err(b) => return Err(PacketParseError::InvalidPacketType(b)),
    };
    let want_len = packet_type.payload_len().unwrap_or(payload_len);
    if payload_len != want_len {
        return Err(PacketParseError::MismatchLen {
            want: want_len,
            got: payload_len,
        });
    }
    let mut payload = vec![0u8; want_len];
    r.read_exact(&mut payload).await?;
    match Payload::new(payload) {
        Ok(payload) => Ok(Packet::new(packet_type, payload)),
        Err(bytes) => Err(PacketParseError::BadPayload(bytes)),
    }
}
