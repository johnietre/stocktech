pub struct Data {
    pub sym: Arc<str>,
    pub close: f64,
    pub timestamp: u64,
}

pub struct DataRx {
}

impl DataRx {
    pub fn recv_data(&self) -> Result<Data, ()> {
    }
}
