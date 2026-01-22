use drawrs::file::File;
use drawrs::page::Page;

#[test]
fn test_default_values() {
    let file = File::new();
    assert_eq!(file.pages.len(), 0);
    assert_eq!(file.host, "Electron");
    assert_eq!(file.file_type, "device");
    assert_eq!(file.version, "21.6.5");
}

#[test]
fn test_with_custom_name() {
    let file = File::with_name("Test Name.drawio".to_string());
    assert_eq!(file.pages.len(), 0);
    assert_eq!(file.host, "Electron");
}

#[test]
fn test_add_page_basic() {
    let mut file = File::new();
    let page = Page::new(None, true);
    file.add_page(page);
    assert_eq!(file.pages.len(), 1);
}

#[test]
fn test_add_multiple_pages() {
    let mut file = File::new();
    let page1 = Page::new(None, true);
    let page2 = Page::new(None, true);
    let page3 = Page::new(None, true);
    file.add_page(page1);
    file.add_page(page2);
    file.add_page(page3);
    assert_eq!(file.pages.len(), 3);
}

#[test]
fn test_write_basic() {
    let mut file = File::new();
    let page = Page::new(None, true);
    file.add_page(page);

    let xml_content = file.write();
    assert!(!xml_content.is_empty());
    assert!(xml_content.contains("<mxfile"));
    assert!(xml_content.contains("</mxfile>"));
}
