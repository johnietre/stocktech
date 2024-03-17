use std::net::{IpAddr, Ipv4Addr, SocketAddr, ToSocketAddrs, UdpSocket};

fn main() {
    let conn = UdpSocket::bind("0.0.0.0:10000").unwrap();
    let addr = "224.0.0.0:10000".to_socket_addrs().unwrap().next().unwrap();
    let IpAddr::V4(addr) = addr.ip() else {
        panic!("no");
    };
    conn.join_multicast_v4(&addr, &Ipv4Addr::UNSPECIFIED).unwrap();
    println!("{:?}", conn.local_addr());
    println!("{:?}", conn.peer_addr());
    loop {
        let mut b = vec![0u8; 1000];
        match conn.recv_from(&mut b) {
            Ok((n, addr)) => println!("{addr:?} ({n} bytes) => {}", b[0]),
            Err(e) => println!("{e}"),
        }
    }

    /*
    let conn = socket2::Socket::new(socket2::Domain::IPV4, socket2::Type::DGRAM, Some(socket2::Protocol::UDP)).unwrap();
    conn.set_reuse_address(true).unwrap();
    let Some(SocketAddr::V4(addr)) = "224.0.0.0:10000".to_socket_addrs().unwrap().find(|a| a.is_ipv4()) else {
        // TODO
        panic!("no good");
    };
    conn.set_reuse_address(true).unwrap();
    conn.set_reuse_port(true).unwrap();
    conn.join_multicast_v4(addr.ip(), &Ipv4Addr::UNSPECIFIED).unwrap();
    conn.bind(&"0.0.0.0:0".to_socket_addrs().unwrap().next().unwrap().into()).unwrap();

    loop {
        let mut b = vec![std::mem::MaybeUninit::new(0u8); 1000];
        match conn.recv_from(&mut b) {
            Ok((n, addr)) => println!("{addr:?} ({n} bytes) => {}", unsafe { b[0].assume_init() }),
            Err(e) => println!("{e}"),
        }
    }
    */
}
/*
use std::net::Ipv4Addr;
use std::net::UdpSocket;
use std::str::FromStr;
use std::thread;
use std::time::Duration;

fn main() {
    // Host of group A.
    let proposers_host = "239.0.0.1";

    // Host of group B.
    let acceptors_host = "239.0.0.1";

    let acceptors_address = "239.0.0.1:7000";

    let join_handle1: thread::JoinHandle<_> = thread::spawn(move || {
        println!("{}", "Spawning 1st thread");

        let socket = UdpSocket::bind("0.0.0.0:0").expect("Couldn't bind proposer UDP socket");

        // Does NOT work even if I uncomment the following.
        // socket
        //     .set_multicast_loop_v4(true)
        //     .expect("Could not enable multicast loop, to send packets back to the local socket");

        // This socket joined this group. Let's call this group A.
        socket
            .join_multicast_v4(
                &Ipv4Addr::from_str(proposers_host).unwrap(),
                &Ipv4Addr::UNSPECIFIED,
            ).expect("Could not join multicast group A");

        for i in 1..10 {
            // Send message to the acceptors that joined the multicast group B.
            socket
                .send_to(&[i], acceptors_address)
                .expect("couldn't send data");
            println!("Sent message: {:?}\n---\n", i);

            thread::sleep(Duration::from_millis(1));
        }
    });

    let join_handle2: thread::JoinHandle<_> = thread::spawn(move || {
        println!("{}", "Spawning 2nd thread");

        let socket = UdpSocket::bind("0.0.0.0:0").expect("Could not bind acceptor 1 UDP socket");

        // Joining group B.
        socket
            .join_multicast_v4(
                &Ipv4Addr::from_str(acceptors_host).unwrap(),
                &Ipv4Addr::UNSPECIFIED,
            ).expect("Could not join multicast group B");

        let mut buf = [0; 10];
        let mut c = 0;
        loop {
            let (number_of_bytes, src_addr) =
                socket.recv_from(&mut buf).expect("Didn't receive data");

            let filled_buf = &mut buf[..number_of_bytes];

            println!("I am the 2nd socket");
            println!("Message received from address = {:?}", src_addr);
            println!("Contents of the message = {:?}\n---\n", filled_buf);

            thread::sleep(Duration::from_millis(1));

            c += 1;
            if c == 5 {
                break;
            }
        }
    });

    let join_handle3: thread::JoinHandle<_> = thread::spawn(move || {
        println!("{}", "Spawning 3rd thread");

        let socket = UdpSocket::bind("0.0.0.0:0").expect("Could not bind acceptor 2 UDP socket");

        socket
            .join_multicast_v4(
                &Ipv4Addr::from_str(acceptors_host).unwrap(),
                &Ipv4Addr::UNSPECIFIED,
            ).expect("Could not join multicast group B");

        let mut buf = [0; 10];
        loop {
            let (number_of_bytes, src_addr) =
                socket.recv_from(&mut buf).expect("Didn't receive data");

            let filled_buf = &mut buf[..number_of_bytes];

            println!("I am the 3rd socket");
            println!("Message received from address = {:?}", src_addr);
            println!("Contents of the message = {:?}\n---\n", filled_buf);

            thread::sleep(Duration::from_millis(1));
        }
    });

    println!("{}", "At the end");

    join_handle1.join().unwrap();
    join_handle2.join().unwrap();
    join_handle3.join().unwrap();
}
*/
