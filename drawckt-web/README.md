# DrawCkt Web

Web-based schematic viewer and editor using WASM and React.

## Features

- **WASM Integration**: All rendering logic runs in WebAssembly for optimal performance
- **Three-Column Layout**: 
  - Settings panel (collapsible) with schematic upload and layer styles editor
  - Symbols list with embedded draw.io viewers
  - Schematic view with embedded draw.io viewer
- **Resizable Columns**: Adjust column widths by dragging the resize handles
- **Layer Styles Editor**: Edit colors, stroke widths, font sizes, and priorities for each layer
- **Beautiful Logger**: Real-time log viewer with color-coded log levels
- **WASM Routing**: All embedded draw.io files are served through WASM, keeping data in memory

## Prerequisites

- Rust (with wasm-pack installed: `cargo install wasm-pack`)
- Node.js and npm

## Building

### 1. Build WASM Module

```bash
# From the drawckt-web directory
wasm-pack build --target web --out-dir pkg
```

Or use the provided script:

```bash
./build.sh
```

### 2. Install Dependencies

```bash
npm install
```

### 3. Development

```bash
npm run dev
```

The development server will start on `http://localhost:3000` (or another port if 3000 is busy).

### 4. Build for Production

First, build the WASM module:

```bash
wasm-pack build --target web --out-dir pkg
```

Then build the frontend:

```bash
npm run build
```

Or use the provided script:

```bash
./build-production.sh
```

The production build will be in the `dist/` directory.

### 5. Serve Static Files

You can serve the static files using any static file server:

```bash
# Using Python
cd dist && python3 -m http.server 8000

# Using Node.js serve
npx serve dist

# Using any other static file server
```

Then open `http://localhost:8000` in your browser.

## Subfolder Deployment

The application supports deployment in a subfolder (e.g., `https://hostname/path/to/drawckt`).

### Automatic Detection

The application automatically detects the base path from the current URL, so you can:

1. Copy the `dist/` folder contents to your web server's subfolder
2. Access it at `https://hostname/path/to/drawckt/`
3. All paths (assets, embedded files, routes) will automatically adjust

### Example Deployment

```bash
# Build the application
./build-production.sh

# Copy to web server subfolder
cp -r dist/* /var/www/html/path/to/drawckt/

# Or for Nginx
cp -r dist/* /usr/share/nginx/html/path/to/drawckt/
```

The application will work correctly at `https://yourdomain.com/path/to/drawckt/`.

### Manual Base Path (Optional)

If you need to set a specific base path at build time, you can use the `BASE_PATH` environment variable:

```bash
BASE_PATH=/path/to/drawckt npm run build
```

This is usually not necessary as the application auto-detects the base path at runtime.

## Usage

1. **Start the development server**: `npm run dev`
2. **Open the browser** to the displayed URL (usually `http://localhost:3000`)
3. **Upload a schematic.json file**:
   - Click "Upload schematic.json" in the settings panel
   - Select your schematic JSON file
   - The system will automatically:
     - Render all symbols using `render_symbols` logic
     - Render the schematic using `render_schematic` logic
     - Display symbols in the middle column
     - Display schematic in the right column
4. **View symbols**: All symbols are displayed as embedded draw.io viewers in the middle column
5. **View schematic**: The complete schematic is displayed in the right column
6. **Adjust layer styles**: 
   - Expand any layer in the Layer Styles section
   - Modify colors, stroke widths, font sizes, priorities, etc.
   - Click "Save Layer Styles" to apply changes
   - The schematic and symbols will automatically re-render
7. **Resize columns**: Drag the resize handles between columns to adjust widths
8. **Toggle menu**: Click the arrow button (◀/▶) to hide/show the settings panel
9. **View logs**: Click the logger button at the bottom to view real-time logs

## Architecture

- **WASM Module** (`src/lib.rs`): Contains all rendering logic and state management
- **React Frontend**: 
  - `App.tsx`: Main application component with three-column layout
  - `SettingsPanel.tsx`: Settings and layer styles editor
  - `SymbolsList.tsx`: List of symbol viewers
  - `SchematicView.tsx`: Schematic viewer
  - `Logger.tsx`: Log viewer component
- **WASM Bindings** (`src/wasm.ts`): TypeScript bindings for WASM functions
- **Routing**: All `/embedded/*` paths are intercepted and routed through WASM

## File Structure

```
drawckt-web/
├── src/
│   ├── lib.rs              # WASM module with rendering logic
│   ├── main.tsx            # React entry point
│   ├── App.tsx             # Main app component
│   ├── wasm.ts             # WASM TypeScript bindings
│   └── components/         # React components
│       ├── SettingsPanel.tsx
│       ├── LayerStylesEditor.tsx
│       ├── SymbolsList.tsx
│       ├── SchematicView.tsx
│       └── Logger.tsx
├── pkg/                    # Generated WASM package (after build)
├── Cargo.toml             # Rust dependencies
├── package.json           # Node.js dependencies
└── vite.config.ts         # Vite configuration
```

## Notes

- All data (schematic, symbols, layer styles) is stored in WASM memory for optimal performance
- The draw.io embedded viewers fetch files through a custom fetch interceptor that routes to WASM
- Symbol library configuration is not yet supported (placeholder in UI)
- The logger captures all console.log/info/warn/error calls for debugging

