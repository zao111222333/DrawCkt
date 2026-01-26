use drawckt::renderer::Renderer;

#[test]
fn parse_drawio_file_iopin() {
    let content = include_str!("test_parse_drawio/iopin.drawio");

    // 解析文件
    let pages = Renderer::parse_drawio_file(&content).expect("Failed to parse symbols file");

    // 使用 insta 快照测试每个 SymbolPageData 的 debug format
    insta::assert_debug_snapshot!("iopin", pages);
}

#[test]
fn parse_drawio_file_rupolym() {
    let content = include_str!("test_parse_drawio/rupolym.drawio");

    // 解析文件
    let pages = Renderer::parse_drawio_file(&content).expect("Failed to parse symbols file");

    // 使用 insta 快照测试每个 SymbolPageData 的 debug format
    insta::assert_debug_snapshot!("rupolym", pages);
}

#[test]
fn parse_drawio_file_schematic() {
    let content = include_str!("test_parse_drawio/schematic.drawio");

    // 解析文件
    let pages = Renderer::parse_drawio_file(&content).expect("Failed to parse symbols file");

    // 使用 insta 快照测试每个 SymbolPageData 的 debug format
    insta::assert_debug_snapshot!("schematic", pages);
}
