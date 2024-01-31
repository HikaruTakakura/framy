fn main() {
    if let Err(e) = framy::get_args().and_then(framy::run) {
        eprintln!("{}", e);
        std::process::exit(1);
    }
}
