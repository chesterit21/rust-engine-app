import * as vscode from 'vscode';
import { WebviewProvider } from '../providers/webviewProvider';
import { ContextService } from '../services/contextService';
import { LLMService } from '../services/llmService';
import { Logger } from '../utils/logger';

/**
 * Register chat-related commands
 */
export function registerChatCommands(
    context: vscode.ExtensionContext,
    webviewProvider: WebviewProvider,
    llmService: LLMService
): void {
    // Open chat command
    context.subscriptions.push(
        vscode.commands.registerCommand('SFCoreAgent.openChat', () => {
            webviewProvider.show();
            Logger.info('Chat opened');
        })
    );

    // Cancel current request
    context.subscriptions.push(
        vscode.commands.registerCommand('SFCoreAgent.cancelRequest', async () => {
            try {
                await llmService.cancelRequest();
                vscode.window.showInformationMessage('Request cancelled');
                Logger.info('Request cancelled');
            } catch (error: any) {
                vscode.window.showErrorMessage(`Failed to cancel: ${error.message}`);
                Logger.error('Cancel failed:', error);
            }
        })
    );
}
