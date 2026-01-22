import React, { useEffect, useRef, useState } from 'react';
import './SymbolEditor.css';

interface SymbolEditorProps {
  lib: string;
  cell: string;
  content: string;
  onSave: (lib: string, cell: string, newContent: string) => void;
  onClose: () => void;
  isDarkMode?: boolean; // Dark mode state
}

const SymbolEditor: React.FC<SymbolEditorProps> = ({ lib, cell, content, onSave, onClose, isDarkMode = false }) => {
  const iframeRef = useRef<HTMLIFrameElement>(null);
  const isInitializedRef = useRef(false);
  const currentContentRef = useRef(content);
  const editorReadyRef = useRef(false);
  const [isLoading, setIsLoading] = useState(true);

  useEffect(() => {
    currentContentRef.current = content;
    // If editor is already ready, reload the content
    if (editorReadyRef.current && iframeRef.current?.contentWindow) {
      iframeRef.current.contentWindow.postMessage(
        JSON.stringify({
          action: 'load',
          xml: content,
        }),
        '*'
      );
    }
  }, [content]);

  useEffect(() => {
    if (!iframeRef.current) {
      return;
    }

    const iframe = iframeRef.current;
    
    // Re-initialize if dark mode changes after initial load
    // This ensures the theme is applied when user switches dark mode while editor is open
    const shouldReinit = isInitializedRef.current && editorReadyRef.current;
    
    if (!isInitializedRef.current || shouldReinit) {
      // Use embed.diagrams.net which is specifically designed for embedding
      // and doesn't have CSP frame-ancestors restrictions
      // protoVersion=1 is important for newer versions
      // Set UI theme based on dark mode: 'dark' for dark mode, 'atlas' for light mode
      const uiTheme = isDarkMode ? 'dark' : 'atlas';
      const embedUrl = `https://embed.diagrams.net/?embed=1&proto=json&configure=1&ui=${uiTheme}&noSaveBtn=1&noExitBtn=1&spin=1&protoVersion=1`;
      iframe.src = embedUrl;
      isInitializedRef.current = true;
      editorReadyRef.current = false;
      setIsLoading(true);
    }

    // Listen for messages from the iframe
    const handleMessage = (event: MessageEvent) => {
      // Log all messages for debugging
      
      // Security: Only accept messages from draw.io domain
      const allowedOrigins = [
        'https://embed.diagrams.net',
        'https://www.draw.io',
        'https://app.diagrams.net',
        'https://draw.io',
      ];
      
      // Check if origin matches any allowed pattern
      const isAllowed = allowedOrigins.some(origin => {
        return event.origin === origin || event.origin.startsWith(origin);
      });
      
      if (!isAllowed) {
        console.warn('Rejected message from origin:', event.origin, 'Expected one of:', allowedOrigins);
        return;
      }

      let msg = event.data;
      
      // Parse message if it's a string (some browsers may stringify JSON)
      if (typeof msg === 'string') {
        try {
          msg = JSON.parse(msg);
        } catch (e) {
          console.warn('Failed to parse message as JSON:', msg, e);
          return;
        }
      }
      
      // Validate message format - must be a plain object with event property
      // Note: typeof null === 'object' in JavaScript, so we need to check for null explicitly
      if (msg === null || msg === undefined) {
        console.warn('Message is null or undefined:', msg);
        return;
      }
      
      if (typeof msg !== 'object' || Array.isArray(msg)) {
        console.warn('Message is not a plain object:', msg, 'type:', typeof msg);
        return;
      }
      
      // Check if it's a valid draw.io message (should have event property)
      if (!msg.event || typeof msg.event !== 'string') {
        console.warn('Message missing or invalid event property:', msg);
        return;
      }
      

      // Handle different message types
      // IMPORTANT: When configure=1 is used, configure event comes BEFORE init event
      // We must respond to configure before editor will send init event
      if (msg.event === 'configure') {
        // When configure=1 is used, editor sends configure event first
        // We need to respond with configure action before init event
        // NOTE: Even with proto=json, draw.io may expect JSON string format
        if (iframe.contentWindow) {
          const configResponse = {
            action: 'configure',
            config: {
              // Set theme based on dark mode
              // draw.io uses 'defaultTheme' or 'theme' property to set the UI theme
              defaultTheme: isDarkMode ? 'dark' : 'atlas',
            },
          };
          // Send as JSON string - draw.io seems to expect this format
          iframe.contentWindow.postMessage(JSON.stringify(configResponse), '*');
        }
      } else if (msg.event === 'init') {
        editorReadyRef.current = true;
        setIsLoading(false);
        // Editor is ready, send the XML content to load
        // Add a small delay to ensure editor is fully ready
        setTimeout(() => {
          if (iframe.contentWindow) {
            const xmlContent = currentContentRef.current;
            
            // Validate XML content
            if (!xmlContent || xmlContent.trim().length === 0) {
              setIsLoading(false);
              return;
            }
            
            const loadMessage = {
              action: 'load',
              xml: xmlContent,
            };
            
            // Send as JSON string - draw.io seems to expect this format
            iframe.contentWindow.postMessage(JSON.stringify(loadMessage), '*');
          }
        }, 300);
      } else if (msg.event === 'load') {
        setIsLoading(false);
      } else if (msg.event === 'save') {
        // User saved the diagram
        const newXml = msg.xml || currentContentRef.current;
        onSave(lib, cell, newXml);
        onClose();
      } else if (msg.event === 'exit') {
        // User closed the editor without saving
        onClose();
      }
    };

    // Also listen for iframe load event
    const handleIframeLoad = () => {
      // Iframe loaded
    };

    const handleIframeError = (_error: Event) => {
      // Iframe error - silently handle
    };

    iframe.addEventListener('load', handleIframeLoad);
    iframe.addEventListener('error', handleIframeError);
    window.addEventListener('message', handleMessage);

    return () => {
      window.removeEventListener('message', handleMessage);
      iframe.removeEventListener('load', handleIframeLoad);
      iframe.removeEventListener('error', handleIframeError);
      editorReadyRef.current = false;
    };
  }, [lib, cell, onSave, onClose, isDarkMode]);

  return (
    <div className="symbol-editor-overlay">
      <div className="symbol-editor-container">
        <div className="symbol-editor-header">
          <h3>Edit: {lib}/{cell}</h3>
          <button className="symbol-editor-close-btn" onClick={onClose}>
            ×
          </button>
        </div>
        <div className="symbol-editor-content">
          {isLoading && (
            <div className="symbol-editor-loading">
              Loading editor...
            </div>
          )}
          <iframe
            ref={iframeRef}
            className="symbol-editor-iframe"
            title="Draw.io Editor"
            allow="clipboard-read; clipboard-write"
            style={{ display: isLoading ? 'none' : 'block' }}
          />
        </div>
      </div>
    </div>
  );
};

export default SymbolEditor;
