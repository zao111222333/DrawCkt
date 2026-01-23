use drawckt::renderer::{Renderer, SymbolContexts};
use drawckt::schematic::Schematic;
use drawckt::DrawcktResult;
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
        warn!(
            "Usage: {} <json_file> [symbols_dir] [style_file] [output_file]",
            args[0]
        );
        warn!("  json_file: Input JSON schematic file");
        warn!("  symbols_dir: Input symbols directory (default: ./symbols)");
        warn!("  style_file: Input style.json file (optional, uses default if not provided)");
        warn!("  output_file: Output schematic.drawio file (default: schematic.drawio)");
        return Ok(());
    }

    let json_path = &args[1];
    let symbols_dir = args.get(2).map(|s| s.as_str()).unwrap_or("./symbols");
    let style_file = args.get(3);
    let output_file = args
        .get(4)
        .map(|s| s.as_str())
        .unwrap_or("schematic.drawio");

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

    // Load symbols from directory structure: {symbols_dir}/{lib}/{cell}.drawio
    let symbol_contexts = SymbolContexts::load_from_dir(symbols_dir)?;

    // Create renderer and render schematic
    let output_content =
        Renderer::new(&schematic, &layer_styles).render_schematic_file(&symbol_contexts)?;

    // Write output to file
    fs::write(output_file, output_content)?;
    log::info!("Schematic rendered to: {:?}", output_file);
    Ok(())
}
