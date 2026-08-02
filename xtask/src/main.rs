fn main() {
    if let Err(error) = xtask::run_from_environment() {
        eprintln!("xtask: {error}");
        std::process::exit(1);
    }
}
