use drawrs::{File, Page, PieChart};
use std::collections::HashMap;
use std::fs;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create a new file
    let mut file = File::new();

    // Create a page
    let mut page = Page::new(None, true);

    // Create pie chart data
    let mut data = HashMap::new();
    data.insert("Red".to_string(), 30.0);
    data.insert("Blue".to_string(), 25.0);
    data.insert("Green".to_string(), 20.0);
    data.insert("Yellow".to_string(), 15.0);
    data.insert("Purple".to_string(), 10.0);

    // Create pie chart
    let mut chart = PieChart::new(data)?;
    chart.move_to([200.0, 200.0]);

    // Add chart objects to page
    // Objects already have their parent set correctly by PieChart:
    // - Groups have parent="1" (page root)
    // - Slices and labels have parent set to their group ID
    for obj in chart.objects {
        page.add_object(obj.into());
    }

    file.add_page(page);

    // Write the file
    let output_file = "Pie Chart Example.drawio";
    let xml_content = file.write();
    fs::write(output_file, xml_content)?;
    println!("Pie chart written to: {}", output_file);
    Ok(())
}
