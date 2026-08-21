fn main() {
    if let Err(error) = sticky::cli::run() {
        eprintln!("Error: {error:#}");
        std::process::exit(1);
    }
}
