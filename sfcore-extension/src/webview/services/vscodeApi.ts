/**
 * VS Code API bridge for webview
 */

// Acquire VS Code API handle
const vscode = acquireVsCodeApi();

/**
 * VS Code API interface
 */
export const vscodeApi = {
    /**
     * Post message to extension host
     */
    postMessage(message: { type: string; payload: unknown }): void {
        vscode.postMessage(message);
    },

    /**
     * Get persisted state
     */
    getState<T>(): T | undefined {
        return vscode.getState() as T | undefined;
    },

    /**
     * Set persisted state
     */
    setState<T>(state: T): void {
        vscode.setState(state);
    },
};

/**
 * Declare VS Code API for TypeScript
 */
declare function acquireVsCodeApi(): {
    postMessage(message: unknown): void;
    getState(): unknown;
    setState(state: unknown): void;
};
