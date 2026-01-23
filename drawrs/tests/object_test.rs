use drawrs::diagram::Object;

#[test]
fn test_default_values() {
    let obj = Object::new(None);
    assert_eq!(obj.value(), None);
    assert_eq!(obj.position(), [0.0, 0.0]);
}

#[test]
fn test_with_value() {
    let mut obj = Object::new(None);
    obj.set_value("Test Object".to_string());
    assert_eq!(obj.value(), Some(&"Test Object".to_string()));
}

#[test]
fn test_with_position() {
    let mut obj = Object::new(None);
    obj.set_position([100.0, 200.0]);
    assert_eq!(obj.position(), [100.0, 200.0]);
}

#[test]
fn test_with_dimensions() {
    let mut obj = Object::new(None);
    obj.set_value("Test".to_string());
    obj.set_width(150.0);
    obj.set_height(100.0);
    assert_eq!(obj.width(), 150.0);
    assert_eq!(obj.height(), 100.0);
}

#[test]
fn test_apply_style_string() {
    let mut obj = Object::new(None);
    let style_str = "whiteSpace=wrap;rounded=1;fillColor=#6a00ff;strokeColor=#000000;";
    obj.parse_and_set_style(style_str);

    assert_eq!(obj.fill_color(), Some(&"#6a00ff".to_string()));
    assert_eq!(obj.stroke_color(), Some(&"#000000".to_string()));
    assert_eq!(obj.rounded(), Some(true));
}

#[test]
fn test_set_fill_color() {
    let mut obj = Object::new(None);
    obj.set_fill_color(Some("#FF6B6B".to_string()));
    assert_eq!(obj.fill_color(), Some(&"#FF6B6B".to_string()));
}

#[test]
fn test_set_stroke_color() {
    let mut obj = Object::new(None);
    obj.set_stroke_color(Some("#000000".to_string()));
    assert_eq!(obj.stroke_color(), Some(&"#000000".to_string()));
}

#[test]
fn test_set_rounded() {
    let mut obj = Object::new(None);
    obj.set_rounded(Some(true));
    assert_eq!(obj.rounded(), Some(true));
}

#[test]
fn test_set_opacity() {
    let mut obj = Object::new(None);
    obj.set_opacity(Some(50));
    assert_eq!(obj.opacity(), Some(50));
}

#[test]
fn test_xml_generation() {
    let mut obj = Object::new(None);
    obj.set_value("Test".to_string());
    let xml = obj.xml().to_string();
    assert!(xml.contains("mxCell"));
    assert!(xml.contains("Test"));
    assert!(xml.contains("mxGeometry"));
}
