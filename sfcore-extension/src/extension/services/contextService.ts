import * as vscode from 'vscode';
import * as path from 'path';
import { Logger } from '../utils/logger';

/**
 * File context structure
 */
export interface FileContext {
    uri: vscode.Uri;
    name: string;
    content: string;
    language: string;
}

/**
 * Context Service for file management
 */
export class ContextService {
    private fileContexts: Map<string, FileContext>;
    private onContextChangeEmitter: vscode.EventEmitter<FileContext[]>;
    public readonly onContextChange: vscode.Event<FileContext[]>;

    constructor() {
        this.fileContexts = new Map();
        this.onContextChangeEmitter = new vscode.EventEmitter();
        this.onContextChange = this.onContextChangeEmitter.event;
    }

    /**
     * Add file to context
     */
    async addFile(uri: vscode.Uri): Promise<void> {
        try {
            const document = await vscode.workspace.openTextDocument(uri);
            const content = document.getText();
            const name = path.basename(uri.fsPath);
            const language = document.languageId;

            const context: FileContext = {
                uri,
                name,
                content,
                language,
            };

            this.fileContexts.set(uri.toString(), context);
            this.notifyChange();

            Logger.info(`Added file to context: ${name}`);
        } catch (error) {
            Logger.error('Failed to add file to context:', error);
            throw error;
        }
    }

    /**
     * Remove file from context
     */
    removeFile(uri: vscode.Uri): void {
        const key = uri.toString();
        if (this.fileContexts.delete(key)) {
            this.notifyChange();
            Logger.info(`Removed file from context: ${path.basename(uri.fsPath)}`);
        }
    }

    /**
     * Get all files in context
     */
    getFiles(): FileContext[] {
        return Array.from(this.fileContexts.values());
    }

    /**
     * Get file content by URI
     */
    getFileContent(uri: vscode.Uri): string | undefined {
        return this.fileContexts.get(uri.toString())?.content;
    }

    /**
     * Clear all files from context
     */
    clearAll(): void {
        this.fileContexts.clear();
        this.notifyChange();
    }

    /**
     * Get formatted context string
     */
    getContextString(): string {
        return this.getFiles()
            .map((file) => `\n--- ${file.name} ---\n${file.content}`)
            .join('\n\n');
    }

    private notifyChange(): void {
        this.onContextChangeEmitter.fire(this.getFiles());
    }

    dispose(): void {
        this.fileContexts.clear();
        this.onContextChangeEmitter.dispose();
    }
}
