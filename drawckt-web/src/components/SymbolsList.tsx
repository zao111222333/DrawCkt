import React, { useEffect, useRef, useState } from 'react';
import { getBasePath } from '../utils/path';
import { wasmAPI } from '../wasm';
import './SymbolsList.css';

interface SymbolsListProps {
  symbols: Array<{ lib: string; cell: string }>;
  refreshKey?: number; // Key to force re-render
  onEditSymbol?: (lib: string, cell: string) => void; // Callback when edit button is clicked
  onSymbolUpdated?: (lib?: string, cell?: string) => void; // Callback when symbol is updated (for undo/redo), with optional lib/cell to update only specific symbol
}

interface SymbolHistoryState {
  canUndo: boolean;
  canRedo: boolean;
}

const SymbolsList: React.FC<SymbolsListProps> = ({ symbols, refreshKey, onEditSymbol, onSymbolUpdated }) => {
  const containerRef = useRef<HTMLDivElement>(null);
  const viewerInstancesRef = useRef<Map<string, HTMLElement>>(new Map());
  const [symbolHistory, setSymbolHistory] = useState<Map<string, SymbolHistoryState>>(new Map());

  useEffect(() => {
    // Clean up previous viewer instances
    const cleanup = () => {
      if (containerRef.current) {
        try {
          // Remove all existing mxgraph elements
          const viewerElements = containerRef.current.querySelectorAll('.mxgraph');
          viewerElements.forEach((element) => {
            element.remove();
          });
        } catch (e) {
          console.warn('Error cleaning up viewers:', e);
        }
        viewerInstancesRef.current.clear();
      }
    };

    // Load draw.io viewer script if not already loaded
    const loadViewer = () => {
      if ((window as any).GraphViewer) {
        initializeViewers();
        return;
      }

      if (!document.querySelector('script[src*="viewer-static.min.js"]')) {
        const script = document.createElement('script');
        script.src = 'https://viewer.diagrams.net/js/viewer-static.min.js';
        script.async = true;
        script.onload = () => {
          // Wait a bit for GraphViewer to be available
          setTimeout(initializeViewers, 100);
        };
        document.body.appendChild(script);
      } else {
        // Script already loading, wait for it
        const checkInterval = setInterval(() => {
          if ((window as any).GraphViewer) {
            clearInterval(checkInterval);
            initializeViewers();
          }
        }, 100);
        return () => clearInterval(checkInterval);
      }
    };

    const initializeViewers = () => {
      // Wait for React to finish rendering DOM
      setTimeout(() => {
        if (containerRef.current && (window as any).GraphViewer) {
          // Clean up any existing viewers first
          cleanup();

          // Filter out invalid symbols
          const validSymbols = symbols.filter(
            (symbol) => symbol.lib && symbol.cell && symbol.lib !== 'undefined' && symbol.cell !== 'undefined'
          );

          // Create new mxgraph divs for each symbol
          validSymbols.forEach((symbol) => {
            const key = `${symbol.lib}/${symbol.cell}`;
            const symbolItem = containerRef.current?.querySelector(`[data-symbol-key="${key}"]`);
            
            if (symbolItem) {
              const wrapper = symbolItem.querySelector('.mxgraph-wrapper');
              if (wrapper) {
                // Create new mxgraph div
                const viewerDiv = document.createElement('div');
                viewerDiv.className = 'mxgraph';
                viewerDiv.style.width = '100%';
                viewerDiv.style.border = 'none';
                viewerDiv.style.margin = '0';
                viewerDiv.style.padding = '0';
                
                // Add timestamp to URL to force reload
                const url = `${getBasePath()}/embedded/symbols/${symbol.lib}/${symbol.cell}.drawio?t=${Date.now()}`;
                viewerDiv.setAttribute('data-mxgraph', JSON.stringify({
                  highlight: '#0000ff',
                  target: 'blank',
                  nav: true,
                  lightbox: false,
                  url: url,
                }));
                
                wrapper.appendChild(viewerDiv);
                
                // Add click handler to open editor (use capture phase to intercept before mxgraph)
                const handleViewerClick = (e: Event) => {
                  e.preventDefault();
                  e.stopPropagation();
                  e.stopImmediatePropagation();
                  if (onEditSymbol) {
                    onEditSymbol(symbol.lib, symbol.cell);
                  }
                };
                viewerDiv.addEventListener('click', handleViewerClick, true);
                viewerDiv.style.cursor = 'pointer';
                
                // Also add to wrapper to catch all clicks
                const handleWrapperClick = (e: Event) => {
                  e.preventDefault();
                  e.stopPropagation();
                  e.stopImmediatePropagation();
                  if (onEditSymbol) {
                    onEditSymbol(symbol.lib, symbol.cell);
                  }
                };
                wrapper.addEventListener('click', handleWrapperClick, true);
                
                try {
                  (window as any).GraphViewer.createViewerForElement(viewerDiv);
                  viewerInstancesRef.current.set(key, viewerDiv);
                  
                  // After viewer is created, ensure click handler still works
                  setTimeout(() => {
                    const svg = viewerDiv.querySelector('svg');
                    const canvas = viewerDiv.querySelector('canvas');
                    if (svg) {
                      svg.addEventListener('click', handleViewerClick, true);
                      svg.style.cursor = 'pointer';
                    }
                    if (canvas) {
                      canvas.addEventListener('click', handleViewerClick, true);
                      canvas.style.cursor = 'pointer';
                    }
                  }, 200);
                } catch (e) {
                  console.warn('Failed to initialize viewer:', e);
                }
              }
            }
          });
        }
      }, 0);
    };

    loadViewer();
    
    return cleanup;
  }, [symbols, refreshKey]);

  // Function to refresh a specific symbol viewer
  const refreshSymbolViewer = React.useCallback((lib: string, cell: string) => {
    if (containerRef.current && (window as any).GraphViewer) {
      const key = `${lib}/${cell}`;
      const symbolItem = containerRef.current?.querySelector(`[data-symbol-key="${key}"]`);
      
      if (symbolItem) {
        const wrapper = symbolItem.querySelector('.mxgraph-wrapper');
        if (wrapper) {
          // Remove existing viewer
          const existingViewer = viewerInstancesRef.current.get(key);
          if (existingViewer && existingViewer.parentNode) {
            existingViewer.parentNode.removeChild(existingViewer);
            viewerInstancesRef.current.delete(key);
          }

          // Create new mxgraph div
          const viewerDiv = document.createElement('div');
          viewerDiv.className = 'mxgraph';
          viewerDiv.style.width = '100%';
          viewerDiv.style.border = 'none';
          viewerDiv.style.margin = '0';
          viewerDiv.style.padding = '0';
          
          // Add timestamp to URL to force reload
          const url = `${getBasePath()}/embedded/symbols/${lib}/${cell}.drawio?t=${Date.now()}`;
          viewerDiv.setAttribute('data-mxgraph', JSON.stringify({
            highlight: '#0000ff',
            target: 'blank',
            nav: true,
            edit: '_blank',
            url: url,
          }));
          
            wrapper.appendChild(viewerDiv);
            
            // Add click handler to open editor (use capture phase to intercept before mxgraph)
            const handleViewerClick = (e: Event) => {
              e.preventDefault();
              e.stopPropagation();
              e.stopImmediatePropagation();
              if (onEditSymbol) {
                onEditSymbol(lib, cell);
              }
            };
            viewerDiv.addEventListener('click', handleViewerClick, true);
            viewerDiv.style.cursor = 'pointer';
            
            // Also add to wrapper to catch all clicks
            const handleWrapperClick = (e: Event) => {
              e.preventDefault();
              e.stopPropagation();
              e.stopImmediatePropagation();
              if (onEditSymbol) {
                onEditSymbol(lib, cell);
              }
            };
            wrapper.addEventListener('click', handleWrapperClick, true);
            
            try {
              (window as any).GraphViewer.createViewerForElement(viewerDiv);
              viewerInstancesRef.current.set(key, viewerDiv);
              
              // After viewer is created, ensure click handler still works
              setTimeout(() => {
                const svg = viewerDiv.querySelector('svg');
                const canvas = viewerDiv.querySelector('canvas');
                if (svg) {
                  svg.addEventListener('click', handleViewerClick, true);
                  svg.style.cursor = 'pointer';
                }
                if (canvas) {
                  canvas.addEventListener('click', handleViewerClick, true);
                  canvas.style.cursor = 'pointer';
                }
              }, 200);
            } catch (e) {
              console.warn('Failed to initialize viewer:', e);
            }
        }
      }
    }
  }, []);

  // Load symbol history info
  useEffect(() => {
    const loadHistoryInfo = async () => {
      const newHistory = new Map<string, SymbolHistoryState>();
      for (const symbol of symbols) {
        if (symbol.lib && symbol.cell) {
          try {
            const info = await wasmAPI.getSymbolInfo(symbol.lib, symbol.cell);
            newHistory.set(`${symbol.lib}/${symbol.cell}`, {
              canUndo: info.can_undo,
              canRedo: info.can_redo,
            });
          } catch (error) {
            console.warn(`Failed to get history info for ${symbol.lib}/${symbol.cell}:`, error);
            newHistory.set(`${symbol.lib}/${symbol.cell}`, {
              canUndo: false,
              canRedo: false,
            });
          }
        }
      }
      setSymbolHistory(newHistory);
    };
    
    if (symbols.length > 0) {
      loadHistoryInfo();
    }
  }, [symbols, refreshKey]);

  // Expose updateSymbolHistory function to parent via ref or callback
  // We'll use a different approach - update history when refreshKey changes for a specific symbol

  const handleEditClick = (lib: string, cell: string) => {
    if (onEditSymbol) {
      onEditSymbol(lib, cell);
    }
  };

  const handleUndo = async (lib: string, cell: string) => {
    try {
      await wasmAPI.undoSymbol(lib, cell);
      // Update history state
      const key = `${lib}/${cell}`;
      const info = await wasmAPI.getSymbolInfo(lib, cell);
      setSymbolHistory(prev => {
        const newMap = new Map(prev);
        newMap.set(key, {
          canUndo: info.can_undo,
          canRedo: info.can_redo,
        });
        return newMap;
      });
      // Refresh only this symbol viewer
      refreshSymbolViewer(lib, cell);
      // Refresh schematic
      if (onSymbolUpdated) {
        onSymbolUpdated(lib, cell);
      }
    } catch (error) {
      console.error('Failed to undo symbol:', error);
    }
  };

  const handleRedo = async (lib: string, cell: string) => {
    try {
      await wasmAPI.redoSymbol(lib, cell);
      // Update history state
      const key = `${lib}/${cell}`;
      const info = await wasmAPI.getSymbolInfo(lib, cell);
      setSymbolHistory(prev => {
        const newMap = new Map(prev);
        newMap.set(key, {
          canUndo: info.can_undo,
          canRedo: info.can_redo,
        });
        return newMap;
      });
      // Refresh only this symbol viewer
      refreshSymbolViewer(lib, cell);
      // Refresh schematic
      if (onSymbolUpdated) {
        onSymbolUpdated(lib, cell);
      }
    } catch (error) {
      console.error('Failed to redo symbol:', error);
    }
  };

  // Filter out invalid symbols (with undefined lib or cell)
  const validSymbols = symbols.filter(
    (symbol) => symbol.lib && symbol.cell && symbol.lib !== 'undefined' && symbol.cell !== 'undefined'
  );

  if (validSymbols.length === 0) {
    return (
      <div className="symbols-list empty">
        <div className="empty-message">
          {symbols.length === 0
            ? 'Upload a schematic.json file to see symbols'
            : 'No valid symbols found'}
        </div>
      </div>
    );
  }

  return (
    <div className="symbols-list" ref={containerRef}>
      {validSymbols.map((symbol) => {
        const key = `${symbol.lib}/${symbol.cell}`;
        return (
          <div key={key} data-symbol-key={key} className="symbol-item">
            <div className="symbol-header">
              <span className="symbol-name">{symbol.lib}/{symbol.cell}</span>
              <div className="symbol-actions">
                <button
                  className={`symbol-undo-btn ${symbolHistory.get(key)?.canUndo ? 'enabled' : 'disabled'}`}
                  onClick={() => symbolHistory.get(key)?.canUndo && handleUndo(symbol.lib, symbol.cell)}
                  title="Undo"
                  disabled={!symbolHistory.get(key)?.canUndo}
                >
                  Undo
                </button>
                <button
                  className={`symbol-redo-btn ${symbolHistory.get(key)?.canRedo ? 'enabled' : 'disabled'}`}
                  onClick={() => symbolHistory.get(key)?.canRedo && handleRedo(symbol.lib, symbol.cell)}
                  title="Redo"
                  disabled={!symbolHistory.get(key)?.canRedo}
                >
                  Redo
                </button>
                <button
                  className="symbol-edit-btn"
                  onClick={() => handleEditClick(symbol.lib, symbol.cell)}
                  title="Edit Symbol"
                >
                  Edit
                </button>
              </div>
            </div>
            <div className="mxgraph-wrapper">
              {/* mxgraph div will be created dynamically in useEffect */}
            </div>
          </div>
        );
      })}
      {/* Spacer div to allow scrolling to bottom */}
      <div style={{ height: '20px', flexShrink: 0 }} />
    </div>
  );
};

export default SymbolsList;

