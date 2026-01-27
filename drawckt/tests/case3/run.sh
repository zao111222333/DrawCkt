#!/bin/bash
set -e

rm -rf symbols schematic.drawio
cargo run --bin render_symbols tests/case3/schematic.json
cargo run --bin render_schematic tests/case3/schematic.json symbols
/Applications/draw.io.app/Contents/MacOS/draw.io schematic.drawio