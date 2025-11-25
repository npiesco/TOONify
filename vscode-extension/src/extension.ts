// TOONify VS Code Extension
// Main extension entry point

import * as vscode from 'vscode';
import * as path from 'path';
import * as fs from 'fs';

let toonifyModule: any = null;
let cachedConverter: any = null;

/**
 * Load the WASM module from the pkg directory
 */
async function loadWasmModule(): Promise<any> {
    if (toonifyModule) {
        return toonifyModule;
    }

    try {
        // Try to load from parent directory's pkg folder
        const pkgPath = path.join(__dirname, '..', '..', 'pkg');
        const wasmPath = path.join(pkgPath, 'toonify_bg.wasm');
        
        // Check if WASM file exists
        if (!fs.existsSync(wasmPath)) {
            throw new Error(`WASM file not found at ${wasmPath}`);
        }

        // Import the WASM module
        const wasmModule = await import(path.join(pkgPath, 'toonify.js'));
        
        // Initialize with the WASM binary
        const wasmBytes = fs.readFileSync(wasmPath);
        await wasmModule.default(wasmBytes);
        
        toonifyModule = wasmModule;
        
        // Initialize cached converter (100 max entries for editor session)
        cachedConverter = new wasmModule.WasmCachedConverter(100);
        
        return toonifyModule;
    } catch (error) {
        vscode.window.showErrorMessage(`Failed to load TOONify WASM module: ${error}`);
        throw error;
    }
}

/**
 * Convert JSON to TOON (using cached converter)
 */
async function jsonToToon(jsonText: string): Promise<string> {
    await loadWasmModule();
    if (cachedConverter) {
        return cachedConverter.jsonToToon(jsonText);
    }
    // Fallback to non-cached if converter not initialized
    return toonifyModule.json_to_toon(jsonText);
}

/**
 * Convert TOON to JSON (using cached converter)
 */
async function toonToJson(toonText: string): Promise<string> {
    await loadWasmModule();
    if (cachedConverter) {
        return cachedConverter.toonToJson(toonText);
    }
    // Fallback to non-cached if converter not initialized
    return toonifyModule.toon_to_json(toonText);
}

/**
 * Get the selected text or entire document
 */
function getTextToConvert(editor: vscode.TextEditor): { text: string; range: vscode.Range } {
    const selection = editor.selection;
    
    if (!selection.isEmpty) {
        // Use selected text
        return {
            text: editor.document.getText(selection),
            range: selection
        };
    } else {
        // Use entire document
        const fullRange = new vscode.Range(
            editor.document.positionAt(0),
            editor.document.positionAt(editor.document.getText().length)
        );
        return {
            text: editor.document.getText(),
            range: fullRange
        };
    }
}

/**
 * Command: Convert JSON to TOON
 */
async function convertJsonToToonCommand() {
    const editor = vscode.window.activeTextEditor;
    if (!editor) {
        vscode.window.showErrorMessage('No active editor');
        return;
    }

    try {
        const { text, range } = getTextToConvert(editor);
        
        // Validate JSON
        try {
            JSON.parse(text);
        } catch (e) {
            vscode.window.showErrorMessage('Invalid JSON: ' + e);
            return;
        }

        // Convert to TOON
        const toonResult = await jsonToToon(text);
        
        // Replace text in editor
        await editor.edit(editBuilder => {
            editBuilder.replace(range, toonResult);
        });

        vscode.window.showInformationMessage('✓ Converted JSON to TOON');
    } catch (error) {
        vscode.window.showErrorMessage(`Conversion failed: ${error}`);
    }
}

/**
 * Command: Convert TOON to JSON
 */
async function convertToonToJsonCommand() {
    const editor = vscode.window.activeTextEditor;
    if (!editor) {
        vscode.window.showErrorMessage('No active editor');
        return;
    }

    try {
        const { text, range } = getTextToConvert(editor);

        // Convert to JSON
        const jsonResult = await toonToJson(text);
        
        // Format JSON for readability
        const formattedJson = JSON.stringify(JSON.parse(jsonResult), null, 2);
        
        // Replace text in editor
        await editor.edit(editBuilder => {
            editBuilder.replace(range, formattedJson);
        });

        vscode.window.showInformationMessage('✓ Converted TOON to JSON');
    } catch (error) {
        vscode.window.showErrorMessage(`Conversion failed: ${error}`);
    }
}

