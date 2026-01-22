import React, { useState, useRef, useEffect } from 'react';
import LayerStylesEditor from './LayerStylesEditor';
import { LayerStyles, ProcessResult, wasmAPI } from '../wasm';
import { toAbsolutePath } from '../utils/path';
import './SettingsPanel.css';

interface SettingsPanelProps {
  onSchematicUpload: (jsonContent: string, filename?: string) => Promise<ProcessResult>;
  onSchematicClear?: () => void;
  layerStyles: LayerStyles | null;
  onLayerStylesUpdate: (styles: LayerStyles) => void;
  canExport: boolean;
  onExport: () => void;
}

const SettingsPanel: React.FC<SettingsPanelProps> = ({
  onSchematicUpload,
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

  // Load demo list from WASM on mount
  useEffect(() => {
    const loadDemoList = async () => {
      try {
        const demos = await wasmAPI.getDemoList();
        const demoOptions = [
          { value: '', label: '-- Select a demo case --' },
          ...demos.map((name) => ({
            value: name,
            label: name.replace(/\.json$/, ''),
          })),
        ];
        setDemoList(demoOptions);
      } catch (error) {
        console.error('Failed to load demo list:', error);
      }
    };
    loadDemoList();
  }, []);

  // Restore selectedDemo after key change to preserve selection state
  useEffect(() => {
    if (demoValueRef.current) {
      setSelectedDemo(demoValueRef.current);
    }
  }, [demoSelectKey]);

  const handleSchematicContent = async (content: string, filename: string) => {
    // Cancel any pending auto-clear so failures can stay visible.
    if (uploadStatusTimeoutRef.current) {
      window.clearTimeout(uploadStatusTimeoutRef.current);
      uploadStatusTimeoutRef.current = null;
    }

    try {
      const result = await onSchematicUpload(content, filename);
      const ok = Boolean((result as any)?.success ?? (result as any)?.schematic_rendered ?? false);
      if (!ok) {
        const msg = (result as any)?.message ? String((result as any).message) : 'Failed to process schematic';
        setUploadStatus(`✗ ${msg}`);
        return; // keep showing
      }

      setUploadStatus(`✓ Loaded '${filename}.json'`);
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
    if (!file.name.endsWith('.json')) {
      setUploadStatus('✗ Please upload a .json file');
      setTimeout(() => setUploadStatus(''), 3000);
      return;
    }

    // Clear demo selection when user uploads their own file
    demoValueRef.current = '';
    setSelectedDemo('');
    setDemoSelectKey(prev => prev + 1);

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
      // Remove .json extension from filename for processing
      const filename = demoValue.replace(/\.json$/, '');
      // Load the demo JSON file from WASM
      const content = await wasmAPI.loadDemo(demoValue);
      await handleSchematicContent(content, filename);
      
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
                  const label = demo.label || (demo.value ? demo.value.replace(/\.json$/, '') : '');
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
                accept=".json"
                onChange={handleFileSelect}
                style={{ display: 'none' }}
              />
              <div className="upload-icon">📄</div>
              {!uploadStatus && (
                <>
                  <div className="upload-text">Upload Schematic.json</div>
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
          {layerStyles ? (
            <LayerStylesEditor
              styles={layerStyles}
              onUpdate={onLayerStylesUpdate}
            />
          ) : (
            <div className="info-text">Loading layer styles...</div>
          )}
        </section>

        <div className="logger-spacing"></div>
      </div>
    </div>
  );
};

export default SettingsPanel;

