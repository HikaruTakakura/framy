fn main() {
    if let Err(e) = framy::get_args().and_then(|(paths, config)| framy::run(paths, config)) {
        eprintln!("{}", e);
        std::process::exit(1);
    }
}