/**
 * Command: Validate TOON syntax
 */
async function validateToonCommand() {
    const editor = vscode.window.activeTextEditor;
    if (!editor) {
        vscode.window.showErrorMessage('No active editor');
        return;
    }

    try {
        const { text } = getTextToConvert(editor);

        // Try to convert TOON to JSON (validates syntax)
        await toonToJson(text);
        
        vscode.window.showInformationMessage('✓ TOON syntax is valid');
    } catch (error) {
        vscode.window.showErrorMessage(`TOON syntax error: ${error}`);
    }
}

/**
 * Command: Format TOON (roundtrip through JSON)
 */
async function formatToonCommand() {
    const editor = vscode.window.activeTextEditor;
    if (!editor) {
        vscode.window.showErrorMessage('No active editor');
        return;
    }

    try {
        const { text, range } = getTextToConvert(editor);

        // Convert TOON → JSON → TOON for formatting
        const jsonResult = await toonToJson(text);
        const formattedToon = await jsonToToon(jsonResult);
        
        // Replace text in editor
        await editor.edit(editBuilder => {
            editBuilder.replace(range, formattedToon);
        });

        vscode.window.showInformationMessage('✓ TOON formatted');
    } catch (error) {
        vscode.window.showErrorMessage(`Format failed: ${error}`);
    }
}

/**
 * Command: Show cache statistics
 */
async function showCacheStatsCommand() {
    try {
        await loadWasmModule();
        
        if (cachedConverter) {
            const stats = cachedConverter.cacheStats();
            const statsObj = JSON.parse(stats);
            
            const message = `Cache Statistics:\n` +
                          `  Entries: ${statsObj.entries}/${statsObj.maxSize}\n` +
                          `  Hit Rate: Better performance on repeated conversions`;
            
            vscode.window.showInformationMessage(message);
        } else {
            vscode.window.showWarningMessage('Cache not initialized');
        }
    } catch (error) {
        vscode.window.showErrorMessage(`Failed to get cache stats: ${error}`);
    }
}

/**
 * Command: Clear cache
 */
async function clearCacheCommand() {
    try {
        await loadWasmModule();
        
        if (cachedConverter) {
            cachedConverter.clearCache();
            vscode.window.showInformationMessage('✓ Cache cleared');
        } else {
            vscode.window.showWarningMessage('Cache not initialized');
        }
    } catch (error) {
        vscode.window.showErrorMessage(`Failed to clear cache: ${error}`);
    }
}

/**
 * Extension activation
 */
export function activate(context: vscode.ExtensionContext) {
    console.log('TOONify extension is now active');

    // Preload WASM module
    loadWasmModule().catch(err => {
        console.error('Failed to preload WASM module:', err);
    });

    // Register commands
    context.subscriptions.push(
        vscode.commands.registerCommand('toonify.jsonToToon', convertJsonToToonCommand)
    );

    context.subscriptions.push(
        vscode.commands.registerCommand('toonify.toonToJson', convertToonToJsonCommand)
    );

    context.subscriptions.push(
        vscode.commands.registerCommand('toonify.validateToon', validateToonCommand)
    );

    context.subscriptions.push(
        vscode.commands.registerCommand('toonify.formatToon', formatToonCommand)
    );

    context.subscriptions.push(
        vscode.commands.registerCommand('toonify.showCacheStats', showCacheStatsCommand)
    );

    context.subscriptions.push(
        vscode.commands.registerCommand('toonify.clearCache', clearCacheCommand)
    );

    // Show welcome message
    vscode.window.showInformationMessage('TOONify extension activated! Cached converter ready (100 entries). Use Cmd+Alt+T to convert.');
}

/**
 * Extension deactivation
 */
export function deactivate() {
    console.log('TOONify extension deactivated');
}

