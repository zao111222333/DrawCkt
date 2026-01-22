use drawrs::xml_base::XMLBase;

#[test]
fn test_default_values() {
    let xml_base = XMLBase::new(None);
    assert_eq!(xml_base.xml_class, "xml_tag");
    assert!(xml_base.xml_parent.is_none());
    assert!(xml_base.tag.is_none());
    assert!(xml_base.tooltip.is_none());
}

#[test]
fn test_custom_id() {
    let xml_base = XMLBase::new(Some("custom_id".to_string()));
    assert_eq!(xml_base.id, "custom_id");
}

#[test]
fn test_custom_xml_class() {
    let xml_base = XMLBase::with_class("mxCell".to_string());
    assert_eq!(xml_base.xml_class, "mxCell");
}

#[test]
fn test_xml_ify() {
    assert_eq!(XMLBase::xml_ify(">"), "&gt;");
    assert_eq!(XMLBase::xml_ify("<"), "&lt;");
    assert_eq!(XMLBase::xml_ify("&"), "&amp;");
    assert_eq!(XMLBase::xml_ify("\""), "&quot;");
    assert_eq!(XMLBase::xml_ify("'"), "&apos;");
    assert_eq!(XMLBase::xml_ify(""), "");
    assert_eq!(XMLBase::xml_ify("hello"), "hello");
    assert_eq!(
        XMLBase::xml_ify("<div>&test</div>"),
        "&lt;div&gt;&amp;test&lt;/div&gt;"
    );
}

#[test]
fn test_translate_txt() {
    use std::collections::HashMap;
    let mut replacements = HashMap::new();
    replacements.insert('a', "X");
    replacements.insert('c', "Z");
    assert_eq!(XMLBase::translate_txt("abc", &replacements), "XbZ");
}
