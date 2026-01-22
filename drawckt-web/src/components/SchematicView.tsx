import React, { useEffect, useRef, useState, useCallback } from 'react';
import { getBasePath } from '../utils/path';
import { wasmAPI } from '../wasm';
import './SchematicView.css';

interface SchematicViewProps {
  ready: boolean;
  refreshKey?: number; // Key to force re-render
  onEditSchematic?: () => void; // Callback when edit button is clicked
  onSchematicUpdated?: () => void; // Callback when schematic is updated (for undo/redo)
  isDarkMode?: boolean; // Dark mode state
}

const SchematicView: React.FC<SchematicViewProps> = ({ ready, refreshKey, onEditSchematic, onSchematicUpdated, isDarkMode = false }) => {
  const containerRef = useRef<HTMLDivElement>(null);
  const scrollContainerRef = useRef<HTMLDivElement>(null);
  const viewerInstanceRef = useRef<any>(null);
  const [panStart, setPanStart] = useState<{ x: number; y: number } | null>(null);
  const [isDragging, setIsDragging] = useState(false);
  const [schematicHistory, setSchematicHistory] = useState<{ canUndo: boolean; canRedo: boolean }>({ canUndo: false, canRedo: false });

  useEffect(() => {
    if (!ready) {
      return;
    }

    // Clean up previous viewer instance if exists
    const cleanup = () => {
      if (containerRef.current) {
        try {
          // Remove all existing mxgraph elements
          const viewerElements = containerRef.current.querySelectorAll('.mxgraph');
          viewerElements.forEach((element) => {
            element.remove();
          });
        } catch (e) {
          console.warn('Error cleaning up viewer:', e);
        }
        viewerInstanceRef.current = null;
      }
      
      // Also clean up any toolbar wrapper that was moved to scroll container
      if (scrollContainerRef.current) {
        try {
          const toolbarWrappers = scrollContainerRef.current.querySelectorAll('.schematic-toolbar-wrapper');
          toolbarWrappers.forEach((wrapper) => {
            wrapper.remove();
          });
        } catch (e) {
          console.warn('Error cleaning up toolbar:', e);
        }
      }
    };

    // Load draw.io viewer script if not already loaded
    const loadViewer = () => {
      if ((window as any).GraphViewer) {
        setTimeout(initializeViewer, 50);
        return;
      }

      const existingScript = document.querySelector('script[src*="viewer-static.min.js"]');
      if (!existingScript) {
        const script = document.createElement('script');
        script.src = 'https://viewer.diagrams.net/js/viewer-static.min.js';
        script.async = true;
        script.onload = () => {
          // Wait a bit for GraphViewer to be available
          setTimeout(initializeViewer, 200);
        };
        script.onerror = (e) => {
          console.error('Failed to load draw.io viewer script:', e);
        };
        document.body.appendChild(script);
      } else {
        // Script already loading, wait for it
        const checkInterval = setInterval(() => {
          if ((window as any).GraphViewer) {
            clearInterval(checkInterval);
            setTimeout(initializeViewer, 100);
          }
        }, 100);
        return () => clearInterval(checkInterval);
      }
    };

    const initializeViewer = () => {
      if (containerRef.current && scrollContainerRef.current && (window as any).GraphViewer) {
        // Clean up any existing viewer first
        cleanup();
        
        // Create new mxgraph div
        const viewerDiv = document.createElement('div');
        viewerDiv.className = 'mxgraph';
        viewerDiv.style.width = '100%';
        viewerDiv.style.height = '100%';
        viewerDiv.style.border = 'none';
        viewerDiv.style.margin = '0';
        viewerDiv.style.padding = '0';
        
        // Add timestamp to URL to force reload
        const url = `${getBasePath()}/embedded/schematic.drawio?t=${Date.now()}`;
        viewerDiv.setAttribute('data-mxgraph', JSON.stringify({
          highlight: '#0000ff',
          target: 'blank',
          nav: true,
          resize: true,
          toolbar: 'tags layers zoom',
          'toolbar-nohide': true,
          lightbox: false,
          url: url,
        }));
        
        containerRef.current.appendChild(viewerDiv);
        
        try {
          (window as any).GraphViewer.createViewerForElement(viewerDiv);
          viewerInstanceRef.current = viewerDiv;
          
          // Wait for toolbar to be created, then move it to scroll container
          const moveToolbarToScrollContainer = () => {
            if (!scrollContainerRef.current || !viewerDiv) return;
            
            // Find toolbar - it's typically a div with position absolute inside mxgraph
            // Look for the toolbar element (usually has specific styling)
            const findToolbar = () => {
              // Try multiple selectors to find the toolbar
              // Toolbar typically has position: absolute, z-index: 999, and display: flex
              const allDivs = viewerDiv.querySelectorAll('div');
              
              for (const div of Array.from(allDivs)) {
                const style = (div as HTMLElement).style;
                const computedStyle = window.getComputedStyle(div);
                const styleText = div.getAttribute('style') || '';
                
                // Check for toolbar characteristics:
                // 1. Has position absolute
                // 2. Has z-index 999 or higher
                // 3. Has display flex
                const hasAbsolutePosition = computedStyle.position === 'absolute' || style.position === 'absolute';
                const hasHighZIndex = parseInt(computedStyle.zIndex) >= 999 || parseInt(style.zIndex) >= 999 || styleText.includes('z-index: 999');
                const hasFlexDisplay = computedStyle.display === 'flex' || style.display === 'flex';
                
                if (hasAbsolutePosition && hasHighZIndex && hasFlexDisplay) {
                  return div as HTMLElement;
                }
              }
              
              return null;
            };
            
            let toolbar = findToolbar();
            let attempts = 0;
            const maxAttempts = 30;
            
            // Retry finding toolbar with increasing delays
            const retryFind = setInterval(() => {
              attempts++;
              if (!toolbar) {
                toolbar = findToolbar();
              }
              
              if (toolbar || attempts >= maxAttempts) {
                clearInterval(retryFind);
                
                if (toolbar && scrollContainerRef.current) {
                  // Create a wrapper container for the toolbar
                  const toolbarWrapper = document.createElement('div');
                  toolbarWrapper.className = 'schematic-toolbar-wrapper';
                  toolbarWrapper.style.position = 'absolute';
                  toolbarWrapper.style.top = '0';
                  toolbarWrapper.style.left = '0';
                  toolbarWrapper.style.right = '0';
                  toolbarWrapper.style.width = '100%';
                  toolbarWrapper.style.zIndex = '1000';
                  toolbarWrapper.style.pointerEvents = 'auto';
                  
                  // Move toolbar to wrapper and adjust its styles
                  toolbar.style.position = 'relative';
                  toolbar.style.top = 'auto';
                  toolbar.style.left = 'auto';
                  toolbar.style.right = 'auto';
                  toolbar.style.width = '100%';
                  
                  // Add class for styling
                  toolbar.classList.add('schematic-toolbar-fixed');
                  
                  // Set color-scheme based on dark mode
                  toolbar.style.colorScheme = isDarkMode ? 'dark' : 'light';
                  
                  // Create action buttons container - positioned absolutely to float over toolbar
                  const actionsContainer = document.createElement('div');
                  actionsContainer.className = 'schematic-actions';
                  actionsContainer.style.position = 'absolute';
                  actionsContainer.style.right = '8px';
                  actionsContainer.style.top = '50%';
                  actionsContainer.style.transform = 'translateY(-50%)';
                  actionsContainer.style.display = 'flex';
                  actionsContainer.style.gap = '4px';
                  actionsContainer.style.alignItems = 'center';
                  actionsContainer.style.zIndex = '1001';
                  actionsContainer.style.pointerEvents = 'auto';
                  
                  // Create undo button
                  const undoBtn = document.createElement('button');
                  undoBtn.className = `schematic-undo-btn ${schematicHistory.canUndo ? 'enabled' : 'disabled'}`;
                  undoBtn.textContent = 'Undo';
                  undoBtn.title = 'Undo';
                  undoBtn.disabled = !schematicHistory.canUndo;
                  undoBtn.onclick = async () => {
                    // Don't check closure value, check button state instead
                    if (undoBtn.disabled) {
                      return;
                    }
                    try {
                      await wasmAPI.undoSchematic();
                      await updateSchematicHistory();
                      if (onSchematicUpdated) {
                        onSchematicUpdated();
                      }
                    } catch (error) {
                      console.error('Failed to undo schematic:', error);
                    }
                  };
                  
                  // Create redo button
                  const redoBtn = document.createElement('button');
                  redoBtn.className = `schematic-redo-btn ${schematicHistory.canRedo ? 'enabled' : 'disabled'}`;
                  redoBtn.textContent = 'Redo';
                  redoBtn.title = 'Redo';
                  redoBtn.disabled = !schematicHistory.canRedo;
                  redoBtn.onclick = async () => {
                    // Don't check closure value, check button state instead
                    if (redoBtn.disabled) {
                      return;
                    }
                    try {
                      await wasmAPI.redoSchematic();
                      await updateSchematicHistory();
                      if (onSchematicUpdated) {
                        onSchematicUpdated();
                      }
                    } catch (error) {
                      console.error('Failed to redo schematic:', error);
                    }
                  };
                  
                  // Create edit button
                  const editBtn = document.createElement('button');
                  editBtn.className = 'schematic-edit-btn';
                  editBtn.textContent = 'Edit';
                  editBtn.title = 'Edit Schematic';
                  editBtn.onclick = () => {
                    if (onEditSchematic) {
                      onEditSchematic();
                    }
                  };
                  
                  actionsContainer.appendChild(undoBtn);
                  actionsContainer.appendChild(redoBtn);
                  actionsContainer.appendChild(editBtn);
                  
                  // Append toolbar to wrapper, then append actions (so it floats over toolbar)
                  toolbarWrapper.appendChild(toolbar);
                  toolbarWrapper.appendChild(actionsContainer);
                  
                  // Move wrapper to scroll container
                  scrollContainerRef.current.appendChild(toolbarWrapper);
                  
                  // Update history state
                  updateSchematicHistory();
                }
              }
            }, 100);
          };
          
          // Start trying to move toolbar after a short delay
          setTimeout(moveToolbarToScrollContainer, 300);
          
        } catch (e) {
          console.error('Failed to initialize schematic viewer:', e);
        }
      } else {
        console.warn('Container or GraphViewer not available', {
          container: !!containerRef.current,
          scrollContainer: !!scrollContainerRef.current,
          graphViewer: !!(window as any).GraphViewer,
        });
      }
    };

    // Clean up on unmount or when refreshKey changes
    cleanup();
    
    loadViewer();
    
    return cleanup;
  }, [ready, refreshKey, isDarkMode]);


  // Handle mouse drag to pan
  const handleMouseDown = useCallback((e: React.MouseEvent) => {
    if (e.button !== 0 || !containerRef.current) return; // Only left button
    if ((e.target as HTMLElement).closest('a, button, input, .schematic-toolbar-fixed')) return; // Don't drag if clicking on interactive elements or toolbar
    
    setIsDragging(true);
    setPanStart({
      x: e.clientX + containerRef.current.scrollLeft,
      y: e.clientY + containerRef.current.scrollTop,
    });
    e.preventDefault();
  }, []);

  const handleMouseMove = useCallback((e: MouseEvent) => {
    if (!isDragging || !panStart || !containerRef.current) return;
    
    containerRef.current.scrollLeft = panStart.x - e.clientX;
    containerRef.current.scrollTop = panStart.y - e.clientY;
  }, [isDragging, panStart]);

  const handleMouseUp = useCallback(() => {
    setIsDragging(false);
    setPanStart(null);
  }, []);

  // Handle touch gestures for pan
  const handleTouchStart = useCallback((e: React.TouchEvent) => {
    if (!containerRef.current) return;
    
    if (e.touches.length === 1) {
      // Single touch - start pan
      const touch = e.touches[0];
      setIsDragging(true);
      setPanStart({
        x: touch.clientX + containerRef.current.scrollLeft,
        y: touch.clientY + containerRef.current.scrollTop,
      });
    }
  }, []);

  const handleTouchMove = useCallback((e: React.TouchEvent) => {
    if (!containerRef.current) return;
    
    if (e.touches.length === 1 && isDragging && panStart) {
      // Single touch - pan
      e.preventDefault();
      const touch = e.touches[0];
      containerRef.current.scrollLeft = panStart.x - touch.clientX;
      containerRef.current.scrollTop = panStart.y - touch.clientY;
    }
  }, [isDragging, panStart]);

  const handleTouchEnd = useCallback(() => {
    setIsDragging(false);
    setPanStart(null);
  }, []);

  // Attach global mouse event listeners for dragging
  useEffect(() => {
    if (isDragging) {
      document.addEventListener('mousemove', handleMouseMove);
      document.addEventListener('mouseup', handleMouseUp);
      
      return () => {
        document.removeEventListener('mousemove', handleMouseMove);
        document.removeEventListener('mouseup', handleMouseUp);
      };
    }
  }, [isDragging, handleMouseMove, handleMouseUp]);

  // Update schematic history state
  const updateSchematicHistory = async () => {
    try {
      const info = await wasmAPI.getSchematicInfo();
      setSchematicHistory({
        canUndo: info.can_undo,
        canRedo: info.can_redo,
      });
      
      // Update button states if toolbar exists
      if (scrollContainerRef.current) {
        const toolbarWrapper = scrollContainerRef.current.querySelector('.schematic-toolbar-wrapper');
        if (toolbarWrapper) {
          const undoBtn = toolbarWrapper.querySelector('.schematic-undo-btn') as HTMLButtonElement;
          const redoBtn = toolbarWrapper.querySelector('.schematic-redo-btn') as HTMLButtonElement;
          
          if (undoBtn) {
            undoBtn.disabled = !info.can_undo;
            undoBtn.className = `schematic-undo-btn ${info.can_undo ? 'enabled' : 'disabled'}`;
          }
          if (redoBtn) {
            redoBtn.disabled = !info.can_redo;
            redoBtn.className = `schematic-redo-btn ${info.can_redo ? 'enabled' : 'disabled'}`;
          }
        }
      }
    } catch (error) {
      console.error('Failed to get schematic info:', error);
    }
  };

  // Update history when refreshKey changes
  useEffect(() => {
    if (ready) {
      updateSchematicHistory();
    }
  }, [ready, refreshKey]);

  // Update toolbar color-scheme when dark mode changes
  useEffect(() => {
    const fixedToolbars = document.querySelectorAll('.schematic-toolbar-fixed');
    fixedToolbars.forEach((toolbar) => {
      (toolbar as HTMLElement).style.colorScheme = isDarkMode ? 'dark' : 'light';
    });
  }, [isDarkMode]);

  // Remove schematic-toolbar-fixed when schematic-view becomes empty
  useEffect(() => {
    if (!ready) {
      // When ready is false, schematic-view has 'empty' class
      // Remove any existing toolbar wrappers and fixed toolbars
      const toolbarWrappers = document.querySelectorAll('.schematic-toolbar-wrapper');
      toolbarWrappers.forEach((wrapper) => {
        wrapper.remove();
      });
      
      // Also remove schematic-toolbar-fixed class from any remaining toolbars
      const fixedToolbars = document.querySelectorAll('.schematic-toolbar-fixed');
      fixedToolbars.forEach((toolbar) => {
        toolbar.classList.remove('schematic-toolbar-fixed');
      });
    }
  }, [ready]);


  if (!ready) {
    return (
      <div className="schematic-view empty">
        <div className="empty-message">
          Upload a schematic.json file to see the schematic
        </div>
      </div>
    );
  }

  // mxgraph div will be created in useEffect
  return (
    <div 
      className="schematic-view-scroll-container"
      ref={scrollContainerRef}
    >
      <div 
        className="schematic-view" 
        ref={containerRef}
        onMouseDown={handleMouseDown}
        onTouchStart={handleTouchStart}
        onTouchMove={handleTouchMove}
        onTouchEnd={handleTouchEnd}
        style={{ cursor: isDragging ? 'grabbing' : 'grab' }}
      >
        {/* mxgraph div is created dynamically in useEffect */}
      </div>
    </div>
  );
};

export default SchematicView;

