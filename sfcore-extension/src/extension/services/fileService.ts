import * as vscode from 'vscode';
import * as fs from 'fs';
import * as path from 'path';
import { Logger } from '../utils/logger';

/**
 * File Service for file operations
 */
export class FileService {
    /**
     * Read file content
     */
    async readFile(uri: vscode.Uri): Promise<string> {
        try {
            const document = await vscode.workspace.openTextDocument(uri);
            return document.getText();
        } catch (error) {
            Logger.error('Failed to read file:', error);
            throw error;
        }
    }

    /**
     * Write content to file
     */
    async writeFile(uri: vscode.Uri, content: string): Promise<void> {
        try {
            const edit = new vscode.WorkspaceEdit();
            const document = await vscode.workspace.openTextDocument(uri);
            const fullRange = new vscode.Range(
                document.positionAt(0),
                document.positionAt(document.getText().length)
            );
            edit.replace(uri, fullRange, content);
            await vscode.workspace.applyEdit(edit);
            await document.save();
            Logger.info(`Written to file: ${path.basename(uri.fsPath)}`);
        } catch (error) {
            Logger.error('Failed to write file:', error);
            throw error;
        }
    }

    /**
     * Create new file
     */
    async createFile(uri: vscode.Uri, content: string = ''): Promise<void> {
        try {
            const edit = new vscode.WorkspaceEdit();
            edit.createFile(uri, { overwrite: false });
            await vscode.workspace.applyEdit(edit);

            if (content) {
                await this.writeFile(uri, content);
            }

            Logger.info(`Created file: ${path.basename(uri.fsPath)}`);
        } catch (error) {
            Logger.error('Failed to create file:', error);
            throw error;
        }
    }

    /**
     * Get file info
     */
    async getFileInfo(
        uri: vscode.Uri
    ): Promise<{ name: string; size: number; language: string } | null> {
        try {
            const document = await vscode.workspace.openTextDocument(uri);
            const stat = await vscode.workspace.fs.stat(uri);

            return {
                name: path.basename(uri.fsPath),
                size: stat.size,
                language: document.languageId,
            };
        } catch (error) {
            Logger.error('Failed to get file info:', error);
            return null;
        }
    }

    /**
     * Check if file exists
     */
    async exists(uri: vscode.Uri): Promise<boolean> {
        try {
            await vscode.workspace.fs.stat(uri);
            return true;
        } catch {
            return false;
        }
    }
}
