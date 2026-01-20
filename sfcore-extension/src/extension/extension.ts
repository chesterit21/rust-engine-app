import * as vscode from 'vscode';
import { WebviewProvider } from './providers/webviewProvider';
import { LLMService } from './services/llmService';
import { ContextService } from './services/contextService';
import { TransportFactory } from './transport';
import { Logger } from './utils/logger';
import { registerChatCommands, registerFileCommands } from './commands';

let llmService: LLMService | undefined;
let contextService: ContextService | undefined;
let webviewProvider: WebviewProvider | undefined;

/**
 * Extension activation
 */
export async function activate(context: vscode.ExtensionContext): Promise<void> {
    Logger.initialize(context);
    Logger.info('AI Dev Agent Extension activating...');

    try {
        // Initialize transport with fallback
        let transport;
        try {
            transport = await TransportFactory.create();
        } catch (error) {
            Logger.warn('Failed to create transport, using mock transport:', error);
            // Create a mock transport for development when server is not running
            transport = {
                connect: async () => { },
                send: async () => ({ type: 'response', payload: { content: 'Server not available' } }),
                sendStream: async () => { },
                dispose: () => { },
            };
        }

        // Initialize services
        llmService = new LLMService(transport);
        contextService = new ContextService();

        // Initialize webview provider
        webviewProvider = new WebviewProvider(context.extensionUri, llmService, contextService);

        // Register webview provider
        context.subscriptions.push(
            vscode.window.registerWebviewViewProvider(WebviewProvider.viewType, webviewProvider, {
                webviewOptions: {
                    retainContextWhenHidden: true,
                },
            })
        );

        // Register commands
        registerChatCommands(context, webviewProvider, llmService);
        registerFileCommands(context, contextService, webviewProvider);

        Logger.info('AI Dev Agent Extension activated successfully');
        vscode.window.showInformationMessage('AI Dev Agent activated');
    } catch (error: any) {
        Logger.error('Failed to activate extension:', error);
        vscode.window.showErrorMessage(`AI Dev Agent activation failed: ${error.message}`);
    }
}

/**
 * Extension deactivation
 */
export function deactivate(): void {
    llmService?.dispose();
    contextService?.dispose();
    Logger.info('AI Dev Agent Extension deactivated');
}
