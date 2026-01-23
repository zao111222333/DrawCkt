use drawrs::{BarChart, DrawFile, Page};
use std::collections::HashMap;
use std::fs;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create a new file
    let mut file = DrawFile::new();

    // Create a page
    let mut page = Page::new(None, true);

    // Create bar chart data
    let mut data = HashMap::new();
    data.insert("Q1".to_string(), 100.0);
    data.insert("Q2".to_string(), 150.0);
    data.insert("Q3".to_string(), 120.0);
    data.insert("Q4".to_string(), 180.0);

    // Create bar chart
    let mut chart = BarChart::new(data)?;
    chart.move_to([100.0, 100.0]);

    // Add chart objects to page
    for mut obj in chart.objects {
        obj.set_xml_parent(Some("1".to_string()));
        page.add_object(obj.into());
    }

    file.add_page(page);

    // Write the file
    let output_file = "Bar Chart Example.drawio";
    let xml_content = file.write();
    fs::write(output_file, xml_content)?;
    println!("Bar chart written to: {}", output_file);
    Ok(())
}
