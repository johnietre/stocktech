use std::convert::TryInto;
use std::io;
use std::net::{Ipv4Addr, SocketAddr, ToSocketAddrs, UdpSocket};
use std::sync::Arc;

pub struct Transmitter {
    conn: UdpSocket,
    session: [u8; 10],
    next_seq_num: AtomicU64,
}

impl Transmitter {
    pub fn new(addr: impl ToSocketAddrs, session: [u8; 10]) -> io::Result<Self> {
        let conn = UdpSocket::bind("0.0.0.0:0")?;
        conn.connect(addr)?;
        Ok(Self { conn, session })
    }

    pub fn send(&self, packet: DownstreamPacket) -> io::Result<()> {
        self.conn.send(&packet.serialize()).map(|_| ())
    }

    pub fn send_message_blocks(&self, blocks: Vec<MessageBlocks>) -> io::Result<()> {
    }

    pub fn send_heartbeat(&self) -> io::Result<()> {
        // TODO
        self.send(&DownstreamPacket::heartbeat(self.session)).map(|_| ())
    }

    pub fn send_end_session(&self) -> io::Result<()> {
        self.conn.send(&DownstreamPacket::end_session().serialize()).map(|_| ())
    }
}

pub struct DownstreamPacket {
    pub header: Header,
    pub message_blocks: Vec<MessageBlock>,
}

impl DownstreamPacket {
    pub fn new(message_blocks: Vec<MessageBlock>) -> Result<Self, ()> {
        let l = message_blocks.len();
        if l > 0xFFFF {
            // TODO
        } else if l == 0xFFFF {
            // TODO
        }
        // TODO
        Ok(Self {
            header: Header {
                session: [0u8; 10],
                sequence_number: 0,
                message_count: l as u16,
            },
            message_blocks,
        })
    }

    pub fn heartbeat() -> Self {
        Self {
            header: Header::heartbeat(),
            message_blocks: Vec::new(),
        }
    }

    pub fn end_session() -> Self {
        Self {
            header: Header::end_session(),
            message_blocks: Vec::new(),
        }
    }

    pub fn parse(mut b: &[u8]) -> Result<Self, ()> {
        if b.len() < 20 {
            return Err(());
        }
        let header = Header::parse(b)?;
        if header.is_heartbeat() || header.is_end_session() {
            return Ok(Self {
                header,
                message_blocks: Vec::new(),
            });
        }
        b = &b[20..];
        let mut message_blocks = Vec::with_capacity(header.message_count as usize);
        for _ in 0..header.message_count {
            if b.len() < 2 {
                return Err(());
            }
            let ml = u16::from_be_bytes(b[..2].try_into().unwrap()) as usize;
            b = &b[2..];
            if b.len() < ml {
                return Err(());
            }
            message_blocks.push(MessageBlock {
                message_len: ml as u16,
                message_data: b[..ml].to_vec(),
            });
            b = &b[ml..];
        }
        Ok(Self { header, message_blocks })
    }

