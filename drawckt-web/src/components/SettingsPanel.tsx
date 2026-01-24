import React, { useState, useRef, useEffect } from 'react';
import LayerStylesEditor from './LayerStylesEditor';
import { LayerStyles, ProcessResult, wasmAPI } from '../wasm';
import { toAbsolutePath } from '../utils/path';
import './SettingsPanel.css';

interface StyleSelectorProps {
  layerStyles: LayerStyles | null;
  onLayerStylesUpdate: (styles: LayerStyles) => void;
  onLayerStylesChange?: (styles: LayerStyles) => void; // For style switching without re-rendering
}

const StyleSelector: React.FC<StyleSelectorProps> = ({
  layerStyles,
  onLayerStylesUpdate,
  onLayerStylesChange,
}) => {
  const [styleNames, setStyleNames] = useState<string[]>([]);
  const [currentStyleName, setCurrentStyleName] = useState<string>('');
  const [isLoading, setIsLoading] = useState(false);
  const styleFileInputRef = useRef<HTMLInputElement>(null);
  const styleSelectRef = useRef<HTMLSelectElement>(null);

  const loadStyleNames = async () => {
    try {
      const names = await wasmAPI.getAllStyleNames();
      setStyleNames(names);
      // Get current style name from WASM
      const stylesState = await wasmAPI.getStylesState();
      if (stylesState && typeof stylesState === 'object' && 'current_name' in stylesState) {
        setCurrentStyleName((stylesState as any).current_name || '');
      }
    } catch (error) {
      console.error('Failed to load style names:', error);
    }
  };

  useEffect(() => {
    loadStyleNames();
  }, []);

  // Reload style names and current name when layerStyles changes (e.g., after loading ZIP)
  useEffect(() => {
    if (layerStyles) {
      loadStyleNames();
    }
  }, [layerStyles]);

  const handleStyleSelect = async (event: React.ChangeEvent<HTMLSelectElement>) => {
    const selectedValue = event.target.value;
    
    // Special value for upload option
    if (selectedValue === '__upload__') {
      styleFileInputRef.current?.click();
      // Reset select to current style name after a short delay
      setTimeout(() => {
        if (styleSelectRef.current) {
          styleSelectRef.current.value = currentStyleName;
        }
      }, 0);
      return;
    }

    if (!selectedValue) return;

    setIsLoading(true);
    try {
      await wasmAPI.setCurrentStyle(selectedValue);
      setCurrentStyleName(selectedValue);
      // Reload layer styles
      const newStyles = await wasmAPI.getLayerStyles();
      // Use onLayerStylesChange if available (for style switching without triggering updateLayerStyles)
      // Otherwise use onLayerStylesUpdate (for normal updates)
      if (onLayerStylesChange) {
        onLayerStylesChange(newStyles);
      } else {
        onLayerStylesUpdate(newStyles);
      }
      // Reload style names in case they changed
      const names = await wasmAPI.getAllStyleNames();
      setStyleNames(names);
    } catch (error) {
      console.error('Failed to set style:', error);
      alert(`Failed to set style: ${error}`);
      // Reset select to current style name on error
      if (styleSelectRef.current) {
        styleSelectRef.current.value = currentStyleName;
      }
    } finally {
      setIsLoading(false);
    }
  };

  const handleStyleFileUpload = async (event: React.ChangeEvent<HTMLInputElement>) => {
    const file = event.target.files?.[0];
    if (!file) return;

    if (!file.name.endsWith('.json')) {
      alert('Please upload a .json file');
      return;
    }

    const reader = new FileReader();
    reader.onload = async (e) => {
      try {
        const content = e.target?.result as string;
        const styles: LayerStyles = JSON.parse(content);
        
        // Extract name from filename (without .json extension)
        const name = file.name.replace(/\.json$/, '');
        
        setIsLoading(true);
        try {
          await wasmAPI.addStyle(name, JSON.stringify(styles));
          setCurrentStyleName(name);
          onLayerStylesUpdate(styles);
          // Reload style names
          const names = await wasmAPI.getAllStyleNames();
          setStyleNames(names);
        } catch (error) {
          console.error('Failed to add style:', error);
          alert(`Failed to add style: ${error}`);
        } finally {
          setIsLoading(false);
        }
      } catch (error) {
        console.error('Failed to parse style file:', error);
        alert(`Failed to parse style file: ${error}`);
      }
    };
    reader.onerror = () => {
      alert('Failed to read file');
    };
    reader.readAsText(file);
    
    // Reset input
    if (styleFileInputRef.current) {
      styleFileInputRef.current.value = '';
    }
  };

  return (
    <>
      <select
        ref={styleSelectRef}
        id="style-select"
        className="demo-select"
        value={currentStyleName}
        onChange={handleStyleSelect}
        disabled={isLoading}
        style={{
          flex: 1,
        }}
      >
        {styleNames.map((name) => (
          <option key={name} value={name}>
            {name}
          </option>
        ))}
        <option value="__upload__">
          📤 Upload Style JSON...
        </option>
      </select>
      <input
        ref={styleFileInputRef}
        type="file"
        accept=".json"
        onChange={handleStyleFileUpload}
        style={{ display: 'none' }}
      />
    </>
  );
};

