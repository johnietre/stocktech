macro_rules! die {
    ($($args:tt)*) => {{
        ::std::eprintln!($($args)*);
        ::std::process::exit(1)
    }}
}

fn main() {
}

fn run_client() {
    let size = term_size();
}

fn term_size() -> (u16, u16) {
    match crossterm::terminal::size() {
        Ok(t) => t,
        Err(e) => die!("error getting terminal size: {e}"),
    }
}
