use drawrs::page::Page;

#[test]
fn test_default_values() {
    let page = Page::new(None, true);
    assert_eq!(page.objects().len(), 2); // Two basic mxCell objects
}

#[test]
fn test_viewport_settings() {
    let page = Page::new(None, true);
    // Note: We need to add getters for dx, dy, etc.
    // For now, just check that page was created
    assert_eq!(page.objects().len(), 2);
}

#[test]
fn test_with_custom_name() {
    // Note: We need to add a way to set custom name
    let page = Page::new(None, true);
    assert_eq!(page.objects().len(), 2);
}

#[test]
fn test_add_object() {
    use drawrs::diagram::Object;
    let mut page = Page::new(None, true);
    let initial_count = page.objects().len();
    let obj = Object::new(None);
    // Note: We need to add a way to add objects to page
    page.add_object(obj.into());
    assert_eq!(page.objects().len(), initial_count + 1);
}
