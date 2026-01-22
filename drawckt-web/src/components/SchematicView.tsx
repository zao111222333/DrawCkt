import React, { useEffect, useRef, useState, useCallback } from 'react';
import { getBasePath } from '../utils/path';
import './SchematicView.css';

interface SchematicViewProps {
  ready: boolean;
  refreshKey?: number; // Key to force re-render
}

const SchematicView: React.FC<SchematicViewProps> = ({ ready, refreshKey }) => {
  const containerRef = useRef<HTMLDivElement>(null);
  const scrollContainerRef = useRef<HTMLDivElement>(null);
  const viewerInstanceRef = useRef<any>(null);
  const [panStart, setPanStart] = useState<{ x: number; y: number } | null>(null);
  const [isDragging, setIsDragging] = useState(false);

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
  }, [ready, refreshKey]);


  // Handle mouse drag to pan
  const handleMouseDown = useCallback((e: React.MouseEvent) => {
    if (e.button !== 0 || !scrollContainerRef.current) return; // Only left button
    if ((e.target as HTMLElement).closest('a, button, input')) return; // Don't drag if clicking on interactive elements
    
    setIsDragging(true);
    setPanStart({
      x: e.clientX + scrollContainerRef.current.scrollLeft,
      y: e.clientY + scrollContainerRef.current.scrollTop,
    });
    e.preventDefault();
  }, []);

  const handleMouseMove = useCallback((e: MouseEvent) => {
    if (!isDragging || !panStart || !scrollContainerRef.current) return;
    
    scrollContainerRef.current.scrollLeft = panStart.x - e.clientX;
    scrollContainerRef.current.scrollTop = panStart.y - e.clientY;
  }, [isDragging, panStart]);

  const handleMouseUp = useCallback(() => {
    setIsDragging(false);
    setPanStart(null);
  }, []);

  // Handle touch gestures for pan
  const handleTouchStart = useCallback((e: React.TouchEvent) => {
    if (!scrollContainerRef.current) return;
    
    if (e.touches.length === 1) {
      // Single touch - start pan
      const touch = e.touches[0];
      setIsDragging(true);
      setPanStart({
        x: touch.clientX + scrollContainerRef.current.scrollLeft,
        y: touch.clientY + scrollContainerRef.current.scrollTop,
      });
    }
  }, []);

  const handleTouchMove = useCallback((e: React.TouchEvent) => {
    if (!scrollContainerRef.current) return;
    
    if (e.touches.length === 1 && isDragging && panStart) {
      // Single touch - pan
      e.preventDefault();
      const touch = e.touches[0];
      scrollContainerRef.current.scrollLeft = panStart.x - touch.clientX;
      scrollContainerRef.current.scrollTop = panStart.y - touch.clientY;
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
      onMouseDown={handleMouseDown}
      onTouchStart={handleTouchStart}
      onTouchMove={handleTouchMove}
      onTouchEnd={handleTouchEnd}
      style={{ cursor: isDragging ? 'grabbing' : 'grab' }}
    >
      <div className="schematic-view" ref={containerRef}>
        {/* mxgraph div is created dynamically in useEffect */}
      </div>
    </div>
  );
};

export default SchematicView;

