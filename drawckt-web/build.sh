#!/bin/bash
set -e

echo "Building WASM module..."
wasm-pack build --target web --out-dir pkg

echo "WASM build complete!"
echo "Next steps:"
echo "1. Run 'npm install' to install dependencies"
echo "2. Run 'npm run dev' to start the development server"

