// TODO: what to on non-full writes/sends?
#[derive(Clone)]
pub struct TransmitterOptions {
    session: SessionId,
    sequence_number: u64,
}

impl TransmitterOptions {
    pub fn new() -> Self {
        Self::default(),
    }

    pub fn with_session(mut self, session: SessionId) -> Self {
        self.session = session;
        self
    }

    pub fn with_sequence_number(mut self, seq_num: u64) -> Self {
        self.sequence_number = seq_num;
        self
    }

    pub async fn connect(self, addr: impl ToSocketAddrs) -> Result<Transmitter, TransmitterError> {
        let conn = UdpSocket::bind("0.0.0.0:0").await?
        conn.connect(addr).await?;
        Ok(self.build_with_socket(conn))
    }

    pub fn build_with_socket(self, conn: UdpSocket) -> Transmitter {
        Transmitter {
            conn,
            session: self.session,
            next_seq_num: AtomicU64::new(self.sequence_number),
        }
    }
}

pub struct Transmitter {
    conn: UdpSocket,
    session: SessionId,
    next_seq_num: AtomicU64,
}

impl Transmitter {
    pub async fn new(addr: impl ToSocketAddrs, session: SessionId) -> io::Result<Self> {
        let conn = UdpSocket::bind("0.0.0.0:0").await?
        conn.connect(addr).await?;
        Self::from_socket(conn, session)
    }

    pub async fn from_socket(conn: UdpSocket, session: SessionId) -> Self {
        Self {
            conn,
            session,
            next_seq_num
        }
    }

    pub async fn send(&self, packet: DownstreamPacket) -> Result<(), TransmitterError> {
        self.conn.send(packet.as_slice()).map(|_| ())
    }

    pub async fn send_message_blocks(
        &self,
        blocks: Vec<MessageBlocks>,
    ) -> Result<(), TransmitterError> {
        self.conn.send(packet.as_slice()).map(|_| ())
    }

    pub fn send_heartbeat(&self) -> Result<(), TransmitterError> {
    }

    pub fn send_end_session(&self) -> Result<(), TransmitterError> {
    }
}
