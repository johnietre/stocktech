use std::process::Command;

struct App {
    cmd: Command,
    client_packets: Vec<Packet>,
    server_packets: Vec<Packet>,
}

impl App {
    fn new(prog: AsRef<OsStr>) -> Self {
        Self {
            cmd,
            client_packets: Vec::new(),
            server_packets: Vec::new(),
        }
    }

    fn run() {
    }
}
