#[path = "../packet.rs"]
mod packet;
use packet::*;

const ADDR: &str = "224.0.0.0:10000";

fn main() {
    if std::env::args().skip(1).next().map(|s| s != "no").unwrap_or(true) {
        std::thread::spawn(move || run_transmitter());
    }

    std::thread::spawn(move || run_receiver(1));
    std::thread::spawn(move || run_receiver(2));

    std::thread::sleep(std::time::Duration::from_secs(15));
}

fn run_transmitter() {
    let tm = Transmitter::new(ADDR).unwrap();

    for i in 1..=10 {
        std::thread::sleep(std::time::Duration::from_secs(1));
        let mut blocks = Vec::with_capacity(i + 1);
        for j in 1..=blocks.capacity() {
            blocks.push(MessageBlock::new(format!("{i}.{j}").into()).unwrap());
        }
        tm.send(DownstreamPacket::new(blocks).unwrap()).unwrap();
    }

    std::thread::sleep(std::time::Duration::from_secs(1));
    tm.send(DownstreamPacket::end_session()).unwrap();
    std::thread::sleep(std::time::Duration::from_secs(1));
}

fn run_receiver(num: i32) {
    let packet_handler = Box::new(move |packet: DownstreamPacket| {
        println!("{num} received packet");
        for block in packet.message_blocks {
            println!("{num} received message: {}", String::from_utf8_lossy(&block.message_data));
        }
    });
    let rcvr = match Receiver::connect(ADDR, Vec::new(), packet_handler) {
        Ok(r) => r,
        Err(e) => panic!("error creating receiver {}: {}", num, e),
    };
    std::thread::sleep(std::time::Duration::from_secs(15));
}
