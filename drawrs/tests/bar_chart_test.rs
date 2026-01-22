use drawrs::DrawrsError;
use drawrs::diagram_types::bar_chart::BarChart;
use std::collections::HashMap;

#[test]
fn test_initialization_empty_data_raises_error() {
    let data = HashMap::new();
    let result = BarChart::new(data);
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

    let chart = BarChart::new(data);
    assert!(chart.is_ok());
    let chart = chart.unwrap();
    assert_eq!(chart.len(), 2);
}

#[test]
fn test_calculate_scale() {
    let mut data = HashMap::new();
    data.insert("A".to_string(), 50.0);
    data.insert("B".to_string(), 100.0);

    let chart = BarChart::new(data).unwrap();
    let scale = chart.calculate_scale();
    // max_bar_height / max_value = 200 / 100 = 2.0
    assert!((scale - 2.0).abs() < 0.001);
}

#[test]
fn test_update_data() {
    let mut data = HashMap::new();
    data.insert("A".to_string(), 10.0);
    let mut chart = BarChart::new(data).unwrap();

    let mut new_data = HashMap::new();
    new_data.insert("X".to_string(), 15.0);
    new_data.insert("Y".to_string(), 25.0);
    new_data.insert("Z".to_string(), 30.0);

    let result = chart.update_data(new_data);
    assert!(result.is_ok());
    assert_eq!(chart.len(), 3);
}

#[test]
fn test_move() {
    let mut data = HashMap::new();
    data.insert("A".to_string(), 10.0);
    let mut chart = BarChart::new(data).unwrap();

    chart.move_to([100.0, 200.0]);
    assert_eq!(chart.position(), [100.0, 200.0]);
}
