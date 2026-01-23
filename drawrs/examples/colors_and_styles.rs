use drawrs::{File, Object, Page, StandardColor};
use std::fs;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create a new file
    let mut file = File::new();

    // Create a page
    let mut page = Page::new(None, true);

    // Create objects with different colors
    let mut obj1 = Object::new(None);
    obj1.set_value("Standard Color".to_string());
    obj1.set_position([100.0, 100.0]);
    obj1.set_fill_color(Some(StandardColor::Blue5.value().to_string()));
    obj1.set_xml_parent(Some("1".to_string()));
    page.add_object(obj1.into());

    let mut obj2 = Object::new(None);
    obj2.set_value("Custom Color".to_string());
    obj2.set_position([300.0, 100.0]);
    obj2.set_fill_color(Some("#FF6B6B".to_string()));
    obj2.set_stroke_color(Some("#000000".to_string()));
    obj2.set_xml_parent(Some("1".to_string()));
    page.add_object(obj2.into());

    let mut obj3 = Object::new(None);
    obj3.set_value("Rounded".to_string());
    obj3.set_position([500.0, 100.0]);
    obj3.set_rounded(Some(true));
    obj3.set_fill_color(Some("#4ECDC4".to_string()));
    obj3.set_xml_parent(Some("1".to_string()));
    page.add_object(obj3.into());

    let mut obj4 = Object::new(None);
    obj4.set_value("Semi-transparent".to_string());
    obj4.set_position([100.0, 250.0]);
    obj4.set_fill_color(Some("#95E1D3".to_string()));
    obj4.set_opacity(Some(50));
    obj4.set_xml_parent(Some("1".to_string()));
    page.add_object(obj4.into());

    // Create object with style string
    let mut obj5 = Object::new(None);
    obj5.set_value("Styled".to_string());
    obj5.set_position([300.0, 250.0]);
    obj5.apply_style_string(
        "rounded=1;fillColor=#6a00ff;strokeColor=#000000;opacity=80;",
    );
    obj5.set_xml_parent(Some("1".to_string()));
    page.add_object(obj5.into());

    file.add_page(page);

    // Write the file
    let output_file = "Colors and Styles.drawio";
    let xml_content = file.write();
    fs::write(output_file, xml_content)?;
    println!("Colors and styles example written to: {}", output_file);
    Ok(())
}
