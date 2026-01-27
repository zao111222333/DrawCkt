import { defineConfig } from 'vite';
import react from '@vitejs/plugin-react';
import wasm from 'vite-plugin-wasm';
import { resolve, dirname } from 'path';
import { fileURLToPath } from 'url';
import { readFileSync, mkdirSync, existsSync, unlinkSync, watch, copyFileSync, readdirSync, statSync } from 'fs';
import { execSync } from 'child_process';

// Get __dirname equivalent in ES modules
const __filename = fileURLToPath(import.meta.url);
const __dirname = dirname(__filename);

// Support subfolder deployment via BASE_PATH environment variable
// Example: BASE_PATH=/path/to/drawckt npm run build
const base = process.env.BASE_PATH || './';

// Function to build docs
function buildDocs() {
  try {
    const buildScript = resolve(__dirname, 'scripts/build-docs.js');
    execSync(`node ${buildScript}`, { stdio: 'inherit', cwd: __dirname });
  } catch (error) {
    console.error('Error building docs:', error);
  }
}

// Plugin to build markdown docs and copy doc.html to dist/doc/index.html after build
const buildDocsPlugin = () => {
  let watcher = null;
  let rebuildTimer = null;
  
  // Debounced rebuild function
  const debouncedRebuild = (filename) => {
    if (rebuildTimer) {
      clearTimeout(rebuildTimer);
    }
    rebuildTimer = setTimeout(() => {
      console.log(`\n📝 Documentation file changed: ${filename}`);
      console.log('Rebuilding documentation...');
      buildDocs();
      rebuildTimer = null;
    }, 300); // 300ms debounce
  };
  
  return {
    name: 'build-docs',
    configureServer(server) {
      // Build docs on server start
      console.log('Building documentation from markdown...');
      buildDocs();
      
      // Watch doc directory for changes in development
      const docDir = resolve(__dirname, 'static/doc');
      if (existsSync(docDir)) {
        watcher = watch(docDir, { recursive: true }, (eventType, filename) => {
          // Only rebuild on change/rename events, ignore other events
          if (eventType === 'change' || eventType === 'rename') {
            if (filename && (filename.endsWith('.md') || !filename.includes('.'))) {
              debouncedRebuild(filename);
            }
          }
        });
        
        console.log('👀 Watching static/doc/ directory for changes...');
      }
      
      // Handle /doc route in development - serve from dist/doc
      const distDocDir = resolve(__dirname, 'dist/doc');
      // Handle /demo route in development - serve from static/demo
      const staticDemoDir = resolve(__dirname, 'static/demo');
      server.middlewares.use((req, res, next) => {
        if (req.url === '/doc' || req.url === '/doc/') {
          req.url = '/doc/index.html';
        }
        // Serve static files from dist/doc
        if (req.url?.startsWith('/doc/')) {
          const filePath = resolve(distDocDir, req.url.replace('/doc/', ''));
          if (existsSync(filePath)) {
            const content = readFileSync(filePath);
            const ext = filePath.split('.').pop();
            const contentType = ext === 'html' ? 'text/html' :
                                ext === 'css' ? 'text/css' :
                                ext === 'js' ? 'application/javascript' :
                                ext === 'json' ? 'application/json' :
                                ext === 'il' ? 'text/plain' :
                                'text/plain';
            res.setHeader('Content-Type', contentType);
            // Force download for .il files
            if (ext === 'il') {
              const fileName = filePath.split(/[/\\]/).pop();
              res.setHeader('Content-Disposition', `attachment; filename="${fileName}"`);
            }
            res.end(content);
            return;
          }
        }
        // Serve static files from static/demo
        if (req.url?.startsWith('/demo/')) {
          const filePath = resolve(staticDemoDir, req.url.replace('/demo/', ''));
          if (existsSync(filePath) && statSync(filePath).isFile()) {
            const content = readFileSync(filePath);
            const ext = filePath.split('.').pop();
            const contentType = ext === 'json' ? 'application/json' :
                                ext === 'zip' ? 'application/zip' :
                                'application/octet-stream';
            res.setHeader('Content-Type', contentType);
            res.end(content);
            return;
          }
        }
        next();
      });
    },
    buildEnd() {
      // Close watcher on build end
      if (watcher) {
        watcher.close();
        watcher = null;
      }
      if (rebuildTimer) {
        clearTimeout(rebuildTimer);
        rebuildTimer = null;
      }
    },
    writeBundle() {
      // Build markdown docs to static HTML
      console.log('Building documentation from markdown...');
      buildDocs();
      
      const distDir = resolve(__dirname, 'dist');
      const docDir = resolve(distDir, 'doc');
      const docHtml = resolve(distDir, 'doc.html');
      
      // If doc.html exists (from old build), remove it since we now use markdown-generated docs
      if (existsSync(docHtml)) {
        unlinkSync(docHtml);
      }
      
      // Copy demo files from static/demo to dist/demo
      const staticDemoDir = resolve(__dirname, 'static/demo');
      const distDemoDir = resolve(distDir, 'demo');
      if (existsSync(staticDemoDir)) {
        if (!existsSync(distDemoDir)) {
          mkdirSync(distDemoDir, { recursive: true });
        }
        const files = readdirSync(staticDemoDir);
        for (const file of files) {
          const srcPath = resolve(staticDemoDir, file);
          const dstPath = resolve(distDemoDir, file);
          if (statSync(srcPath).isFile()) {
            copyFileSync(srcPath, dstPath);
          }
        }
        console.log('📦 Copied demo files to dist/demo/');
      }
    },
  };
};

export default defineConfig({
  base,
  plugins: [react(), wasm(), buildDocsPlugin()],
  server: {
    port: 3000,
    // Handle /doc route by serving doc/index.html
    fs: {
      allow: ['..'],
    },
  },
  preview: {
    port: 3000,
  },
  build: {
    target: 'esnext',
    rollupOptions: {
      input: {
        main: resolve(__dirname, 'index.html'),
      },
      output: {
        manualChunks: undefined,
      },
    },
  },
  optimizeDeps: {
    exclude: ['drawckt-web'],
  },
});

