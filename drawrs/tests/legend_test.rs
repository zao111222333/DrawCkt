use drawrs::diagram_types::legend::Legend;
use std::collections::HashMap;

#[test]
fn test_requires_non_empty_mapping() {
    let mapping = HashMap::new();
    let result = Legend::new(mapping);
    assert!(result.is_err());
}

#[test]
fn test_default_values() {
    let mut mapping = HashMap::new();
    mapping.insert("Alpha".to_string(), "#ff0000".to_string());
    mapping.insert("Beta".to_string(), "#00ff00".to_string());

    let legend = Legend::new(mapping).unwrap();
    assert_eq!(legend.items(), 2);
    assert_eq!(legend.position(), [0.0, 0.0]);
}

#[test]
fn test_move() {
    let mut mapping = HashMap::new();
    mapping.insert("Alpha".to_string(), "#ff0000".to_string());
    let mut legend = Legend::new(mapping).unwrap();

    legend.move_to([10.0, 20.0]);
    assert_eq!(legend.position(), [10.0, 20.0]);
}

#[test]
fn test_update_mapping() {
    let mut mapping = HashMap::new();
    mapping.insert("Alpha".to_string(), "#ff0000".to_string());
    let mut legend = Legend::new(mapping).unwrap();

    let mut new_mapping = HashMap::new();
    new_mapping.insert("New".to_string(), "#000000".to_string());

    let result = legend.update_mapping(new_mapping);
    assert!(result.is_ok());
    assert_eq!(legend.items(), 1);
}
