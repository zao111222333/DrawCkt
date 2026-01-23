use drawrs::{DrawFile, Edge, Object, Page};
use std::fs;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create a new file
    let mut file = DrawFile::new();

    // Create a page
    let mut page = Page::new(None, true);

    // Create D Latch circuit using NAND gates
    // Position constants
    let gate_width = 105.0;
    let gate_height = 93.0;
    let gate_spacing_x = 200.0;
    let gate_spacing_y = 200.0;

    // Left column: Inputs and first NAND gate
    let left_x = 200.0;
    let input_y = 150.0;
    let gate1_y = 300.0;

    // Right column: Second NAND gate and outputs
    let right_x = left_x + gate_spacing_x;
    let gate2_y = gate1_y + gate_spacing_y;
    let output_y = 600.0;

    // Input labels: D (Data) and CLK (Clock)
    let mut d_input = Object::new(None);
    d_input.set_value("D".to_string());
    d_input.set_position([50.0, input_y]);
    d_input.set_width(80.0);
    d_input.set_height(40.0);
    d_input.set_fill_color(Some("none".to_string()));
    d_input.set_stroke_color(Some("#000000".to_string()));
    d_input.set_xml_parent(Some("1".to_string()));
    let d_input_id = d_input.id().to_string();
    page.add_object(d_input.into());

    let mut clk_input = Object::new(None);
    clk_input.set_value("CLK".to_string());
    clk_input.set_position([50.0, input_y + 100.0]);
    clk_input.set_width(80.0);
    clk_input.set_height(40.0);
    clk_input.set_fill_color(Some("none".to_string()));
    clk_input.set_stroke_color(Some("#000000".to_string()));
    clk_input.set_xml_parent(Some("1".to_string()));
    let clk_input_id = clk_input.id().to_string();
    page.add_object(clk_input.into());

    // NAND Gate 1 (top-left) - This represents a NAND gate
    // In Draw.io, NAND gates are often drawn using "or" shape with thick input lines
    let mut nand1 = Object::new(None);
    nand1.set_value("".to_string());
    nand1.set_position([left_x, gate1_y]);
    nand1.set_width(gate_width);
    nand1.set_height(gate_height);
    nand1.set_fill_color(Some("#FFFFFF".to_string()));
    nand1.set_stroke_color(Some("#000000".to_string()));
    // Use "or" shape for NAND gate (common in circuit diagrams)
    nand1.set_shape("or".to_string());
    nand1.set_xml_parent(Some("1".to_string()));
    let nand1_id = nand1.id().to_string();
    page.add_object(nand1.into());

    // NAND Gate 2 (bottom-right)
    let mut nand2 = Object::new(None);
    nand2.set_value("".to_string());
    nand2.set_position([right_x, gate2_y]);
    nand2.set_width(gate_width);
    nand2.set_height(gate_height);
    nand2.set_fill_color(Some("#FFFFFF".to_string()));
    nand2.set_stroke_color(Some("#000000".to_string()));
    nand2.set_shape("or".to_string());
    nand2.set_xml_parent(Some("1".to_string()));
    let nand2_id = nand2.id().to_string();
    page.add_object(nand2.into());

    // Output label: Q
    let mut q_output = Object::new(None);
    q_output.set_value("Q".to_string());
    q_output.set_position([right_x + gate_width + 50.0, output_y]);
    q_output.set_width(80.0);
    q_output.set_height(40.0);
    q_output.set_fill_color(Some("none".to_string()));
    q_output.set_stroke_color(Some("#000000".to_string()));
    q_output.set_xml_parent(Some("1".to_string()));
    let q_output_id = q_output.id().to_string();
    page.add_object(q_output.into());

    // Connection points (small circles for circuit junctions)
    // Junction 1: Between D input and NAND1
    let mut junction1 = Object::new(None);
    junction1.set_value("".to_string());
    junction1.set_position([left_x - 8.0, gate1_y + gate_height / 2.0 - 8.0]);
    junction1.set_width(16.0);
    junction1.set_height(16.0);
    junction1.set_fill_color(Some("#000000".to_string()));
    junction1.set_stroke_color(Some("#000000".to_string()));
    junction1.set_shape("ellipse".to_string());
    junction1.set_aspect("fixed".to_string());
    junction1.set_xml_parent(Some("1".to_string()));
    let junction1_id = junction1.id().to_string();
    page.add_object(junction1.into());

    // Junction 2: Between NAND1 output and NAND2 input
    let mut junction2 = Object::new(None);
    junction2.set_value("".to_string());
    junction2.set_position([right_x - 8.0, gate2_y + gate_height / 2.0 - 8.0]);
    junction2.set_width(16.0);
    junction2.set_height(16.0);
    junction2.set_fill_color(Some("#000000".to_string()));
    junction2.set_stroke_color(Some("#000000".to_string()));
    junction2.set_shape("ellipse".to_string());
    junction2.set_aspect("fixed".to_string());
    junction2.set_xml_parent(Some("1".to_string()));
    let junction2_id = junction2.id().to_string();
    page.add_object(junction2.into());

    // Create edges (connections)
    // D -> Junction1 -> NAND1 input (top)
    let mut edge1 = Edge::new(None);
    edge1.set_source(Some(d_input_id.clone()));
    edge1.set_target(Some(junction1_id.clone()));
    edge1.set_line_end_target(Some("classic".to_string()));
    edge1.set_stroke_width(Some(5.0));
    edge1.set_xml_parent(Some("1".to_string()));
    page.add_object(edge1.into());

    let mut edge1b = Edge::new(None);
    edge1b.set_source(Some(junction1_id.clone()));
    edge1b.set_target(Some(nand1_id.clone()));
    edge1b.set_line_end_target(Some("classic".to_string()));
    edge1b.set_stroke_width(Some(5.0));
    edge1b.set_xml_parent(Some("1".to_string()));
    page.add_object(edge1b.into());

    // CLK -> NAND1 input (bottom)
    let mut edge2 = Edge::new(None);
    edge2.set_source(Some(clk_input_id.clone()));
    edge2.set_target(Some(nand1_id.clone()));
    edge2.set_line_end_target(Some("classic".to_string()));
    edge2.set_stroke_width(Some(5.0));
    edge2.set_xml_parent(Some("1".to_string()));
    page.add_object(edge2.into());

    // NAND1 output -> Junction2 -> NAND2 input (top)
    let mut edge3 = Edge::new(None);
    edge3.set_source(Some(nand1_id.clone()));
    edge3.set_target(Some(junction2_id.clone()));
    edge3.set_line_end_target(Some("blockThin".to_string()));
    edge3.set_end_fill_target(true);
    edge3.set_end_size(Some(25));
    edge3.set_stroke_width(Some(5.0));
    edge3.set_xml_parent(Some("1".to_string()));
    page.add_object(edge3.into());

    let mut edge3b = Edge::new(None);
    edge3b.set_source(Some(junction2_id.clone()));
    edge3b.set_target(Some(nand2_id.clone()));
    edge3b.set_line_end_target(Some("classic".to_string()));
    edge3b.set_stroke_width(Some(5.0));
    edge3b.set_xml_parent(Some("1".to_string()));
    page.add_object(edge3b.into());

    // CLK -> NAND2 input (bottom)
    let mut edge4 = Edge::new(None);
    edge4.set_source(Some(clk_input_id.clone()));
    edge4.set_target(Some(nand2_id.clone()));
    edge4.set_line_end_target(Some("classic".to_string()));
    edge4.set_stroke_width(Some(5.0));
    edge4.set_xml_parent(Some("1".to_string()));
    page.add_object(edge4.into());

    // NAND2 output -> Q
    let mut edge5 = Edge::new(None);
    edge5.set_source(Some(nand2_id.clone()));
    edge5.set_target(Some(q_output_id.clone()));
    edge5.set_line_end_target(Some("blockThin".to_string()));
    edge5.set_end_fill_target(true);
    edge5.set_end_size(Some(25));
    edge5.set_stroke_width(Some(5.0));
    edge5.set_xml_parent(Some("1".to_string()));
    page.add_object(edge5.into());

    // Feedback: Q -> Junction1 (feedback path)
    let mut edge6 = Edge::new(None);
    edge6.set_source(Some(q_output_id.clone()));
    edge6.set_target(Some(junction1_id.clone()));
    edge6.set_line_end_target(Some("classic".to_string()));
    edge6.set_stroke_width(Some(5.0));
    edge6.set_xml_parent(Some("1".to_string()));
    page.add_object(edge6.into());

    // Add title
    let mut title = Object::new(None);
    title.set_value("(a) Latch".to_string());
    title.set_position([left_x, 20.0]);
    title.set_width(400.0);
    title.set_height(60.0);
    title.set_fill_color(Some("none".to_string()));
    title.set_stroke_color(Some("none".to_string()));
    title.set_xml_parent(Some("1".to_string()));
    page.add_object(title.into());

    file.add_page(page);

    // Write the file
    let output_file = "Latch Circuit.drawio";
    let xml_content = file.write();
    fs::write(output_file, xml_content)?;
    println!("Latch circuit written to: {}", output_file);
    Ok(())
}
