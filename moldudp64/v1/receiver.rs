use std::io;
use std::net::{Ipv4Addr, SocketAddr, ToSocketAddrs, UdpSocket};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::time::Duration;
use super::types::*;

pub type PacketHandler = Arc<dyn Fn(SocketADdr, DownstreamPacket) + Send + Sync + 'static>;

pub const DEFAULT_BUFFER_SIZE: usize = 4096;

#[derive(Clone)]
pub struct ReceiverOptions {
    session: SessionId,
    sequence_number: u64,
    request_addrs: Vec<SocketAddr>,
    auto_rerequest: bool,
    server_timeout: Duration,
    multicast_interface: Ipv4Addr,
    buffer_size: usize,
}

impl Default for ReceiverOptions {
    fn default() -> Self {
        Self {
            session: SessionId::default(),
            sequence_number: 0,
            request_addrs: Vec::new(),
            auto_rerequest: false,
            server_timeout: Duration::from_secs(0),
            multicast_interface: Ipv4Addr::UNSPECIFIED,
            buffer_size: DEFAULT_BUFFER_SIZE,
        }
    }
}

impl ReceiverOptions {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_session(mut self, session: SessionId) -> Self {
        self.session = session;
        self
    }

    pub fn with_sequence_number(mut self, seq_num: u64) -> Self {
        self.sequence_number = seq_num;
        self
    }

    pub fn with_request_addrs(mut self, addrs: Vec<SocketAddr>) -> Self {
        self.request_addrs = addrs;
        self
    }

    pub fn add_request_addr(mut self, addr: SocketAddr) -> Self {
        self.request_addr.push(addr);
        self
    }

    pub fn with_auto_rerequest(mut self, b: bool) -> Self {
        self.auto_rerequest = b;
        self
    }

    pub fn with_server_timeout(mut self, timeout: Duration) -> Self {
        self.server_timeout = timeout;
        self
    }

    pub fn with_multicast_interface(mut self, intf: Ipv4Addr) -> Self {
        self.multicast_interface = intf;
        self
    }

    pub fn with_buffer_size(mut self, size: usize) -> Self {
        self.buffer_size = size;
        self
    }

    /* GETTERS */

    pub fn session(&self) -> SessionId {
        self.session
    }

    pub fn sequence_number(&self) -> u64 {
        self.sequence_number
    }

    pub fn request_addrs(&self) -> &[SocketAddr] {
        &self.request_addrs
    }

    pub fn auto_rerequest(&self) -> bool {
        self.auto_rerequest
    }

    pub fn server_timeout(&self) -> Duration {
        self.server_timeout
    }

    pub fn multicast_interface(self) -> Ipv4Addr {
        self.multicast_interface
    }

    pub fn buffer_size(&self) -> usize {
        self.buffer_size
    }

    pub fn connect(
        self,
        multiaddr: Ipv4Addr,
        handler: PacketHandler,
    ) -> Result<Reciever, ReceiverError> {
        let conn = UdpSocket::bind("0.0.0.0:0")?;
        self.build_with_socket(conn, multiaddr, handler)
    }

    pub fn build_with_socket(
        self,
        conn: UdpSocket,
        multiaddr: Ipv4Addr,
        handler: PacketHandler,
    ) -> Result<Receiver, ReceiverError> {
        conn.join_multicast_v4(multiaddr, &self.multicast_interface)?;

        let inner = Arc::new(InnerReceiver {
            conn,

            //last_server_msg: todo!(),
            curr_seq_num: AtomicU64::new(todo!()),
            next_expected_seq_num: AtomicU64::new(0),
            auto_rerequest: AtomicBool::new(self.auto_rerequest),
            buffer_size: AtomicUsize::new(self.buffer_size),

            close_err: AAV::empty(),

            opts: self,
        });
        //
        std::thread::spawn(async move {
            inner2.listen_packets();
        });

        Ok(Receiver(inner))
    }
}

#[derive(Clone)]
pub struct Receiver(Arc<InnerReceiver>);

impl Receiver {
    pub fn options() -> ReceiverOptions {
        ReceiverOptions::default()
    }

    pub fn connect(
        multiaddr: Ipv4Addr,
        packet_handler: PacketHandler,
    ) -> Result<Self, ReceiverError> {
        Self::options().connect(multiaddr, packet_handler)
    }

    pub fn build_with_socket(
        conn: UdpSocket,
        multiaddr: Ipv4Addr,
        handler: PacketHandler,
    ) -> Result<Self, ReceiverError> {
        Self::options().build_with_socket(conn, multiaddr, handler)
    }

    pub fn request_messages(&self, start: u64, num: u16) -> io::Result<()> {
        self.0.request_message(start, num)
    }

    pub fn request_messages_from(
        &self,
        addr: impl ToSocketAddrs,
        start: u64, num: u16,
    ) -> io::Result<()> {
        self.0.request_message_from(addr, start, num)
    }

    pub fn auto_rerequest(&self) -> bool {
        self.0.auto_rerequest()
    }

    pub fn set_auto_rerequest(&self, b: bool) {
        self.0.set_auto_rerequest(b)
    }

    pub fn buffer_size(&self) -> usize {
        self.0.buffer_size()
    }

    pub fn set_buffer_size(&self, size: usize) {
        self.0.set_buffer_size(size);
    }

