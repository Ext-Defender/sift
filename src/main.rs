fn main() {
    if let Err(e) = sift::get_args().and_then(sift::run) {
        eprintln!("{}", e);
        std::process::exit(1);
    }
}
