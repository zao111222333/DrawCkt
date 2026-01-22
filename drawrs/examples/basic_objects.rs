use drawrs::{File, Object, Page};
use std::fs;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create a new file
    let mut file = File::new();

    // Create a page
    let mut page = Page::new(None, true);

    // Create some objects
    let mut obj1 = Object::new(None);
    obj1.set_value("Hello".to_string());
    obj1.set_position([100.0, 100.0]);
    obj1.set_fill_color(Some("#DDFFDD".to_string()));
    obj1.set_xml_parent(Some("1".to_string())); // Set parent to page root
    page.add_object(obj1.into());

    let mut obj2 = Object::new(None);
    obj2.set_value("World".to_string());
    obj2.set_position([300.0, 100.0]);
    obj2.set_rounded(Some(true));
    obj2.set_fill_color(Some("#FFDDDD".to_string()));
    obj2.set_xml_parent(Some("1".to_string()));
    page.add_object(obj2.into());

    let mut obj3 = Object::new(None);
    obj3.set_value("Rust!".to_string());
    obj3.set_position([500.0, 100.0]);
    obj3.set_fill_color(Some("#DDDDFF".to_string()));
    obj3.set_opacity(Some(80));
    obj3.set_xml_parent(Some("1".to_string()));
    page.add_object(obj3.into());

    file.add_page(page);

    // Write the file
    let output_file = "Basic Objects.drawio";
    let xml_content = file.write();
    fs::write(output_file, xml_content)?;
    println!("File written to: {}", output_file);
    Ok(())
}
