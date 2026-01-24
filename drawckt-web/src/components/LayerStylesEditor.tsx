import React, { useState, useEffect, useRef } from 'react';
import { LayerStyles, wasmAPI } from '../wasm';
import './LayerStylesEditor.css';

interface LayerStylesEditorProps {
  styles: LayerStyles;
  onUpdate: (styles: LayerStyles) => void;
  wasmStyles?: LayerStyles; // Current WASM state
  fixedCurrentStyle?: LayerStyles; // Fixed (saved) style from WASM
  onReset?: () => void; // Callback for reset button
}

// Deep comparison function for LayerStyles
const areStylesEqual = (a: LayerStyles, b: LayerStyles): boolean => {
  // Check layer_order
  if (JSON.stringify(a.layer_order) !== JSON.stringify(b.layer_order)) {
    return false;
  }
  
  // Check wire_show_intersection
  if (a.wire_show_intersection !== b.wire_show_intersection) {
    return false;
  }
  
  // Check wire_intersection_scale
  if (Math.abs(a.wire_intersection_scale - b.wire_intersection_scale) > Number.EPSILON) {
    return false;
  }
  
  const layerKeys: Array<'instance' | 'device' | 'annotate' | 'pin' | 'wire' | 'text'> = ['instance', 'device', 'annotate', 'pin', 'wire', 'text'];
  
  for (const layerKey of layerKeys) {
    const layerA = a[layerKey];
    const layerB = b[layerKey];
    
    if (
      layerA.stroke_color !== layerB.stroke_color ||
      Math.abs(layerA.stroke_width - layerB.stroke_width) > Number.EPSILON ||
      layerA.text_color !== layerB.text_color ||
      Math.abs(layerA.font_zoom - layerB.font_zoom) > Number.EPSILON ||
      layerA.font_family !== layerB.font_family ||
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
    layer_order: [...styles.layer_order],
    instance: { ...styles.instance },
    device: { ...styles.device },
    annotate: { ...styles.annotate },
    pin: { ...styles.pin },
    wire: { ...styles.wire },
    wire_show_intersection: styles.wire_show_intersection,
    wire_intersection_scale: styles.wire_intersection_scale,
    text: { ...styles.text },
  };
};

// Recommended font list
const RECOMMENDED_FONTS = [
  'Verdana',
  'Times New Roman',
  'Arial',
  'Helvetica',
  'Georgia',
  'Courier New',
  'Comic Sans MS',
  'Trebuchet MS',
  'Impact',
  'Lucida Console',
  'Palatino',
  'Garamond',
  'Bookman',
  'Tahoma',
  'Monaco',
  'Menlo',
  'SF Mono',
];

// Custom Font Input Component
interface FontInputProps {
  value: string;
  onChange: (value: string) => void;
  id: string;
}

const FontInput: React.FC<FontInputProps> = ({ value, onChange, id }) => {
  const [isOpen, setIsOpen] = useState(false);
  const [inputValue, setInputValue] = useState(value);
  const [highlightedIndex, setHighlightedIndex] = useState(-1);
  const inputRef = useRef<HTMLInputElement>(null);
  const dropdownRef = useRef<HTMLDivElement>(null);
  const wrapperRef = useRef<HTMLDivElement>(null);

  // Sync inputValue with value prop
  useEffect(() => {
    setInputValue(value);
  }, [value]);

  // Filter fonts based on input
  const getFilteredFonts = () => {
    const lowerInput = inputValue.toLowerCase().trim();
    if (!lowerInput) {
      return RECOMMENDED_FONTS;
    }
    
    const filtered = RECOMMENDED_FONTS.filter(font =>
      font.toLowerCase().includes(lowerInput)
    );
    
    // If only one match and it exactly equals the input, show all fonts
    if (filtered.length === 1 && filtered[0].toLowerCase() === lowerInput) {
      return RECOMMENDED_FONTS;
    }
    
    return filtered;
  };

  const filteredFonts = getFilteredFonts();

  // Handle input change
  const handleInputChange = (e: React.ChangeEvent<HTMLInputElement>) => {
    const newValue = e.target.value;
    setInputValue(newValue);
    onChange(newValue);
    setIsOpen(true);
    setHighlightedIndex(-1);
  };

  // Handle input focus
  const handleInputFocus = () => {
    setIsOpen(true);
    setHighlightedIndex(-1);
  };

  // Handle font selection
  const handleFontSelect = (font: string) => {
    setInputValue(font);
    onChange(font);
    setIsOpen(false);
    inputRef.current?.focus();
  };

  // Handle keyboard navigation
  const handleKeyDown = (e: React.KeyboardEvent<HTMLInputElement>) => {
    if (!isOpen && (e.key === 'ArrowDown' || e.key === 'ArrowUp')) {
      setIsOpen(true);
      return;
    }

    if (!isOpen) return;

    switch (e.key) {
      case 'ArrowDown':
        e.preventDefault();
        setHighlightedIndex(prev => 
          prev < filteredFonts.length - 1 ? prev + 1 : prev
        );
        break;
      case 'ArrowUp':
        e.preventDefault();
        setHighlightedIndex(prev => prev > 0 ? prev - 1 : -1);
        break;
      case 'Enter':
        e.preventDefault();
        if (highlightedIndex >= 0 && highlightedIndex < filteredFonts.length) {
          handleFontSelect(filteredFonts[highlightedIndex]);
        }
        break;
      case 'Escape':
        setIsOpen(false);
        setHighlightedIndex(-1);
        break;
    }
  };

  // Close dropdown when clicking outside
  useEffect(() => {
    const handleClickOutside = (event: MouseEvent) => {
      if (
        wrapperRef.current &&
        !wrapperRef.current.contains(event.target as Node)
      ) {
        setIsOpen(false);
        setHighlightedIndex(-1);
      }
    };

    if (isOpen) {
      document.addEventListener('mousedown', handleClickOutside);
      return () => {
        document.removeEventListener('mousedown', handleClickOutside);
      };
    }
  }, [isOpen]);

  return (
    <div className="font-input-wrapper" ref={wrapperRef}>
      <input
        ref={inputRef}
        type="text"
        id={id}
        value={inputValue}
        onChange={handleInputChange}
        onFocus={handleInputFocus}
        onKeyDown={handleKeyDown}
        className="font-input"
        placeholder="Select or type font name"
        autoComplete="off"
      />
      {isOpen && filteredFonts.length > 0 && (
        <div className="font-dropdown" ref={dropdownRef}>
          {filteredFonts.map((font, index) => (
            <div
              key={font}
              className={`font-option ${index === highlightedIndex ? 'highlighted' : ''}`}
              onClick={() => handleFontSelect(font)}
              onMouseEnter={() => setHighlightedIndex(index)}
            >
              {font}
            </div>
          ))}
        </div>
      )}
    </div>
  );
};

const LayerStylesEditor: React.FC<LayerStylesEditorProps> = ({ 
  styles, 
  onUpdate, 
  wasmStyles,
  fixedCurrentStyle,
  onReset,
}) => {
  const [localStyles, setLocalStyles] = useState<LayerStyles>(() => deepCopyLayerStyles(styles));
  const [savedStyles, setSavedStyles] = useState<LayerStyles>(() => deepCopyLayerStyles(styles));
  const [expandedLayer, setExpandedLayer] = useState<string | null>(null);
  const [draggedIndex, setDraggedIndex] = useState<number | null>(null);
  const [dragOverIndex, setDragOverIndex] = useState<number | null>(null);
  const draggedLayerNameRef = useRef<string | null>(null);
  const lastDragOverIndexRef = useRef<number | null>(null);
  const dragStartPosRef = useRef<{ x: number; y: number; time: number } | null>(null);
  const originalLayerOrderRef = useRef<string[] | null>(null);
  const isDraggingRef = useRef<boolean>(false);
  const isInitialMount = useRef(true);
  
  // Drag threshold: minimum distance (in pixels) or time (in ms) to consider it a drag
  const DRAG_THRESHOLD_DISTANCE = 5;
  const DRAG_THRESHOLD_TIME = 200;

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
  
  // Check if WASM current state differs from fixed style (for reset and save button disable condition)
  const wasmDiffersFromFixed = wasmStyles && fixedCurrentStyle ? !areStylesEqual(wasmStyles, fixedCurrentStyle) : true;

  // Auto-update when styles change (but not on initial mount or external updates)
  useEffect(() => {
    if (!isInitialMount.current && hasChanges) {
      onUpdate(localStyles);
      setSavedStyles(deepCopyLayerStyles(localStyles));
    }
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [localStyles]);

  const updateLayer = (layerName: 'instance' | 'device' | 'annotate' | 'pin' | 'wire' | 'text', field: string, value: any) => {
    const newStyles = {
      ...localStyles,
      [layerName]: {
        ...localStyles[layerName],
        [field]: value,
      },
    };
    setLocalStyles(newStyles);
  };

  const updateWireShowIntersection = (value: boolean) => {
    setLocalStyles({
      ...localStyles,
      wire_show_intersection: value,
    });
  };

  const updateWireIntersectionScale = (value: number) => {
    setLocalStyles({
      ...localStyles,
      wire_intersection_scale: value,
    });
  };

  const moveLayer = (layerName: string, direction: 'up' | 'down') => {
    const currentOrder = [...localStyles.layer_order];
    const index = currentOrder.indexOf(layerName);
    
    if (index === -1) return;
    
    if (direction === 'up' && index > 0) {
      [currentOrder[index], currentOrder[index - 1]] = [currentOrder[index - 1], currentOrder[index]];
    } else if (direction === 'down' && index < currentOrder.length - 1) {
      [currentOrder[index], currentOrder[index + 1]] = [currentOrder[index + 1], currentOrder[index]];
    } else {
      return;
    }
    
    setLocalStyles({
      ...localStyles,
      layer_order: currentOrder,
    });
  };

  const handleDragStart = (e: React.DragEvent, index: number) => {
    // Record initial position and time
    dragStartPosRef.current = {
      x: e.clientX,
      y: e.clientY,
      time: Date.now(),
    };
    
    // Save original layer order in case we need to revert
    originalLayerOrderRef.current = [...localStyles.layer_order];
    
    const layerName = localStyles.layer_order[index];
    setDraggedIndex(index);
    draggedLayerNameRef.current = layerName;
    lastDragOverIndexRef.current = null;
    isDraggingRef.current = true;
    e.dataTransfer.effectAllowed = 'move';
    e.dataTransfer.setData('text/html', ''); // Required for Firefox
    // Make the dragged element semi-transparent
    if (e.currentTarget instanceof HTMLElement) {
      e.currentTarget.style.opacity = '0.5';
    }
  };

  const handleDragOver = (e: React.DragEvent, index: number) => {
    e.preventDefault();
    
    // Check if this is actually a drag (not just a click)
    const dragStart = dragStartPosRef.current;
    if (dragStart) {
      const distance = Math.sqrt(
        Math.pow(e.clientX - dragStart.x, 2) + Math.pow(e.clientY - dragStart.y, 2)
      );
      const time = Date.now() - dragStart.time;
      
      // If distance is too small and time is too short, treat as click, not drag
      if (distance < DRAG_THRESHOLD_DISTANCE && time < DRAG_THRESHOLD_TIME) {
        e.dataTransfer.dropEffect = 'none';
        return;
      }
    }
    
    e.dataTransfer.dropEffect = 'move';
    
    const draggedLayerName = draggedLayerNameRef.current;
    if (draggedLayerName === null) {
      return;
    }

    // Find current position of dragged layer
    const currentOrder = [...localStyles.layer_order];
    const currentDraggedIndex = currentOrder.indexOf(draggedLayerName);
    
    if (currentDraggedIndex === -1 || currentDraggedIndex === index) {
      if (lastDragOverIndexRef.current !== index) {
        setDragOverIndex(null);
        lastDragOverIndexRef.current = null;
      }
      return;
    }

    // Only update if the drag over index changed
    if (lastDragOverIndexRef.current !== index) {
      setDragOverIndex(index);
      lastDragOverIndexRef.current = index;

      // Real-time update: calculate new order based on drag position
      const newOrder = [...currentOrder];
      
      // Remove dragged item from its current position
      newOrder.splice(currentDraggedIndex, 1);
      
      // Calculate new position
      let newIndex = index;
      if (currentDraggedIndex < index) {
        // Dragging down, adjust index because we removed the item
        newIndex = index - 1;
      }
      
      // Insert at new position
      newOrder.splice(newIndex, 0, draggedLayerName);
      
      // Update layer_order in real-time
      setLocalStyles((prevStyles) => ({
        ...prevStyles,
        layer_order: newOrder,
      }));
      
      // Update draggedIndex to reflect the new position
      setDraggedIndex(newIndex);
    }
  };

  const handleDragLeave = () => {
    // Don't clear dragOverIndex here, as it might interfere with child elements
  };

  const handleDrop = (e: React.DragEvent) => {
    e.preventDefault();
    
    // Check if this was actually a drag or just a click
    const dragStart = dragStartPosRef.current;
    const originalOrder = originalLayerOrderRef.current;
    if (dragStart && originalOrder) {
      const distance = Math.sqrt(
        Math.pow(e.clientX - dragStart.x, 2) + Math.pow(e.clientY - dragStart.y, 2)
      );
      const time = Date.now() - dragStart.time;
      
      // If distance is too small and time is too short, treat as click
      if (distance < DRAG_THRESHOLD_DISTANCE && time < DRAG_THRESHOLD_TIME) {
        // Revert any changes made during the "drag"
        setLocalStyles((prevStyles) => ({
          ...prevStyles,
          layer_order: [...originalOrder],
        }));
      }
    }
    
    setDraggedIndex(null);
    setDragOverIndex(null);
    draggedLayerNameRef.current = null;
    lastDragOverIndexRef.current = null;
    dragStartPosRef.current = null;
    originalLayerOrderRef.current = null;
    isDraggingRef.current = false;
    
    // Reset opacity
    if (e.currentTarget instanceof HTMLElement) {
      e.currentTarget.style.opacity = '1';
    }
  };

  const handleDragEnd = (e: React.DragEvent) => {
    // Check if this was actually a drag or just a click
    const dragStart = dragStartPosRef.current;
    const originalOrder = originalLayerOrderRef.current;
    let wasClick = false;
    
    if (dragStart && originalOrder) {
      const distance = Math.sqrt(
        Math.pow(e.clientX - dragStart.x, 2) + Math.pow(e.clientY - dragStart.y, 2)
      );
      const time = Date.now() - dragStart.time;
      
      // If distance is too small and time is too short, treat as click
      if (distance < DRAG_THRESHOLD_DISTANCE && time < DRAG_THRESHOLD_TIME) {
        wasClick = true;
        // Revert any changes made during the "drag"
        setLocalStyles((prevStyles) => ({
          ...prevStyles,
          layer_order: [...originalOrder],
        }));
      }
    }
    
    setDraggedIndex(null);
    setDragOverIndex(null);
    draggedLayerNameRef.current = null;
    lastDragOverIndexRef.current = null;
    dragStartPosRef.current = null;
    originalLayerOrderRef.current = null;
    isDraggingRef.current = false;
    
    // Reset opacity
    if (e.currentTarget instanceof HTMLElement) {
      e.currentTarget.style.opacity = '1';
    }
    
    // If it was a click, trigger toggle after a short delay to avoid conflicts
    if (wasClick) {
      // Store the layer key to toggle after drag end
      const layerKey = (e.currentTarget as HTMLElement).closest('.layer-item')?.getAttribute('data-layer-key');
      if (layerKey) {
        setTimeout(() => {
          setExpandedLayer((prev) => prev === layerKey ? null : layerKey);
        }, 10);
      }
    }
  };

  const handleHeaderClick = (_e: React.MouseEvent, layerKey: string) => {
    // Only toggle if this is not part of a drag operation
    if (!isDraggingRef.current) {
      setExpandedLayer((prev) => prev === layerKey ? null : layerKey);
    }
  };

  const handleSave = async () => {
    try {
      await wasmAPI.fixCurrentStyle();
      // Reload styles from WASM to update fixedCurrentStyle
      const newStyles = await wasmAPI.getLayerStyles();
      const newStylesCopy = deepCopyLayerStyles(newStyles);
      setLocalStyles(newStylesCopy);
      setSavedStyles(newStylesCopy);
      onUpdate(newStylesCopy);
    } catch (error) {
      console.error('Failed to save style:', error);
      alert(`Failed to save style: ${error}`);
    }
  };

  const handleReset = async () => {
    if (fixedCurrentStyle) {
      // Reset current style to fixed style (get_current_fixed_style)
      try {
        await wasmAPI.resetCurrentStyle();
        // Reload styles from WASM
        const newStyles = await wasmAPI.getLayerStyles();
        const newStylesCopy = deepCopyLayerStyles(newStyles);
        setLocalStyles(newStylesCopy);
        setSavedStyles(newStylesCopy);
        onUpdate(newStylesCopy);
      } catch (error) {
        console.error('Failed to reset style:', error);
        // Fallback to fixed style
        const fixedStyleCopy = deepCopyLayerStyles(fixedCurrentStyle);
        setLocalStyles(fixedStyleCopy);
        setSavedStyles(fixedStyleCopy);
        onUpdate(fixedStyleCopy);
      }
    } else if (onReset) {
      onReset();
    }
  };

  const handleExport = async () => {
    try {
      const stylesState = await wasmAPI.getStylesState();
      
      if (!stylesState || typeof stylesState !== 'object' || !('current_name' in stylesState) || !('current' in stylesState)) {
        throw new Error('Failed to get styles state from WASM');
      }
      
      const currentName = (stylesState as any).current_name || 'style';
      const current = (stylesState as any).current;
      
      // Convert current to JSON string
      const jsonContent = JSON.stringify(current, null, 2);
      
      // Create a blob and download it
      const blob = new Blob([jsonContent], { type: 'application/json' });
      const url = URL.createObjectURL(blob);
      const link = document.createElement('a');
      link.href = url;
      link.download = `${currentName}.json`;
      document.body.appendChild(link);
      link.click();
      document.body.removeChild(link);
      URL.revokeObjectURL(url);
    } catch (error) {
      console.error('Failed to export style:', error);
      alert(`Failed to export style: ${error}`);
    }
  };

  // Layer metadata
  type LayerKey = 'instance' | 'device' | 'annotate' | 'pin' | 'wire' | 'text';
  const layerMetadata: Record<string, { key: LayerKey; label: string }> = {
    instance: { key: 'instance', label: 'Instance' },
    device: { key: 'device', label: 'Device' },
    annotate: { key: 'annotate', label: 'Annotate' },
    pin: { key: 'pin', label: 'Pin' },
    wire: { key: 'wire', label: 'Wire' },
    text: { key: 'text', label: 'Text' },
  };

  // Get layers in the order specified by layer_order
  const layers = localStyles.layer_order
    .map((layerName) => layerMetadata[layerName])
    .filter(Boolean) as Array<{ key: LayerKey; label: string }>;

  return (
    <div className="layer-styles-editor">
      {layers.map(({ key, label }, index) => {
        const layer = localStyles[key];
        const isExpanded = expandedLayer === key;
        const layerName = localStyles.layer_order[index];
        const canMoveUp = index > 0;
        const canMoveDown = index < layers.length - 1;
        const isDragging = draggedIndex === index;
        const isDragOver = dragOverIndex === index;

        return (
          <div
            key={key}
            className={`layer-item ${isDragging ? 'dragging' : ''} ${isDragOver ? 'drag-over' : ''}`}
            data-layer-key={key}
            draggable
            onDragStart={(e) => handleDragStart(e, index)}
            onDragOver={(e) => handleDragOver(e, index)}
            onDragLeave={handleDragLeave}
            onDrop={handleDrop}
            onDragEnd={handleDragEnd}
          >
            <div 
              className="layer-header"
              onClick={(e) => handleHeaderClick(e, key)}
            >
              <div className="layer-controls">
                <button
                  className="layer-order-btn"
                  onClick={(e) => {
                    e.stopPropagation();
                    moveLayer(layerName, 'up');
                  }}
                  disabled={!canMoveUp}
                  title="Move up"
                >
                  ↑
                </button>
                <button
                  className="layer-order-btn"
                  onClick={(e) => {
                    e.stopPropagation();
                    moveLayer(layerName, 'down');
                  }}
                  disabled={!canMoveDown}
                  title="Move down"
                >
                  ↓
                </button>
              </div>
              <span className="layer-name">
                {label}
              </span>
              <span className="expand-icon">
                {isExpanded ? '▼' : '▶'}
              </span>
            </div>
            {isExpanded && (
              <div className="layer-fields">
                {/* Text Group */}
                <div className="field-group">
                  <div className="field-group-title">Label</div>
                  <div className="field-group-content">
                    <div className="field">
                      <label>Font</label>
                      <FontInput
                        value={layer.font_family}
                        onChange={(value) => updateLayer(key, 'font_family', value)}
                        id={`font-input-${key}`}
                      />
                    </div>
                    <div className="field">
                      <label>Color</label>
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
                      <label>Scale (%)</label>
                      <input
                        type="number"
                        step="10"
                        min="0"
                        value={Math.round(layer.font_zoom * 100 * 100) / 100}
                        onChange={(e) => updateLayer(key, 'font_zoom', parseFloat(e.target.value) / 100)}
                      />
                    </div>
                  </div>
                </div>

                {/* Line Group */}
                <div className="field-group">
                  <div className="field-group-title">Shape</div>
                  <div className="field-group-content">
                    <div className="field">
                      <label>Color</label>
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
                      <label>Width</label>
                      <input
                        type="number"
                        step="0.1"
                        value={Math.round(layer.stroke_width * 100) / 100}
                        onChange={(e) => updateLayer(key, 'stroke_width', parseFloat(e.target.value))}
                      />
                    </div>
                  </div>
                </div>

                {/* Intersection Group (only for wire layer) */}
                {key === 'wire' && (
                  <div className="field-group">
                    <div className="field-group-title">Intersection</div>
                    <div className="field-group-content">
                      <div className="field">
                        <label>Scale (%)</label>
                        <input
                          type="number"
                          step="10"
                          min="0"
                          value={Math.round(localStyles.wire_intersection_scale * 100 * 100) / 100}
                          onChange={(e) => updateWireIntersectionScale(parseFloat(e.target.value) / 100)}
                        />
                      </div>
                      <div className="field">
                        <label>
                          <input
                            type="checkbox"
                            checked={localStyles.wire_show_intersection}
                            onChange={(e) => updateWireShowIntersection(e.target.checked)}
                          />
                          Show
                        </label>
                      </div>
                    </div>
                  </div>
                )}

                {/* Visible in Schematic */}
                <div className="field-group">
                  <div className="field-group-title"></div>
                  <div className="field-group-content">
                    <div className="field">
                      <label>
                        <input
                          type="checkbox"
                          checked={layer.sch_visible}
                          onChange={(e) => updateLayer(key, 'sch_visible', e.target.checked)}
                        />
                        Show in Schematic
                      </label>
                    </div>
                  </div>
                </div>
              </div>
            )}
          </div>
        );
      })}
      <div style={{ display: 'flex', gap: '8px', justifyContent: 'center'}}>
        {fixedCurrentStyle && (
          <button 
            className={`update-btn ${wasmDiffersFromFixed ? 'enabled' : 'disabled'}`}
            onClick={wasmDiffersFromFixed ? handleReset : undefined}
            disabled={!wasmDiffersFromFixed}
            title={wasmDiffersFromFixed ? 'Reset to last saved style and update' : 'WASM state already matches fixed style'}
            style={{
              background: '#F44336',
            }}
          >
            Reset
          </button>
        )}
        <button 
          className={`update-btn ${wasmDiffersFromFixed ? 'enabled' : 'disabled'}`}
          onClick={wasmDiffersFromFixed ? handleSave : undefined}
          disabled={!wasmDiffersFromFixed}
          title={wasmDiffersFromFixed ? 'Save current style' : 'Current style already matches saved style'}
          style={{
            background: '#28a745',
          }}
        >
          Save
        </button>
        <button 
          className="update-btn enabled"
          onClick={handleExport}
          title="Export current style as JSON file"
          style={{
            background: '#ffc107',
            color: '#212529',
          }}
        >
          Export
        </button>
      </div>
    </div>
  );
};

export default LayerStylesEditor;

