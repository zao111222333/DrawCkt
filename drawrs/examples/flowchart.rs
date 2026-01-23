use drawrs::{DrawFile, Edge, Object, Page};
use std::fs;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create a new file
    let mut file = DrawFile::new();

    // Create a page
    let mut page = Page::new(None, true);

    // Create flowchart objects
    let mut start = Object::new(None);
    start.set_value("Start".to_string());
    start.set_position([200.0, 50.0]);
    start.set_fill_color(Some("#D5E8D4".to_string()));
    start.set_rounded(Some(true));
    start.set_xml_parent(Some("1".to_string()));
    let start_id = start.id().to_string();
    page.add_object(start.into());

    let mut process = Object::new(None);
    process.set_value("Process".to_string());
    process.set_position([200.0, 150.0]);
    process.set_fill_color(Some("#D5E8D4".to_string()));
    process.set_xml_parent(Some("1".to_string()));
    let process_id = process.id().to_string();
    page.add_object(process.into());

    let mut decision = Object::new(None);
    decision.set_value("Decision?".to_string());
    decision.set_position([200.0, 250.0]);
    decision.set_fill_color(Some("#FFF2CC".to_string()));
    decision.set_rounded(Some(true));
    decision.set_xml_parent(Some("1".to_string()));
    let decision_id = decision.id().to_string();
    page.add_object(decision.into());

    let mut end = Object::new(None);
    end.set_value("End".to_string());
    end.set_position([200.0, 350.0]);
    end.set_fill_color(Some("#F8CECC".to_string()));
    end.set_rounded(Some(true));
    end.set_xml_parent(Some("1".to_string()));
    let end_id = end.id().to_string();
    page.add_object(end.into());

    // Create edges
    let mut edge1 = Edge::new(None);
    edge1.set_source(Some(start_id.clone()));
    edge1.set_target(Some(process_id.clone()));
    edge1.set_line_end_target(Some("classic".to_string()));
    edge1.set_xml_parent(Some("1".to_string()));
    page.add_object(edge1.into());

    let mut edge2 = Edge::new(None);
    edge2.set_source(Some(process_id));
    edge2.set_target(Some(decision_id.clone()));
    edge2.set_line_end_target(Some("classic".to_string()));
    edge2.set_xml_parent(Some("1".to_string()));
    page.add_object(edge2.into());

    let mut edge3 = Edge::new(None);
    edge3.set_source(Some(decision_id));
    edge3.set_target(Some(end_id));
    edge3.set_line_end_target(Some("classic".to_string()));
    edge3.set_xml_parent(Some("1".to_string()));
    page.add_object(edge3.into());

    file.add_page(page);

    // Write the file
    let output_file = "Simple Flowchart.drawio";
    let xml_content = file.write();
    fs::write(output_file, xml_content)?;
    println!("Flowchart written to: {}", output_file);
    Ok(())
}
