// WASM bindings
// @ts-ignore
import init, * as wasm from '../pkg/drawckt_web.js';

let wasmInitialized = false;

export async function initWasm() {
  if (!wasmInitialized) {
    await init();
    wasmInitialized = true;
  }
}

export interface SymbolInfo {
  lib: string;
  cell: string;
}

export interface ProcessResult {
  success: boolean;
  symbol_count: number;
  schematic_rendered: boolean;
  schematic_restored?: boolean;
  message?: string;
}

export interface LayerStyle {
  stroke_color: string;
  stroke_width: number;
  text_color: string;
  font_family: string;
  font_zoom: number;
  priority: number;
  sch_visible: boolean;
}

export interface LayerStyles {
  layer_order: string[];
  instance: LayerStyle;
  device: LayerStyle;
  annotate: LayerStyle;
  pin: LayerStyle;
  wire: LayerStyle;
  wire_show_intersection: boolean;
  wire_intersection_scale: number;
  text: LayerStyle;
}

export const wasmAPI = {
  async processSchematicJson(jsonStr: string, filename?: string): Promise<ProcessResult> {
    await initWasm();
    try {
      let result;
      if (filename) {
        result = wasm.process_schematic_json_with_filename(jsonStr, filename);
      } else {
        result = wasm.process_schematic_json(jsonStr);
      }
      // serde_wasm_bindgen::to_value returns a JavaScript object or Map
      // Convert Map to plain object if needed
      let processResult: ProcessResult;
      if (result instanceof Map) {
        processResult = {
          success: result.get('success') ?? false,
          symbol_count: result.get('symbol_count') ?? 0,
          schematic_rendered: result.get('schematic_rendered') ?? false,
          message: result.get('message') ?? undefined,
        };
      } else {
        processResult = result as unknown as ProcessResult;
      }
      return processResult;
    } catch (error) {
      const msg =
        error instanceof Error
          ? error.message
          : typeof error === 'string'
            ? error
            : JSON.stringify(error);
      // Keep the Rust/wasm error message as-is so UI can display it.
      throw new Error(msg);
    }
  },

  async processSchematicZip(base64Zip: string, filename?: string): Promise<ProcessResult> {
    await initWasm();
    try {
      let result;
      // @ts-ignore - process_schematic_zip functions are defined in Rust but TypeScript types may not be updated yet
      if (filename) {
        result = (wasm as any).process_schematic_zip_with_filename(base64Zip, filename);
      } else {
        result = (wasm as any).process_schematic_zip(base64Zip);
      }
      // serde_wasm_bindgen::to_value returns a JavaScript object or Map
      // Convert Map to plain object if needed
      let processResult: ProcessResult;
      if (result instanceof Map) {
        processResult = {
          success: result.get('success') ?? false,
          symbol_count: result.get('symbol_count') ?? 0,
          schematic_rendered: result.get('schematic_restored') ?? false,
          schematic_restored: result.get('schematic_restored') ?? false,
          message: result.get('message') ?? undefined,
        };
      } else {
        const obj = result as any;
        processResult = {
          success: Boolean(obj.success ?? false),
          symbol_count: Number(obj.symbol_count ?? 0),
          schematic_rendered: Boolean(obj.schematic_restored ?? false),
          schematic_restored: Boolean(obj.schematic_restored ?? false),
          message: obj.message ?? undefined,
        };
      }
      return processResult;
    } catch (error) {
      const msg =
        error instanceof Error
          ? error.message
          : typeof error === 'string'
            ? error
            : JSON.stringify(error);
      // Keep the Rust/wasm error message as-is so UI can display it.
      throw new Error(msg);
    }
  },

  async getSymbolContent(lib: string, cell: string): Promise<string> {
    await initWasm();
    try {
      const result = wasm.get_symbol_content(lib, cell);
      return result as unknown as string;
    } catch (error) {
      throw new Error(`Failed to get symbol content: ${error}`);
    }
  },

  async getSchematicContent(): Promise<string> {
    await initWasm();
    try {
      const result = wasm.get_schematic_content();
      return result as unknown as string;
    } catch (error) {
      throw new Error(`Failed to get schematic content: ${error}`);
    }
  },

  async getAllSymbols(): Promise<SymbolInfo[]> {
    await initWasm();
    try {
      const result = wasm.get_all_symbols();
      
      // serde_wasm_bindgen::to_value returns a JavaScript array of objects
      if (!Array.isArray(result)) {
        console.error('getAllSymbols: result is not an array', result);
        return [];
      }
      
      // Convert to SymbolInfo array - now using proper struct serialization
      const symbols = result as unknown as SymbolInfo[];
      
      // Filter out any symbols with missing or invalid lib/cell (shouldn't happen, but safety check)
      const validSymbols = symbols.filter((s) => {
        return s && 
               typeof s === 'object' && 
               'lib' in s && 
               'cell' in s &&
               typeof s.lib === 'string' && 
               typeof s.cell === 'string' &&
               s.lib.length > 0 && 
               s.cell.length > 0;
      });
      
      if (validSymbols.length !== symbols.length) {
        console.warn(`Filtered out ${symbols.length - validSymbols.length} invalid symbols out of ${symbols.length} total`);
      }
      
      return validSymbols;
    } catch (error) {
      throw new Error(`Failed to get all symbols: ${error}`);
    }
  },

  async updateLayerStyles(stylesJson: string): Promise<{ success: boolean; only_sch_visible_changed: boolean }> {
    await initWasm();
    try {
      const result = wasm.update_layer_styles(stylesJson);
      // Handle different return types (Map or object)
      let updateResult: { success: boolean; only_sch_visible_changed: boolean };
      if (result instanceof Map) {
        updateResult = {
          success: result.get('success') ?? false,
          only_sch_visible_changed: result.get('only_sch_visible_changed') ?? false,
        };
      } else if (result && typeof result === 'object') {
        const obj = result as any;
        updateResult = {
          success: Boolean(obj.success ?? false),
          only_sch_visible_changed: Boolean(obj.only_sch_visible_changed ?? false),
        };
      } else {
        throw new Error(`Unexpected result type: ${typeof result}`);
      }
      return updateResult;
    } catch (error) {
      throw new Error(`Failed to update layer styles: ${error}`);
    }
  },

  async getLayerStyles(): Promise<LayerStyles> {
    await initWasm();
    try {
      const result = wasm.get_layer_styles();
      // serde_wasm_bindgen::to_value returns a JavaScript object
      return result as unknown as LayerStyles;
    } catch (error) {
      throw new Error(`Failed to get layer styles: ${error}`);
    }
  },

  async getDefaultLayerStyles(): Promise<LayerStyles> {
    await initWasm();
    try {
      // @ts-ignore - get_default_layer_styles is defined in Rust but TypeScript types may not be updated yet
      const result = (wasm as any).get_default_layer_styles();
      // serde_wasm_bindgen::to_value returns a JavaScript object
      return result as unknown as LayerStyles;
    } catch (error) {
      throw new Error(`Failed to get default layer styles: ${error}`);
    }
  },

  async routeEmbedded(path: string): Promise<string> {
    await initWasm();
    try {
      const result = wasm.route_embedded(path);
      return result as unknown as string;
    } catch (error) {
      throw new Error(`Failed to route embedded file: ${error}`);
    }
  },

  async updateSymbolContent(lib: string, cell: string, content: string): Promise<{ success: boolean; message: string }> {
    await initWasm();
    try {
      const result = wasm.update_symbol_content(lib, cell, content);
      return result as unknown as { success: boolean; message: string };
    } catch (error) {
      throw new Error(`Failed to update symbol content: ${error}`);
    }
  },

  async undoSymbol(lib: string, cell: string): Promise<{ success: boolean; message: string }> {
    await initWasm();
    try {
      const result = wasm.undo_symbol(lib, cell);
      return result as unknown as { success: boolean; message: string };
    } catch (error) {
      throw new Error(`Failed to undo symbol: ${error}`);
    }
  },

  async redoSymbol(lib: string, cell: string): Promise<{ success: boolean; message: string }> {
    await initWasm();
    try {
      const result = wasm.redo_symbol(lib, cell);
      return result as unknown as { success: boolean; message: string };
    } catch (error) {
      throw new Error(`Failed to redo symbol: ${error}`);
    }
  },

  async getSymbolInfo(lib: string, cell: string): Promise<{ lib: string; cell: string; current_idx: number; hist_len: number; can_undo: boolean; can_redo: boolean }> {
    await initWasm();
    try {
      const result = wasm.get_symbol_info(lib, cell);
      // Handle different return types (Map or object)
      let symbolInfo: { lib: string; cell: string; current_idx: number; hist_len: number; can_undo: boolean; can_redo: boolean };
      if (result instanceof Map) {
        symbolInfo = {
          lib: String(result.get('lib') ?? ''),
          cell: String(result.get('cell') ?? ''),
          current_idx: Number(result.get('current_idx') ?? 0),
          hist_len: Number(result.get('hist_len') ?? 0),
          can_undo: Boolean(result.get('can_undo') ?? false),
          can_redo: Boolean(result.get('can_redo') ?? false),
        };
      } else if (result && typeof result === 'object') {
        // Try to access properties directly
        const obj = result as any;
        symbolInfo = {
          lib: String(obj.lib ?? ''),
          cell: String(obj.cell ?? ''),
          current_idx: Number(obj.current_idx ?? 0),
          hist_len: Number(obj.hist_len ?? 0),
          can_undo: Boolean(obj.can_undo ?? false),
          can_redo: Boolean(obj.can_redo ?? false),
        };
      } else {
        throw new Error(`Unexpected result type: ${typeof result}`);
      }
      return symbolInfo;
    } catch (error) {
      console.error('Error in getSymbolInfo:', error);
      throw new Error(`Failed to get symbol info: ${error}`);
    }
  },

  async exportAllFiles(): Promise<{ success: boolean; filename: string; data: string }> {
    await initWasm();
    try {
      const result = wasm.export_all_files();
      
      // Handle different return types (Map or object)
      let exportResult: { success: boolean; filename: string; data: string };
      if (result instanceof Map) {
        exportResult = {
          success: result.get('success') ?? false,
          filename: String(result.get('filename') ?? ''),
          data: String(result.get('data') ?? ''),
        };
      } else if (result && typeof result === 'object') {
        const obj = result as any;
        exportResult = {
          success: Boolean(obj.success ?? false),
          filename: String(obj.filename ?? ''),
          data: String(obj.data ?? ''),
        };
      } else {
        throw new Error(`Unexpected result type: ${typeof result}`);
      }
      
      return exportResult;
    } catch (error) {
      console.error('Error in exportAllFiles:', error);
      throw new Error(`Failed to export files: ${error}`);
    }
  },

  async getDemoList(): Promise<string[]> {
    await initWasm();
    try {
      const result = wasm.get_demo_list();
      if (!Array.isArray(result)) {
        console.error('getDemoList: result is not an array', result);
        return [];
      }
      return result as unknown as string[];
    } catch (error) {
      throw new Error(`Failed to get demo list: ${error}`);
    }
  },

  async loadDemo(name: string): Promise<string> {
    await initWasm();
    try {
      const result = wasm.load_demo(name);
      return result as unknown as string;
    } catch (error) {
      throw new Error(`Failed to load demo: ${error}`);
    }
  },

  async updateSchematicContent(content: string): Promise<{ success: boolean; message: string }> {
    await initWasm();
    try {
      const result = wasm.update_schematic_content(content);
      return result as unknown as { success: boolean; message: string };
    } catch (error) {
      throw new Error(`Failed to update schematic content: ${error}`);
    }
  },

  async undoSchematic(): Promise<{ success: boolean; message: string }> {
    await initWasm();
    try {
      const result = wasm.undo_schematic();
      return result as unknown as { success: boolean; message: string };
    } catch (error) {
      throw new Error(`Failed to undo schematic: ${error}`);
    }
  },

  async redoSchematic(): Promise<{ success: boolean; message: string }> {
    await initWasm();
    try {
      const result = wasm.redo_schematic();
      return result as unknown as { success: boolean; message: string };
    } catch (error) {
      throw new Error(`Failed to redo schematic: ${error}`);
    }
  },

  async getSchematicInfo(): Promise<{ current_idx: number; hist_len: number; can_undo: boolean; can_redo: boolean }> {
    await initWasm();
    try {
      const result = wasm.get_schematic_info();
      // Handle different return types (Map or object)
      let schematicInfo: { current_idx: number; hist_len: number; can_undo: boolean; can_redo: boolean };
      if (result instanceof Map) {
        schematicInfo = {
          current_idx: Number(result.get('current_idx') ?? 0),
          hist_len: Number(result.get('hist_len') ?? 0),
          can_undo: Boolean(result.get('can_undo') ?? false),
          can_redo: Boolean(result.get('can_redo') ?? false),
        };
      } else if (result && typeof result === 'object') {
        const obj = result as any;
        schematicInfo = {
          current_idx: Number(obj.current_idx ?? 0),
          hist_len: Number(obj.hist_len ?? 0),
          can_undo: Boolean(obj.can_undo ?? false),
          can_redo: Boolean(obj.can_redo ?? false),
        };
      } else {
        throw new Error(`Unexpected result type: ${typeof result}`);
      }
      return schematicInfo;
    } catch (error) {
      console.error('Error in getSchematicInfo:', error);
      throw new Error(`Failed to get schematic info: ${error}`);
    }
  },
};