interface SettingsPanelProps {
  onSchematicUpload: (jsonContent: string, filename?: string) => Promise<ProcessResult>;
  onSchematicZipUpload?: (base64Zip: string, filename?: string) => Promise<ProcessResult>;
  onSchematicClear?: () => void;
  layerStyles: LayerStyles | null;
  onLayerStylesUpdate: (styles: LayerStyles) => void;
  onLayerStylesChange?: (styles: LayerStyles) => void; // For style switching without re-rendering
  canExport: boolean;
  onExport: () => void;
}

const SettingsPanel: React.FC<SettingsPanelProps> = ({
  onSchematicUpload,
  onSchematicZipUpload,
  onSchematicClear,
  layerStyles,
  onLayerStylesUpdate,
  onLayerStylesChange,
  canExport,
  onExport,
}) => {
  const fileInputRef = useRef<HTMLInputElement>(null);
  const uploadStatusTimeoutRef = useRef<number | null>(null);
  const demoValueRef = useRef<string>('');
  const [isDragging, setIsDragging] = useState(false);
  const [uploadStatus, setUploadStatus] = useState<string>('');
  const [selectedDemo, setSelectedDemo] = useState<string>('');
  const [demoSelectKey, setDemoSelectKey] = useState(0);
  const [demoList, setDemoList] = useState<Array<{ value: string; label: string }>>([
    { value: '', label: '-- Select a demo case --' },
  ]);
  const [fixedCurrentStyle, setFixedCurrentStyle] = useState<LayerStyles | null>(null);

  // Load demo list from WASM on mount
  useEffect(() => {
    const loadDemoList = async () => {
      try {
        const demos = await wasmAPI.getDemoList();
        const demoOptions = [
          { value: '', label: '-- Select a demo case --' },
          ...demos.map((name) => ({
            value: name,
            label: name, // Keep extension in display
          })),
        ];
        setDemoList(demoOptions);
      } catch (error) {
        console.error('Failed to load demo list:', error);
      }
    };
    loadDemoList();
  }, []);

  // Load fixed current style from WASM on mount and when layerStyles changes
  const loadFixedCurrentStyle = async () => {
    try {
      const fixedStyle = await wasmAPI.getCurrentFixedLayerStyles();
      setFixedCurrentStyle(fixedStyle);
    } catch (error) {
      console.error('Failed to load fixed current style:', error);
    }
  };

  useEffect(() => {
    loadFixedCurrentStyle();
  }, []);

  // Reload fixed current style when layerStyles changes (e.g., after loading ZIP or switching style)
  useEffect(() => {
    if (layerStyles) {
      loadFixedCurrentStyle();
    }
  }, [layerStyles]);

  // Restore selectedDemo after key change to preserve selection state
  useEffect(() => {
    if (demoValueRef.current) {
      setSelectedDemo(demoValueRef.current);
    }
  }, [demoSelectKey]);

  // Reset page title when uploadStatus is empty or error
  useEffect(() => {
    if (!uploadStatus || uploadStatus.startsWith('✗')) {
      document.title = 'DrawCkt';
    }
  }, [uploadStatus]);

  // Helper function to set page title
  const setLoadedSuccess = (filename: string) => {
    setUploadStatus(`✓ Loaded '${filename}'`);
    document.title = `${filename} - DrawCkt`;
  };

  const handleSchematicContent = async (content: string, filename: string) => {
    // Cancel any pending auto-clear so failures can stay visible.
    if (uploadStatusTimeoutRef.current) {
      window.clearTimeout(uploadStatusTimeoutRef.current);
      uploadStatusTimeoutRef.current = null;
    }

    try {
      const result = await onSchematicUpload(content, filename);
      // Check both schematic_rendered and success to ensure UI updates
      const ok = Boolean(
        (result as any)?.success ?? 
        (result as any)?.schematic_rendered ?? 
        false
      );
      if (!ok) {
        const msg = (result as any)?.message ? String((result as any).message) : 'Failed to process schematic';
        setUploadStatus(`✗ ${msg}`);
        return; // keep showing
      }
      setLoadedSuccess(`${filename}.json`);
      // Keep showing success status until a new file is uploaded
    } catch (error) {
      const msg =
        error instanceof Error
          ? error.message
          : typeof error === 'string'
            ? error
            : JSON.stringify(error);
      setUploadStatus(`✗ ${msg}`);
      console.error('Upload error:', error);
    }
  };

  const handleFile = (file: File) => {
    const isJson = file.name.endsWith('.json');
    const isZip = file.name.endsWith('.zip');
    
    if (!isJson && !isZip) {
      setUploadStatus('✗ Please upload a .json or .zip file');
      setTimeout(() => setUploadStatus(''), 3000);
      return;
    }

    // Clear demo selection when user uploads their own file
    demoValueRef.current = '';
    setSelectedDemo('');
    setDemoSelectKey(prev => prev + 1);

    if (isJson) {
      const reader = new FileReader();
      reader.onload = async (e) => {
        const content = e.target?.result as string;
        // Extract filename without extension
        const filename = file.name.replace(/\.json$/, '');
        await handleSchematicContent(content, filename);
      };
      reader.onerror = () => {
        setUploadStatus('✗ Failed to read file');
        setTimeout(() => setUploadStatus(''), 3000);
      };
      reader.readAsText(file);
    } else if (isZip) {
      const reader = new FileReader();
      reader.onload = async (e) => {
        try {
          // Read file as ArrayBuffer and convert to base64
          const arrayBuffer = e.target?.result as ArrayBuffer;
          const bytes = new Uint8Array(arrayBuffer);
          const binary = Array.from(bytes, byte => String.fromCharCode(byte)).join('');
          const base64 = btoa(binary);
          
          // Extract filename without extension
          const filename = file.name.replace(/\.zip$/, '');
          
          // Process ZIP file using callback if available, otherwise use WASM API directly
          if (onSchematicZipUpload) {
            const result = await onSchematicZipUpload(base64, filename);
            const ok = Boolean((result as any)?.success ?? (result as any)?.schematic_restored ?? false);
            if (!ok) {
              const msg = (result as any)?.message ? String((result as any).message) : 'Failed to process ZIP file';
              setUploadStatus(`✗ ${msg}`);
              return;
            }
            setLoadedSuccess(`${filename}.zip`);
          } else {
            // Fallback: use WASM API directly
            const result = await wasmAPI.processSchematicZip(base64, filename);
            const ok = Boolean((result as any)?.success ?? (result as any)?.schematic_restored ?? false);
            if (!ok) {
              const msg = (result as any)?.message ? String((result as any).message) : 'Failed to process ZIP file';
              setUploadStatus(`✗ ${msg}`);
              return;
            }
            setLoadedSuccess(`${filename}.zip`);
          }
        } catch (error) {
          const msg =
            error instanceof Error
              ? error.message
              : typeof error === 'string'
                ? error
                : JSON.stringify(error);
          setUploadStatus(`✗ ${msg}`);
          console.error('ZIP upload error:', error);
        }
      };
      reader.onerror = () => {
        setUploadStatus('✗ Failed to read file');
        setTimeout(() => setUploadStatus(''), 3000);
      };
      reader.readAsArrayBuffer(file);
    }
  };

  const handleDemoSelect = async (event: React.ChangeEvent<HTMLSelectElement>) => {
    const demoValue = event.target.value;
    
    if (!demoValue) {
      // Clear upload status and schematic when empty option is selected
      setUploadStatus('');
      if (onSchematicClear) {
        onSchematicClear();
      }
      // Reset select by changing key to allow re-selecting
      setDemoSelectKey(prev => prev + 1);
      // Clear the stored value
      demoValueRef.current = '';
      setSelectedDemo('');
      return;
    }

    // Store the selected value in ref and state
    demoValueRef.current = demoValue;
    setSelectedDemo(demoValue);

    try {
      const isZip = demoValue.endsWith('.zip');
      const isJson = demoValue.endsWith('.json');
      
      if (isZip) {
        // Load ZIP demo file
        const filename = demoValue.replace(/\.zip$/, '');
        // Load the demo ZIP file from WASM (as base64)
        const content = await wasmAPI.loadDemo(demoValue);
        // Process ZIP file using callback if available, otherwise use WASM API directly
        if (onSchematicZipUpload) {
          const result = await onSchematicZipUpload(content, filename);
          const ok = Boolean((result as any)?.success ?? (result as any)?.schematic_restored ?? false);
          if (!ok) {
            const msg = (result as any)?.message ? String((result as any).message) : 'Failed to process ZIP demo';
            setUploadStatus(`✗ ${msg}`);
            setDemoSelectKey(prev => prev + 1);
            return;
          }
          setLoadedSuccess(`${filename}.zip`);
        } else {
          // Fallback: use WASM API directly
          const result = await wasmAPI.processSchematicZip(content, filename);
          const ok = Boolean((result as any)?.success ?? (result as any)?.schematic_restored ?? false);
          if (!ok) {
            const msg = (result as any)?.message ? String((result as any).message) : 'Failed to process ZIP demo';
            setUploadStatus(`✗ ${msg}`);
            setDemoSelectKey(prev => prev + 1);
            return;
          }
          setLoadedSuccess(`${filename}.zip`);
        }
      } else if (isJson) {
        // Load JSON demo file
        const filename = demoValue.replace(/\.json$/, '');
        const content = await wasmAPI.loadDemo(demoValue);
        await handleSchematicContent(content, filename);
      } else {
        throw new Error('Unsupported demo file type');
      }
      
      // Reset select by changing key to allow re-selecting the same option
      // The selectedDemo will be restored by useEffect after key change
      setDemoSelectKey(prev => prev + 1);
    } catch (error) {
      const msg =
        error instanceof Error
          ? error.message
          : typeof error === 'string'
            ? error
            : JSON.stringify(error);
      setUploadStatus(`✗ Failed to load demo: ${msg}`);
      console.error('Demo load error:', error);
      // Reset select even on error to allow retry
      // The selectedDemo will be restored by useEffect after key change
      setDemoSelectKey(prev => prev + 1);
    }
  };

  const handleFileSelect = (event: React.ChangeEvent<HTMLInputElement>) => {
    const file = event.target.files?.[0];
    if (file) {
      handleFile(file);
    }
    // Reset input value to allow selecting the same file again
    if (fileInputRef.current) {
      fileInputRef.current.value = '';
    }
  };

  const handleDragOver = (e: React.DragEvent) => {
    e.preventDefault();
    e.stopPropagation();
    setIsDragging(true);
  };

  const handleDragLeave = (e: React.DragEvent) => {
    e.preventDefault();
    e.stopPropagation();
    setIsDragging(false);
  };

  const handleDrop = (e: React.DragEvent) => {
    e.preventDefault();
    e.stopPropagation();
    setIsDragging(false);

    const file = e.dataTransfer.files?.[0];
    if (file) {
      handleFile(file);
    }
  };

  return (
    <div className="settings-panel">
      <div className="settings-content">
        <section className="settings-section">
          <div className="about-content">
            <p className="about-description">
              A online schematic viewer and editor.
            </p>

            <div className="demo-selector">
              <select
                key={demoSelectKey}
                id="demo-select"
                className="demo-select"
                value={selectedDemo}
                onChange={handleDemoSelect}
              >
                {demoList.map((demo) => {
                  const label = demo.label || demo.value || '';
                  return (
                    <option key={demo.value} value={demo.value}>
                      {label}
                    </option>
                  );
                })}
              </select>
            </div>

            <div className="upload-divider">
              <span className="divider-text">or</span>
            </div>
            <p className="about-description">
              <a 
                href={toAbsolutePath('/doc/')}
                className="doc-link"
              >
                View documentation
              </a>
              &nbsp;to create a schematic.json from Virtuoso.
            </p>
            <div
              className={`upload-dropzone ${isDragging ? 'dragging' : ''} ${uploadStatus ? (uploadStatus.startsWith('✓') ? 'success' : 'error') : ''}`}
              onDragOver={handleDragOver}
              onDragLeave={handleDragLeave}
              onDrop={handleDrop}
              onClick={() => fileInputRef.current?.click()}
            >
              <input
                ref={fileInputRef}
                type="file"
                accept=".json,.zip"
                onChange={handleFileSelect}
                style={{ display: 'none' }}
              />
              <div className="upload-icon">📄 / 📦︎</div>
              {!uploadStatus && (
                <>
                  <div className="upload-text">Upload json or zip file</div>
                  <div className="upload-hint">Click or drag file here</div>
                </>
              )}
              {uploadStatus && (
                <div className={`upload-status-inline ${uploadStatus.startsWith('✓') ? 'success' : 'error'}`}>
                  {uploadStatus}
                </div>
              )}
            </div>
            <button
              className={`export-btn ${canExport ? 'enabled' : 'disabled'}`}
              onClick={canExport ? onExport : undefined}
              disabled={!canExport}
              title={canExport ? 'Export all files as ZIP' : 'No schematic loaded'}
            >
              Export All
            </button>
          </div>
        </section>
        
        <section className="settings-section">
          <div style={{ display: 'flex', alignItems: 'center', gap: '12px', marginBottom: '16px' }}>
            <h3 style={{ margin: 0 }}>Layer Style: </h3>
            <StyleSelector
              layerStyles={layerStyles}
              onLayerStylesUpdate={onLayerStylesUpdate}
              onLayerStylesChange={onLayerStylesChange}
            />
          </div>
          {layerStyles && (
            <LayerStylesEditor
              styles={layerStyles}
              onUpdate={onLayerStylesUpdate}
              wasmStyles={layerStyles}
              fixedCurrentStyle={fixedCurrentStyle || undefined}
            />
          )}
        </section>

        <div className="logger-spacing"></div>
      </div>
    </div>
  );
};

export default SettingsPanel;

