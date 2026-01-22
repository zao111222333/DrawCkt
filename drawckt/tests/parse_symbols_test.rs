use drawckt::renderer::Renderer;

#[test]
fn parse_symbols_file() {
    let content = include_str!("parse_symbols_test.drawio");

    // 解析文件
    let pages = Renderer::parse_symbols_file(&content).expect("Failed to parse symbols file");

    // 使用 insta 快照测试每个 SymbolPageData 的 debug format
    insta::assert_debug_snapshot!("parse_symbols_file", pages);
}

#[test]
fn parse_symbols_file_iopin() {
    let content = include_str!("iopin.drawio");

    // 解析文件
    let pages = Renderer::parse_symbols_file(&content).expect("Failed to parse symbols file");

    // 使用 insta 快照测试每个 SymbolPageData 的 debug format
    insta::assert_debug_snapshot!("iopin", pages);
}

#[test]
fn parse_symbols_file_rupolym() {
    let content = include_str!("rupolym.drawio");

    // 解析文件
    let pages = Renderer::parse_symbols_file(&content).expect("Failed to parse symbols file");

    // 使用 insta 快照测试每个 SymbolPageData 的 debug format
    insta::assert_debug_snapshot!("rupolym", pages);
}