    pub fn serialize(&self) -> Vec<u8> {
        let mut res = self.header.serialize();
        for block in &self.message_blocks {
            res.extend_from_slice(&block.serialize());
        }
        res
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub struct Header {
    // TODO
    /// The sessino to which the packet belongs.
    pub session: [u8; 10],
    /// The sequence number of the first message of the packet.
    pub sequence_number: u64,
    /// The count of messages contained in this packet.
    pub message_count: u16,
}

impl Header {
    pub fn heartbeat() -> Self {
        // TODO
        Self {
            session: [0u8; 10],
            sequence_number: 0,
            message_count: 0,
        }
    }

    pub fn end_session() -> Self {
        // TODO
        Self {
            session: [0u8; 10],
            sequence_number: 0,
            message_count: 0xFFFF,
        }
    }

    pub fn parse(b: &[u8]) -> Result<Self, ()> {
        if b.len() != 20 {
            return Err(());
        }
        Ok(Self {
            session: (&b[..10]).try_into().unwrap(),
            sequence_number: u64::from_be_bytes((&b[10..18]).try_into().unwrap()),
            message_count: u16::from_be_bytes((&b[18..]).try_into().unwrap()),
        })
    }

    pub fn serialize(&self) -> Vec<u8> {
        let mut res = Vec::with_capacity(20);
        res.extend_from_slice(&self.session);
        res.extend_from_slice(&self.sequence_number.to_be_bytes());
        res.extend_from_slice(&self.message_count.to_be_bytes());
        res
    }

    pub fn is_heartbeat(&self) -> bool {
        self.message_count == 0
    }

    pub fn is_end_session(&self) -> bool {
        self.message_count == 0xFFFF
    }
}

pub struct MessageBlock {
    /// The length in bytes of the message contained in this MessageBlock.
    /// Does not include the 2 bytes of the message length field (this field).
    pub message_len: u16,
    /// The message data.
    pub message_data: Vec<u8>,
}

impl MessageBlock {
    pub fn new(data: Vec<u8>) -> Result<Self, ()> {
        let l = data.len();
        if l > 0xFFFF {
            return Err(());
        }
        Ok(Self {
            message_len: l as u16,
            message_data: data,
        })
    }

    pub fn serialize(&self) -> Vec<u8> {
        let mut res = Vec::with_capacity(2 + self.message_len as usize);
        res.extend_from_slice(&self.message_len.to_be_bytes());
        res.extend_from_slice(&self.message_data);
        res
    }
}

pub type PacketHandler = Box<dyn Fn(DownstreamPacket) + Send + Sync + 'static>;

pub struct Receiver {
    conn: Arc<UdpSocket>,
    // TODO
    request_addrs: Vec<SocketAddr>,
}

impl Receiver {
    pub fn connect(
        addr: impl ToSocketAddrs,
        request_addrs: Vec<SocketAddr>,
        packet_handler: PacketHandler,
    ) -> io::Result<Self> {
        Self::connect_with_bind("0.0.0.0:0", addr, request_addrs, packet_handler)
    }

    pub fn connect_with_bind(
        bind_addr: impl ToSocketAddrs,
        addr: impl ToSocketAddrs,
        request_addrs: Vec<SocketAddr>,
        packet_handler: PacketHandler,
    ) -> io::Result<Self> {
        /*
        let conn = Arc::new(UdpSocket::bind(bind_addr)?);
        let Some(SocketAddr::V4(addr)) = addr.to_socket_addrs()?.find(|a| a.is_ipv4()) else {
            // TODO
            return Err(io::Error::other("expected Ipv4Addr"));
        };
        conn.join_multicast_v4(addr.ip(), &Ipv4Addr::UNSPECIFIED)?;
        */

        let sock = socket2::Socket::new(socket2::Domain::IPV4, socket2::Type::DGRAM, Some(socket2::Protocol::UDP))?;
        sock.set_reuse_address(true)?;
        let Some(SocketAddr::V4(addr)) = addr.to_socket_addrs()?.find(|a| a.is_ipv4()) else {
            // TODO
            return Err(io::Error::other("expected Ipv4Addr"));
        };
        sock.join_multicast_v4(addr.ip(), &Ipv4Addr::UNSPECIFIED)?;
        sock.bind(&addr.into())?;
        //sock.bind(&bind_addr.to_socket_addrs()?.next().unwrap().into())?;

        //let conn: Arc<UdpSocket> = Arc::new(sock.into());
        let conn: UdpSocket = sock.into();

        loop {
            let mut b = vec![0u8; 4096];
            let bytes = match conn.recv_from(&mut b) {
                Ok((0, _)) => {
                    break;
                }
                Ok((n, _)) => &b[..n],
                Err(e) => {
                    // TODO
                    eprintln!("{e}");
                    break;
                }
            };
            let packet = match DownstreamPacket::parse(&b) {
                Ok(p) => p,
                // TODO
                Err(_) => break,
            };
            if packet.header.is_end_session() {
                break;
            }
            packet_handler(packet);
        }

        let conn = Arc::new(conn);
        let ac = Arc::clone(&conn);
        std::thread::spawn(move || {
            // TODO: Msg len
            loop {
                let mut b = vec![0u8; 4096];
                let bytes = match ac.recv_from(&mut b) {
                    Ok((0, _)) => {
                        break;
                    }
                    Ok((n, _)) => &b[..n],
                    Err(e) => {
                        // TODO
                        eprintln!("{e}");
                        break;
                    }
                };
                let packet = match DownstreamPacket::parse(&b) {
                    Ok(p) => p,
                    // TODO
                    Err(_) => break,
                };
                if packet.header.is_end_session() {
                    break;
                }
                packet_handler(packet);
            }
        });
        Ok(Self { conn, request_addrs })
    }
}

pub struct RequestPacket {
    pub session: [u8; 10],
    pub sequence_number: u64,
    pub requested_message_count: u16,
}

/*
fn test(addr: impl ToSocketAddrs) {
    let conn = UdpSocket::bind("0.0.0.0:0").expect("bad bind");
    conn.join_multicast_v4(&Ipv4Addr::UNSPECIFIED).expect("bad join");
}
*/
