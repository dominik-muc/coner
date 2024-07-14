fn main() {
    if let Err(e) = server::run() {
        eprintln!("Server error: {}", e)
    }
}
