fn main() {
    println!("{}", std::mem::size_of::<std::sync::Mutex<u8>>());
    println!("{}", std::mem::size_of::<std::sync::Mutex<u64>>());
    println!("{}", std::mem::size_of::<std::sync::Mutex<String>>());
    println!("{}", std::mem::size_of::<std::sync::Mutex<[u8; 1024]>>());

    println!("{}", std::mem::size_of::<std::sync::RwLock<u8>>());
    println!("{}", std::mem::size_of::<std::sync::RwLock<u64>>());
    println!("{}", std::mem::size_of::<std::sync::RwLock<String>>());
    println!("{}", std::mem::size_of::<std::sync::RwLock<[u8; 1024]>>());

    let s1: std::sync::Arc<str> = "a".into();
    let s2: std::sync::Arc<str> = "b".into();
    println!("{}", s1 > s2);
    println!("{}", s1 == s2);
    println!("{}", s1 < s2);

    println!("{}", &s1 == &s2);
    println!("{}", &s1 < &s2);

    let mut map = std::collections::BTreeMap::new();
    map.insert(std::sync::Arc::clone(&s1), 1);
    map.insert(std::sync::Arc::clone(&s2), 2);
    println!("{:?}", map.get("a"));

    println!("{}", 0.5f64.round());
    println!("{}", (-0.5f64).round());
    println!("{}", 0.4f64.round());
    println!("{}", (-0.4f64).round());
    println!("{}", 0.6f64.round());
    println!("{}", (-0.6f64).round());
}
