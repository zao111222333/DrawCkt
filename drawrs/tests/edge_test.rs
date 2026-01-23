use drawrs::diagram::Edge;

#[test]
fn test_default_values() {
    let edge = Edge::new(None);
    assert_eq!(edge.waypoints(), "orthogonal");
    assert_eq!(edge.connection(), "line");
    assert_eq!(edge.pattern(), "solid");
    assert_eq!(edge.edge(), 1);
}

#[test]
fn test_with_label() {
    let mut edge = Edge::new(None);
    edge.base_mut().value = Some("Connection".to_string());
    assert_eq!(edge.base().value, Some("Connection".to_string()));
}

#[test]
fn test_stroke_color() {
    let mut edge = Edge::new(None);
    edge.set_stroke_color(Some("#FF0000".to_string()));
    assert_eq!(edge.stroke_color(), Some(&"#FF0000".to_string()));
}

#[test]
fn test_stroke_width() {
    let mut edge = Edge::new(None);
    edge.set_stroke_width(Some(3.0));
    assert_eq!(edge.stroke_width(), Some(3.0));
}

#[test]
fn test_line_end_target() {
    let mut edge = Edge::new(None);
    edge.set_line_end_target(Some("classic".to_string()));
    assert_eq!(edge.line_end_target(), Some(&"classic".to_string()));
}

#[test]
fn test_line_end_source() {
    let mut edge = Edge::new(None);
    edge.set_line_end_source(Some("classic".to_string()));
    assert_eq!(edge.line_end_source(), Some(&"classic".to_string()));
}

#[test]
fn test_end_fill_target() {
    let mut edge = Edge::new(None);
    edge.set_end_fill_target(true);
    assert_eq!(edge.end_fill_target(), true);
}

#[test]
fn test_end_fill_source() {
    let mut edge = Edge::new(None);
    edge.set_end_fill_source(true);
    assert_eq!(edge.end_fill_source(), true);
}

#[test]
fn test_end_size() {
    let mut edge = Edge::new(None);
    edge.set_end_size(Some(12));
    assert_eq!(edge.end_size(), Some(12));
}

#[test]
fn test_start_size() {
    let mut edge = Edge::new(None);
    edge.set_start_size(Some(10));
    assert_eq!(edge.start_size(), Some(10));
}

#[test]
fn test_opacity() {
    let mut edge = Edge::new(None);
    edge.set_opacity(Some(50));
    assert_eq!(edge.opacity(), Some(50));
}

#[test]
fn test_xml_generation() {
    let mut edge = Edge::new(None);
    edge.base_mut().value = Some("Test Label".to_string());
    let xml = edge.xml().to_string();
    assert!(xml.contains("mxCell"));
    assert!(xml.contains("edge"));
}