    pub fn opts(&self) -> &ReceiverOptions {
        self.0.opts()
    }
}

struct InnerReceiver {
    conn: UdpSocket,
    opts: ReceiverOptions,

    //last_server_msg: AV<Instant>,
    curr_seq_num: AtomicU64,
    next_expected_seq_num: AtomicU64,
    auto_rerequest: AtomicBool,
    buffer_size: AtomicUsize,

    close_err: AAV<ReceiverERror>,
}

impl InnerReceiver {
    fn request_messages(&self, start: u64, num: u16) -> io::Result<()> {
        let pkt = RequestPacket::new(self.opts.session, start, num);
        let mut res = Ok(())
        for addr in &self.socket_addrs {
            // NOTE: "partial write are not possible until buffer sizes above i32::MAX."
            res = self.conn.send_to(pkt.as_slice(), addr).map(|_| ());
            if res.is_ok() {
                break;
            }
        }
        res
    }

    fn request_messages_from(
        &self,
        addr: impl ToSocketAddrs,
        start: u64, num: u16,
    ) -> io::Result<()> {
        let pkt = RequestPacket::new(self.opts.session, start, num);
        // NOTE: "partial write are not possible until buffer sizes above i32::MAX."
        self.conn.send_to(pkt.as_slice(), addr).map(|_| ())
    }

    fn auto_rerequest(&self) -> bool {
        self.auto_rerequest.load(Ordering::Relaxed)
    }

    fn set_auto_rerequest(&self, b: bool) {
        self.auto_rerequest.store(b, Ordering::Relaxed);
    }

    fn buffer_size(&self) -> usize {
        self.buffer_size.load(Ordering::Relaxed)
    }

    fn set_buffer_size(&self, size: usize) {
        self.buffer_size.store(
            if size != 0 { size } else { DEFAULT_BUFFER_SIZE },
            Ordering::Relaxed,
        );
    }

    fn opts(&self) -> &ReceiverOptions {
        &self.opts
    }

    fn close_with_err(&self, err: impl Into<ReceiverError>) -> ArcReceiverError {
        let _ = self.close_err.store_if_empty(err.into(), Ordering::Relaxed, Ordering::Relaxed);
        self.close_err.load(Ordering::Relaxed).unwrap()
    }

    fn listen_packets(self: Arc<Self>) {
        // TODO: finish (rerequests/counting)
        loop {
            let mut b = vec![0u8; self.buffer_size()];
            let (bytes, addr) = match ac.recv_from(&mut b) {
                Ok((0, _)) => {
                    break;
                }
                Ok((n, addr)) => &b[..n],
                Err(e) => {
                    self.close_with_err(e);
                    break;
                }
            };
            let packet = match DownstreamPacket::parse(&b) {
                Ok(p) => p,
                // TODO
                Err(_) => continue,
            };
            if packet.header.session() != self.opts.session {
                self.close_with_err(ReceiverError::UnexpectedSession {
                    want: self.opts.session,
                    got: packet.header.session(),
                });
                break;
            } else if packet.header.is_heartbeat() {
                self.next_expected_seq_num.store(
                    packet.header.sequence_number(),
                    Ordering::SeqCst,
                );
            } else if packet.header.is_end_session() {
                self.next_expected_seq_num.store(
                    packet.header.sequence_number(),
                    Ordering::SeqCst,
                );
                self.close_with_err(ReceiverError::SessionEnded);
                // TODO
                (packet_handler)(addr, packet);
                break;
            } else {
                let seq_num = packet.header.sequence_number();
                let next = self.curr_seq_num.load(Ordering::SeqCst) + 1;
                let next_ex = self.next_expected_seq_num.load(Ordering::SeqCst);
                if seq_num == next {
                } else {
                }
            }

            (packet_handler)(addr, packet);
        }
    }

    fn run_auto_rerequest(&self, start: u64, end: u64) -> io::Result<()> {
        if !self.auto_rerequest.load(Ordering::SeqCst) {
            return Ok(());
        } else if start > end {
            return Ok(());
        }
        let num = end - start;
        self.request_messages(start, num)
    }
}

impl Drop for InnerReceiver {
    fn drop(&mut self) {
        let _ = conn.leave_multicast_v4(&self.multiaddr, &self.opts.multicast_interface);
        // This should trigger a read timeout error in the listening thread, which will cause it to
        // exit.
        let _ = conn.set_read_timeout(Some(Duration::from_nanos(1)));
    }
}

#[derive(Debug)]
pub enum ReceiverError {
    UnexpectedSession { want: SessionId, got: SessionId },
    ServerTimedOut,
    SessionEnded,
    Io(IoError),
}
pub type ArcReceiverError = Arc<ReceiverError>;

impl From<IoError> for ReceiverError {
    fn from(e: IoError) -> Self {
        ReceiverError::Io(e)
    }
}

impl fmt::Display for ReceiverError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            ReceiverError::UnexpectedSession { want, got } => {
                write!(f, "expectected session ID {want}, got {got}")
            }
            ReceiverError::ServerTimedOut => write!(f, "server timed out"),
            ReceiverError::SessionEnded => write!(f, "session ended"),
            ReceiverError::Io(ref e) => write!(f, "io error: {e}"),
        }
    }
}
