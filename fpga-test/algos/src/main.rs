mod algos;
pub mod cbuf;

fn main() {
}

struct App {
    algos: Algos,
}

impl App {
    fn run(mut self) {
        let mut data_rx;
        loop {
            let data = match data_rx.recv_data() {
                Ok(data) => data,
                Err(e) => {
                    todo!()
                }
            };
            data
        }
    }
}
