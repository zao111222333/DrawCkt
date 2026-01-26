use drawckt::DrawcktResult;
use drawckt::renderer::Renderer;
use drawckt::schematic::Schematic;
use env_logger::{Builder, Env};
use log::warn;
use std::fs;
use std::io::Write;

fn main() -> DrawcktResult<()> {
    Builder::from_env(Env::default().default_filter_or("info"))
        .format(|buf, record| {
            let file_name = record.file().unwrap_or("<unknown>");
            let line = record.line().unwrap_or(0);
            writeln!(
                buf,
                "{} {}:{}: {}",
                record.level(),
                file_name,
                line,
                record.args()
            )
        })
        .init();

    let args: Vec<String> = std::env::args().collect();
    if args.len() < 2 {
        warn!("Usage: {} <json_file> [style_file] [output_dir]", args[0]);
        warn!("  json_file: Input JSON schematic file");
        warn!("  style_file: Input style.json file (optional, uses default if not provided)");
        warn!("  output_dir: Output directory for symbol files (default: ./symbols)");
        return Ok(());
    }

    let json_path = &args[1];
    let style_file = args.get(2);
    let output_dir = args.get(3).map(|s| s.as_str()).unwrap_or("./symbols");

    // Read JSON file
    let json_content = fs::read_to_string(json_path)?;
    let schematic: Schematic = serde_json::from_str(&json_content)?;

    // Read style file if provided, otherwise use default
    let layer_styles = if let Some(style_path) = style_file {
        let style_content = fs::read_to_string(style_path)?;
        serde_json::from_str(&style_content)?
    } else {
        drawckt::schematic::LayerStyles::default()
    };

    // Create renderer and render symbols
    let renderer = Renderer::new(&schematic, &layer_styles);
    let symbol_contexts = renderer.render_symbols_file()?;
    // Write symbols to directory structure
    symbol_contexts.write_to_dir(output_dir)?;

    Ok(())
}
