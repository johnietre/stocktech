fn main() {
    println!("{}", dec(15.5));
}

const fn dec(f: f64) -> f64 {
    f - trunc(f)
}

const fn trunc(f: f64) -> f64 {
    f as u64 as f64
}
