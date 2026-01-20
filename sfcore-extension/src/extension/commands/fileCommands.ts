import * as vscode from 'vscode';
import { ContextService } from '../services/contextService';
import { WebviewProvider } from '../providers/webviewProvider';
import { Logger } from '../utils/logger';

/**
 * Register file context commands
 */
export function registerFileCommands(
    context: vscode.ExtensionContext,
    contextService: ContextService,
    webviewProvider: WebviewProvider
): void {
    // Add file to context command
    context.subscriptions.push(
        vscode.commands.registerCommand('SFCoreAgent.addFileToContext', async (uri?: vscode.Uri) => {
            const fileUri =
                uri ||
                (await vscode.window
                    .showOpenDialog({
                        canSelectMany: false,
                        openLabel: 'Add to AI Context',
                    })
                    .then((uris) => uris?.[0]));

            if (fileUri) {
                await contextService.addFile(fileUri);
                webviewProvider.notifyContextUpdate();
                vscode.window.showInformationMessage(`Added ${fileUri.fsPath} to context`);
            }
        })
    );

    // Remove file from context
    context.subscriptions.push(
        vscode.commands.registerCommand('SFCoreAgent.removeFileFromContext', async (uri: vscode.Uri) => {
            if (uri) {
                contextService.removeFile(uri);
                webviewProvider.notifyContextUpdate();
            }
        })
    );

    // Clear all context
    context.subscriptions.push(
        vscode.commands.registerCommand('SFCoreAgent.clearContext', () => {
            contextService.clearAll();
            webviewProvider.notifyContextUpdate();
            vscode.window.showInformationMessage('Context cleared');
            Logger.info('Context cleared');
        })
    );

    // Add current file to context
    context.subscriptions.push(
        vscode.commands.registerCommand('SFCoreAgent.addCurrentFileToContext', async () => {
            const editor = vscode.window.activeTextEditor;
            if (editor) {
                await contextService.addFile(editor.document.uri);
                webviewProvider.notifyContextUpdate();
                vscode.window.showInformationMessage('Current file added to context');
            } else {
                vscode.window.showWarningMessage('No active file');
            }
        })
    );
}
