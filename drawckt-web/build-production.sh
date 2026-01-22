#!/bin/bash
set -e

echo "Building WASM module..."
cd "$(dirname "$0")"
wasm-pack build --target web --out-dir pkg

echo "Building frontend..."
npm run build

echo "Build complete! Static files are in the 'dist' directory."
echo "You can serve them with:"
echo "  cd dist && python3 -m http.server 8000"
echo "or"
echo "  npx serve dist"

