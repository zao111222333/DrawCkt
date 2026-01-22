import React, { useState, useRef, useEffect } from 'react';
import LayerStylesEditor from './LayerStylesEditor';
import { LayerStyles, ProcessResult, wasmAPI } from '../wasm';
import { toAbsolutePath } from '../utils/path';
import './SettingsPanel.css';

interface SettingsPanelProps {
  onSchematicUpload: (jsonContent: string, filename?: string) => Promise<ProcessResult>;
  onSchematicZipUpload?: (base64Zip: string, filename?: string) => Promise<ProcessResult>;
  onSchematicClear?: () => void;
  layerStyles: LayerStyles | null;
  onLayerStylesUpdate: (styles: LayerStyles) => void;
  canExport: boolean;
  onExport: () => void;
}

const SettingsPanel: React.FC<SettingsPanelProps> = ({
  onSchematicUpload,
  onSchematicZipUpload,
  onSchematicClear,
  layerStyles,
  onLayerStylesUpdate,
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
  const [defaultLayerStyles, setDefaultLayerStyles] = useState<LayerStyles | null>(null);

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

  // Load default layer styles from WASM on mount
  useEffect(() => {
    const loadDefaultLayerStyles = async () => {
      try {
        const defaultStyles = await wasmAPI.getDefaultLayerStyles();
        setDefaultLayerStyles(defaultStyles);
      } catch (error) {
        console.error('Failed to load default layer styles:', error);
      }
    };
    loadDefaultLayerStyles();
  }, []);

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
              Export
            </button>
          </div>
        </section>
        
        <section className="settings-section">
          <h3>Layers</h3>
          {layerStyles && (
            <LayerStylesEditor
              styles={layerStyles}
              onUpdate={onLayerStylesUpdate}
              wasmStyles={layerStyles}
              defaultStyles={defaultLayerStyles || undefined}
            />
          )}
        </section>

        <div className="logger-spacing"></div>
      </div>
    </div>
  );
};

export default SettingsPanel;

