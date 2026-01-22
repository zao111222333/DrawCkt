use drawrs::DrawrsError;
use drawrs::diagram_types::pie_chart::PieChart;
use std::collections::HashMap;

#[test]
fn test_initialization_empty_data_raises_error() {
    let data = HashMap::new();
    let result = PieChart::new(data);
    assert!(result.is_err());
    if let Err(e) = result {
        matches!(e, DrawrsError::EmptyData);
    }
}

#[test]
fn test_initialization_with_valid_data() {
    let mut data = HashMap::new();
    data.insert("A".to_string(), 10.0);
    data.insert("B".to_string(), 20.0);

    let chart = PieChart::new(data);
    assert!(chart.is_ok());
    let chart = chart.unwrap();
    assert_eq!(chart.len(), 2);
}

#[test]
fn test_update_data() {
    let mut data = HashMap::new();
    data.insert("A".to_string(), 10.0);
    let mut chart = PieChart::new(data).unwrap();

    let mut new_data = HashMap::new();
    new_data.insert("X".to_string(), 15.0);
    new_data.insert("Y".to_string(), 25.0);

    let result = chart.update_data(new_data);
    assert!(result.is_ok());
    assert_eq!(chart.len(), 2);
}

#[test]
fn test_move() {
    let mut data = HashMap::new();
    data.insert("A".to_string(), 10.0);
    let mut chart = PieChart::new(data).unwrap();

    chart.move_to([100.0, 200.0]);
    assert_eq!(chart.position(), [100.0, 200.0]);
}

#[test]
fn test_data_property_returns_copy() {
    let mut data = HashMap::new();
    data.insert("A".to_string(), 10.0);
    let chart = PieChart::new(data).unwrap();

    let chart_data = chart.data();
    assert_eq!(chart_data.len(), 1);
    assert_eq!(chart_data.get("A"), Some(&10.0));
}
