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

/*
fn test(addr: impl ToSocketAddrs) {
    let conn = UdpSocket::bind("0.0.0.0:0").expect("bad bind");
    conn.join_multicast_v4(&Ipv4Addr::UNSPECIFIED).expect("bad join");
}
*/
