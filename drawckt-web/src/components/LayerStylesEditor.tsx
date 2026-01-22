import React, { useState, useEffect, useRef } from 'react';
import { LayerStyles } from '../wasm';
import './LayerStylesEditor.css';

interface LayerStylesEditorProps {
  styles: LayerStyles;
  onUpdate: (styles: LayerStyles) => void;
}

// Deep comparison function for LayerStyles
const areStylesEqual = (a: LayerStyles, b: LayerStyles): boolean => {
  const layers: Array<keyof LayerStyles> = ['instance', 'device', 'annotate', 'pin', 'wire'];
  
  for (const layerKey of layers) {
    const layerA = a[layerKey];
    const layerB = b[layerKey];
    
    if (
      layerA.stroke_color !== layerB.stroke_color ||
      Math.abs(layerA.stroke_width - layerB.stroke_width) > Number.EPSILON ||
      layerA.text_color !== layerB.text_color ||
      Math.abs(layerA.font_size - layerB.font_size) > Number.EPSILON ||
      layerA.priority !== layerB.priority ||
      layerA.sch_visible !== layerB.sch_visible
    ) {
      return false;
    }
  }
  
  return true;
};

// Helper function to create a deep copy of LayerStyles
const deepCopyLayerStyles = (styles: LayerStyles): LayerStyles => {
  return {
    instance: { ...styles.instance },
    device: { ...styles.device },
    annotate: { ...styles.annotate },
    pin: { ...styles.pin },
    wire: { ...styles.wire },
  };
};

const LayerStylesEditor: React.FC<LayerStylesEditorProps> = ({ styles, onUpdate }) => {
  const [localStyles, setLocalStyles] = useState<LayerStyles>(() => deepCopyLayerStyles(styles));
  const [savedStyles, setSavedStyles] = useState<LayerStyles>(() => deepCopyLayerStyles(styles));
  const [expandedLayer, setExpandedLayer] = useState<string | null>(null);
  const isInitialMount = useRef(true);

  // Update savedStyles when props.styles changes (from external updates)
  useEffect(() => {
    if (!isInitialMount.current) {
      // External update happened, sync savedStyles and localStyles
      // Only update if styles actually changed
      setSavedStyles((prevSaved) => {
        if (!areStylesEqual(styles, prevSaved)) {
          // Update both savedStyles and localStyles when external update happens
          const newStyles = deepCopyLayerStyles(styles);
          setLocalStyles(newStyles);
          return newStyles;
        }
        return prevSaved;
      });
    } else {
      isInitialMount.current = false;
    }
  }, [styles]);

  // Check if localStyles differ from savedStyles
  // This will be recalculated on every render when localStyles or savedStyles change
  const hasChanges = !areStylesEqual(localStyles, savedStyles);

  const updateLayer = (layerName: keyof LayerStyles, field: string, value: any) => {
    const newStyles = {
      ...localStyles,
      [layerName]: {
        ...localStyles[layerName],
        [field]: value,
      },
    };
    setLocalStyles(newStyles);
  };

  const handleSave = () => {
    onUpdate(localStyles);
    setSavedStyles(deepCopyLayerStyles(localStyles));
  };

  const layers: Array<{ key: keyof LayerStyles; label: string }> = [
    { key: 'instance', label: 'Instance' },
    { key: 'device', label: 'Device' },
    { key: 'annotate', label: 'Annotate' },
    { key: 'pin', label: 'Pin' },
    { key: 'wire', label: 'Wire' },
  ];

  return (
    <div className="layer-styles-editor">
      {layers.map(({ key, label }) => {
        const layer = localStyles[key];
        const isExpanded = expandedLayer === key;

        return (
          <div key={key} className="layer-item">
            <div
              className="layer-header"
              onClick={() => setExpandedLayer(isExpanded ? null : key)}
            >
              <span className="layer-name">{label}</span>
              <span className="expand-icon">{isExpanded ? '▼' : '▶'}</span>
            </div>
            {isExpanded && (
              <div className="layer-fields">
                <div className="field">
                  <label>Line Color</label>
                  <input
                    type="color"
                    value={layer.stroke_color}
                    onChange={(e) => updateLayer(key, 'stroke_color', e.target.value)}
                  />
                  <input
                    type="text"
                    value={layer.stroke_color}
                    onChange={(e) => updateLayer(key, 'stroke_color', e.target.value)}
                    className="color-text"
                  />
                </div>
                <div className="field">
                  <label>Line Width</label>
                  <input
                    type="number"
                    step="0.1"
                    value={layer.stroke_width}
                    onChange={(e) => updateLayer(key, 'stroke_width', parseFloat(e.target.value))}
                  />
                </div>
                <div className="field">
                  <label>Text Color</label>
                  <input
                    type="color"
                    value={layer.text_color}
                    onChange={(e) => updateLayer(key, 'text_color', e.target.value)}
                  />
                  <input
                    type="text"
                    value={layer.text_color}
                    onChange={(e) => updateLayer(key, 'text_color', e.target.value)}
                    className="color-text"
                  />
                </div>
                <div className="field">
                  <label>Font Size</label>
                  <input
                    type="number"
                    step="0.1"
                    value={layer.font_size}
                    onChange={(e) => updateLayer(key, 'font_size', parseFloat(e.target.value))}
                  />
                </div>
                <div className="field">
                  <label>Priority</label>
                  <input
                    type="number"
                    value={layer.priority}
                    onChange={(e) => updateLayer(key, 'priority', parseInt(e.target.value))}
                  />
                </div>
                <div className="field">
                  <label>
                    <input
                      type="checkbox"
                      checked={layer.sch_visible}
                      onChange={(e) => updateLayer(key, 'sch_visible', e.target.checked)}
                    />
                    Schematic Visible
                  </label>
                </div>
              </div>
            )}
          </div>
        );
      })}
      <button 
        className={`update-btn ${hasChanges ? 'enabled' : 'disabled'}`}
        onClick={hasChanges ? handleSave : undefined}
        disabled={!hasChanges}
        title={hasChanges ? 'Apply layer style changes' : 'No changes to apply'}
      >
        Update
      </button>
    </div>
  );
};

export default LayerStylesEditor;

