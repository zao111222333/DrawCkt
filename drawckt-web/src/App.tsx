import { useState, useEffect, useRef } from 'react';
import { wasmAPI, LayerStyles, initWasm } from './wasm';
import SettingsPanel from './components/SettingsPanel';
import SymbolsList from './components/SymbolsList';
import SchematicView from './components/SchematicView';
import SymbolEditor from './components/SymbolEditor';
import Logger from './components/Logger';
import './App.css';

function App() {
  const [menuVisible, setMenuVisible] = useState(true);
  const [menuWidth, setMenuWidth] = useState(240);
  const [symbolsWidth, setSymbolsWidth] = useState(400);
  const menuWidthRef = useRef(240);
  const symbolsWidthRef = useRef(400);
  const [symbols, setSymbols] = useState<Array<{ lib: string; cell: string }>>([]);
  const [schematicReady, setSchematicReady] = useState(false);
  const [layerStyles, setLayerStyles] = useState<LayerStyles | null>(null);
  const [schematicRefreshKey, setSchematicRefreshKey] = useState(0);
  const [symbolsRefreshKey, setSymbolsRefreshKey] = useState(0);
  const [editingSymbol, setEditingSymbol] = useState<{ lib: string; cell: string; content: string } | null>(null);
  const [editingSchematic, setEditingSchematic] = useState<{ content: string } | null>(null);

  // Handle browser navigation
  useEffect(() => {
    const handlePopState = () => {
      // Handle browser back/forward navigation if needed
    };
    const handlePathChange = () => {
      // Handle path changes if needed
    };
    window.addEventListener('popstate', handlePopState);
    window.addEventListener('pathchange', handlePathChange);
    return () => {
      window.removeEventListener('popstate', handlePopState);
      window.removeEventListener('pathchange', handlePathChange);
    };
  }, []);

  // Keep refs in sync with state
  useEffect(() => {
    menuWidthRef.current = menuWidth;
  }, [menuWidth]);

  useEffect(() => {
    symbolsWidthRef.current = symbolsWidth;
  }, [symbolsWidth]);

  useEffect(() => {
    // Initialize WASM and load default layer styles
    initWasm().then(() => {
      wasmAPI.getLayerStyles().then(setLayerStyles).catch(console.error);
    });

    // Helper function to route embedded files
    const routeEmbeddedFile = async (url: string): Promise<Response | null> => {
      // Extract pathname from full URL if needed
      let pathname = url;
      try {
        // Handle both absolute and relative URLs
        let urlToParse = url;
        if (!url.startsWith('http://') && !url.startsWith('https://') && !url.startsWith('//')) {
          urlToParse = new URL(url, window.location.origin).toString();
        }
        const urlObj = new URL(urlToParse);
        pathname = urlObj.pathname;
        // Remove double slashes
        pathname = pathname.replace(/\/+/g, '/');
      } catch (e) {
        // If URL parsing fails, use the original string and clean up
        pathname = url.replace(/\/+/g, '/');
      }
      
      // Check if it's an embedded path
      // Support both /embedded/ and /path/to/drawckt/embedded/
      const embeddedMatch = pathname.match(/(\/embedded\/.*)$/);
      if (embeddedMatch) {
        const embeddedPath = embeddedMatch[1]; // This will be like /embedded/... or /path/to/drawckt/embedded/...
        
        try {
          const content = await wasmAPI.routeEmbedded(embeddedPath);
          return new Response(content, {
            status: 200,
            headers: { 'Content-Type': 'application/xml' },
          });
        } catch (error) {
          console.error('Failed to route embedded file:', embeddedPath, error);
          return new Response('Not found', { status: 404 });
        }
      }
      
      return null;
    };

    // Override fetch to route embedded files through WASM (global interceptor)
    const originalFetch = window.fetch;
    window.fetch = async (input: RequestInfo | URL, init?: RequestInit) => {
      let url: string;
      if (typeof input === 'string') {
        url = input;
      } else if (input instanceof URL) {
        url = input.toString();
      } else {
        url = input.url;
      }
      
      const routed = await routeEmbeddedFile(url);
      if (routed) {
        return routed;
      }
      
      return originalFetch(input, init);
    };

    // Override XMLHttpRequest to route embedded files (draw.io viewer uses this)
    const originalXHROpen = XMLHttpRequest.prototype.open;
    const originalXHRSend = XMLHttpRequest.prototype.send;
    
    XMLHttpRequest.prototype.open = function(
      method: string,
      url: string | URL,
      async?: boolean,
      username?: string | null,
      password?: string | null
    ) {
      const urlString = typeof url === 'string' ? url : url.toString();
      
      // Extract pathname
      let pathname = urlString;
      try {
        // Handle both absolute and relative URLs
        let urlToParse = urlString;
        if (!urlString.startsWith('http://') && !urlString.startsWith('https://') && !urlString.startsWith('//')) {
          urlToParse = new URL(urlString, window.location.origin).toString();
        }
        const urlObj = new URL(urlToParse);
        pathname = urlObj.pathname;
        // Remove double slashes
        pathname = pathname.replace(/\/+/g, '/');
      } catch (e) {
        // If URL parsing fails, use the original string and clean up
        pathname = urlString.replace(/\/+/g, '/');
      }
      
      // Check if it's an embedded path (supports subfolder deployment)
      const embeddedMatch = pathname.match(/(\/embedded\/.*)$/);
      if (embeddedMatch) {
        const embeddedPath = embeddedMatch[1];
        // Store the pathname for later routing
        (this as any)._routedPathname = embeddedPath;
        // Call original open with a dummy URL
        return originalXHROpen.call(this, method, 'about:blank', async ?? true, username, password);
      }
      
      return originalXHROpen.call(this, method, url, async ?? true, username, password);
    };
    
    XMLHttpRequest.prototype.send = function(body?: Document | XMLHttpRequestBodyInit | null) {
      const routedPathname = (this as any)._routedPathname;
      
      if (routedPathname) {
        console.debug('Intercepting XHR request for:', routedPathname);
        const xhr = this;
        
        // Route through WASM
        routeEmbeddedFile(routedPathname).then((response) => {
          if (response) {
            return response.text();
          }
          throw new Error('No response');
        }).then((text: string) => {
          // Use Object.defineProperty to set read-only properties
          Object.defineProperty(xhr, 'responseText', {
            value: text,
            writable: false,
            configurable: true,
          });
          Object.defineProperty(xhr, 'response', {
            value: text,
            writable: false,
            configurable: true,
          });
          Object.defineProperty(xhr, 'status', {
            value: 200,
            writable: false,
            configurable: true,
          });
          Object.defineProperty(xhr, 'statusText', {
            value: 'OK',
            writable: false,
            configurable: true,
          });
          Object.defineProperty(xhr, 'readyState', {
            value: 4,
            writable: false,
            configurable: true,
          });
          
          // Trigger events in the correct order
          if (xhr.onreadystatechange) {
            xhr.onreadystatechange(new Event('readystatechange') as any);
          }
          if (xhr.onload) {
            xhr.onload(new Event('load') as any);
          }
        }).catch((error: any) => {
          console.error('XHR routing failed:', error);
          Object.defineProperty(xhr, 'status', {
            value: 404,
            writable: false,
            configurable: true,
          });
          Object.defineProperty(xhr, 'statusText', {
            value: 'Not Found',
            writable: false,
            configurable: true,
          });
          Object.defineProperty(xhr, 'readyState', {
            value: 4,
            writable: false,
            configurable: true,
          });
          
          if (xhr.onreadystatechange) {
            xhr.onreadystatechange(new Event('readystatechange') as any);
          }
          if (xhr.onerror) {
            xhr.onerror(new Event('error') as any);
          }
        });
        return;
      }
      
      return originalXHRSend.call(this, body);
    };

    return () => {
      window.fetch = originalFetch;
      XMLHttpRequest.prototype.open = originalXHROpen;
      XMLHttpRequest.prototype.send = originalXHRSend;
    };
  }, []);

  const handleSchematicUpload = async (jsonContent: string, filename?: string) => {
    // Clear old symbols and schematic first to show empty-message state
    // Layer styles are preserved (not cleared)
    setSymbols([]);
    setSchematicReady(false);
    
    try {
      const result = await wasmAPI.processSchematicJson(jsonContent, filename);
      const allSymbols = await wasmAPI.getAllSymbols();
      setSymbols(allSymbols);
      setSchematicReady(result.schematic_rendered);
      // Force schematic view to re-render with new content
      if (result.schematic_rendered) {
        setSchematicRefreshKey(prev => prev + 1);
        // Force symbols list to re-render as well
        setSymbolsRefreshKey(prev => prev + 1);
      }
      return result;
    } catch (error) {
      console.error('Failed to process schematic:', error);
      // Keep empty state on error (already cleared above)
      throw error;
    }
  };

  const handleSchematicClear = () => {
    // Clear symbols and schematic to show empty-message state
    // Layer styles are preserved (not cleared)
    setSymbols([]);
    setSchematicReady(false);
  };

  const handleExport = async () => {
    try {
      const result = await wasmAPI.exportAllFiles();
      
      if (!result.data) {
        throw new Error('No data received from export');
      }
      
      // Clean base64 string (remove whitespace, newlines, and other invalid characters)
      let base64Data = result.data.replace(/\s/g, '');
      
      // Validate base64 string
      if (!/^[A-Za-z0-9+/]*={0,2}$/.test(base64Data)) {
        throw new Error('Invalid base64 encoding');
      }
      
      // Decode base64 to binary
      let binaryString: string;
      try {
        binaryString = atob(base64Data);
      } catch (e) {
        throw new Error(`Failed to decode base64: ${e}`);
      }
      
      const bytes = new Uint8Array(binaryString.length);
      for (let i = 0; i < binaryString.length; i++) {
        bytes[i] = binaryString.charCodeAt(i);
      }
      
      // Create download link
      const blob = new Blob([bytes], {
        type: 'application/zip'
      });
      const url = URL.createObjectURL(blob);
      const a = document.createElement('a');
      a.href = url;
      a.download = result.filename;
      document.body.appendChild(a);
      a.click();
      document.body.removeChild(a);
      URL.revokeObjectURL(url);
    } catch (error) {
      console.error('Failed to export files:', error);
      alert(`Failed to export files: ${error}`);
    }
  };

  const handleLayerStylesUpdate = async (styles: LayerStyles) => {
    try {
      const result = await wasmAPI.updateLayerStyles(JSON.stringify(styles));
      setLayerStyles(styles);
      // Re-process schematic if it exists
      if (schematicReady) {
        // If only sch_visible changed, don't refresh symbols list
        if (!result.only_sch_visible_changed) {
          const allSymbols = await wasmAPI.getAllSymbols();
          setSymbols(allSymbols);
          // Force symbols list to re-render by updating refresh key
          setSymbolsRefreshKey(prev => prev + 1);
        }
        // Always force schematic view to re-render by updating refresh key
        setSchematicRefreshKey(prev => prev + 1);
      }
    } catch (error) {
      console.error('Failed to update layer styles:', error);
    }
  };

  return (
    <div className="app">
      <Logger />
      <button
        className={menuVisible ? "toggle-menu-btn-top" : "toggle-menu-btn-floating"}
        onClick={() => setMenuVisible(!menuVisible)}
        title={menuVisible ? 'Hide menu' : 'Show menu'}
      >
        {menuVisible ? '◀' : '▶'}
      </button>
      <div className="app-layout">
        {menuVisible && (
          <>
            <div className="panel settings-panel" style={{ width: `${menuWidth}px` }}>
            <div 
              className="panel-header settings-panel-header"
              onClick={() => setMenuVisible(!menuVisible)}
              style={{ cursor: 'pointer' }}
              title={menuVisible ? 'Click to hide settings panel' : 'Click to show settings panel'}
            >
              <h2>Toolbar</h2>
            </div>
              <SettingsPanel
                onSchematicUpload={handleSchematicUpload}
                onSchematicClear={handleSchematicClear}
                layerStyles={layerStyles}
                onLayerStylesUpdate={handleLayerStylesUpdate}
                canExport={schematicReady}
                onExport={handleExport}
              />
            </div>
            <div
              className="resize-handle"
              onMouseDown={(e) => {
                e.preventDefault();
                e.stopPropagation();
                const startX = e.clientX;
                const startWidth = menuWidthRef.current;
                const onMouseMove = (e: MouseEvent) => {
                  e.preventDefault();
                  e.stopPropagation();
                  const deltaX = e.clientX - startX;
                  const newWidth = Math.max(240, Math.min(600, startWidth + deltaX));
                  setMenuWidth(newWidth);
                };
                const onMouseUp = (e: MouseEvent) => {
                  e.preventDefault();
                  e.stopPropagation();
                  document.removeEventListener('mousemove', onMouseMove);
                  document.removeEventListener('mouseup', onMouseUp);
                  document.body.style.cursor = '';
                  document.body.style.userSelect = '';
                };
                document.addEventListener('mousemove', onMouseMove);
                document.addEventListener('mouseup', onMouseUp);
                document.body.style.cursor = 'col-resize';
                document.body.style.userSelect = 'none';
              }}
            />
          </>
        )}
        <div className="panel symbols-panel" style={{ width: `${symbolsWidth}px` }}>
          <div 
            className="panel-header symbols-panel-header"
            onClick={() => setMenuVisible(!menuVisible)}
            style={{ cursor: 'pointer' }}
            title={menuVisible ? 'Click to hide settings panel' : 'Click to show settings panel'}
          >
            <h2>Symbols</h2>
          </div>
          <SymbolsList 
            symbols={symbols} 
            refreshKey={symbolsRefreshKey}
            onEditSymbol={async (lib: string, cell: string) => {
              try {
                const content = await wasmAPI.getSymbolContent(lib, cell);
                setEditingSymbol({ lib, cell, content });
              } catch (error) {
                console.error('Failed to get symbol content:', error);
                alert(`Failed to load symbol content: ${error}`);
              }
            }}
            onSymbolUpdated={async (lib?: string, cell?: string) => {
              // If lib and cell are provided, only refresh that symbol and schematic
              // Otherwise, refresh all symbols and schematic
              if (lib && cell) {
                // Only refresh schematic (the specific symbol will be refreshed by SymbolsList)
                setSchematicRefreshKey(prev => prev + 1);
              } else {
                // Refresh all symbols and schematic
                const allSymbols = await wasmAPI.getAllSymbols();
                setSymbols(allSymbols);
                setSchematicRefreshKey(prev => prev + 1);
                setSymbolsRefreshKey(prev => prev + 1);
              }
            }}
          />
        </div>
        <div
          className="resize-handle"
          onMouseDown={(e) => {
            e.preventDefault();
            e.stopPropagation();
            const startX = e.clientX;
            const startWidth = symbolsWidthRef.current;
            const onMouseMove = (e: MouseEvent) => {
              e.preventDefault();
              e.stopPropagation();
              const deltaX = e.clientX - startX;
              const newWidth = Math.max(300, Math.min(800, startWidth + deltaX));
              setSymbolsWidth(newWidth);
            };
            const onMouseUp = (e: MouseEvent) => {
              e.preventDefault();
              e.stopPropagation();
              document.removeEventListener('mousemove', onMouseMove);
              document.removeEventListener('mouseup', onMouseUp);
              document.body.style.cursor = '';
              document.body.style.userSelect = '';
            };
            document.addEventListener('mousemove', onMouseMove);
            document.addEventListener('mouseup', onMouseUp);
            document.body.style.cursor = 'col-resize';
            document.body.style.userSelect = 'none';
          }}
        />
        <div className="panel schematic-panel">
          <div 
            className="panel-header schematic-panel-header"
            onClick={() => setMenuVisible(!menuVisible)}
            style={{ cursor: 'pointer' }}
            title={menuVisible ? 'Click to hide settings panel' : 'Click to show settings panel'}
          >
            <h2>Schematic</h2>
          </div>
          <SchematicView 
            ready={schematicReady} 
            refreshKey={schematicRefreshKey}
            onEditSchematic={async () => {
              try {
                const content = await wasmAPI.getSchematicContent();
                setEditingSchematic({ content });
              } catch (error) {
                console.error('Failed to get schematic content:', error);
                alert(`Failed to load schematic content: ${error}`);
              }
            }}
            onSchematicUpdated={async () => {
              // Refresh schematic view
              setSchematicRefreshKey(prev => prev + 1);
            }}
          />
        </div>
      </div>
      {editingSymbol && (
        <SymbolEditor
          lib={editingSymbol.lib}
          cell={editingSymbol.cell}
          content={editingSymbol.content}
          onSave={async (lib: string, cell: string, newContent: string) => {
            try {
              await wasmAPI.updateSymbolContent(lib, cell, newContent);
              setEditingSymbol(null);
              // Re-fetch symbols and refresh both symbols and schematic
              const allSymbols = await wasmAPI.getAllSymbols();
              setSymbols(allSymbols);
              // Force refresh to update history state - increment refreshKey to trigger history reload
              // Add a small delay to ensure WASM state is fully updated
              setTimeout(() => {
                setSymbolsRefreshKey(prev => prev + 1);
                setSchematicRefreshKey(prev => prev + 1);
              }, 100);
            } catch (error) {
              console.error('Failed to update symbol content:', error);
              alert(`Failed to save symbol: ${error}`);
            }
          }}
          onClose={() => setEditingSymbol(null)}
        />
      )}
      {editingSchematic && (
        <SymbolEditor
          lib=""
          cell="Schematic"
          content={editingSchematic.content}
          onSave={async (_lib: string, _cell: string, newContent: string) => {
            try {
              await wasmAPI.updateSchematicContent(newContent);
              setEditingSchematic(null);
              // Refresh schematic view
              setTimeout(() => {
                setSchematicRefreshKey(prev => prev + 1);
              }, 100);
            } catch (error) {
              console.error('Failed to update schematic content:', error);
              alert(`Failed to save schematic: ${error}`);
            }
          }}
          onClose={() => setEditingSchematic(null)}
        />
      )}
    </div>
  );
}

export default App;

