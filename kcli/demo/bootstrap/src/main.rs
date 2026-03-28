fn main() {
    let argv = std::env::args().collect::<Vec<_>>();
    let parser = kcli::Parser::new();
    parser.parse_or_exit(&argv);
    println!("Bootstrap succeeded.");
}
