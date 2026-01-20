import * as vscode from 'vscode';
import { LLMService } from '../services/llmService';
import { ContextService, FileContext } from '../services/contextService';
import { MessageFromWebview, MessageToWebview } from '../../shared/protocol';
import { Logger } from '../utils/logger';

/**
 * Webview Provider for React UI
 */
export class WebviewProvider implements vscode.WebviewViewProvider {
    public static readonly viewType = 'SFCoreAgent.chatView';
    private view?: vscode.WebviewView;

    constructor(
        private readonly extensionUri: vscode.Uri,
        private readonly llmService: LLMService,
        private readonly contextService: ContextService
    ) {
        // Listen to context changes
        this.contextService.onContextChange((files) => {
            this.postMessage({
                type: 'contextUpdate',
                payload: files.map((f) => ({
                    name: f.name,
                    uri: f.uri.toString(),
                })),
            });
        });
    }

    public resolveWebviewView(
        webviewView: vscode.WebviewView,
        context: vscode.WebviewViewResolveContext,
        token: vscode.CancellationToken
    ): void {
        this.view = webviewView;

        webviewView.webview.options = {
            enableScripts: true,
            localResourceRoots: [this.extensionUri],
        };

        webviewView.webview.html = this.getHtmlForWebview(webviewView.webview);

        // Handle messages from webview
        webviewView.webview.onDidReceiveMessage(async (message: MessageFromWebview) => {
            await this.handleMessage(message);
        });
    }

    private async handleMessage(message: MessageFromWebview): Promise<void> {
        Logger.info(`[WebviewProvider] Received message from webview: ${message.type}`);
        
        switch (message.type) {
            case 'chat':
                Logger.info('[WebviewProvider] Processing chat message...');
                await this.handleChat(message.payload);
                break;
            case 'addFile':
                await this.handleAddFile(message.payload);
                break;
            case 'removeFile':
                await this.handleRemoveFile(message.payload);
                break;
            case 'clearContext':
                this.contextService.clearAll();
                break;
            case 'ready':
                Logger.info('[WebviewProvider] Webview is ready');
                break;
            default: {
                // Exhaustive check - if we get here, message is 'never'
                const _exhaustiveCheck: never = message;
                Logger.warn(`[WebviewProvider] Unknown message type: ${(_exhaustiveCheck as MessageFromWebview).type}`);
            }
        }
    }

    private async handleChat(payload: { messages: any[]; mode: string }): Promise<void> {
        const { messages, mode } = payload;
        const isSearch = mode === 'search';
        const contextFiles = this.contextService.getFiles();

        Logger.info(`[WebviewProvider] Chat request - Mode: ${mode}, Messages: ${messages.length}, Context files: ${contextFiles.length}`);
        Logger.info(`[WebviewProvider] Last message: ${JSON.stringify(messages[messages.length - 1])}`);

        try {
            this.postMessage({
                type: 'chatStart',
                payload: {},
            });
            Logger.info('[WebviewProvider] Sent chatStart to webview');

            const context = contextFiles.map((f) => `File: ${f.name}\n${f.content}`);

            Logger.info('[WebviewProvider] Calling LLMService.chat()...');
            await this.llmService.chat(messages, {
                isSearch,
                context,
                onStream: (chunk) => {
                    Logger.debug(`[WebviewProvider] Received chunk: ${chunk.length} chars`);
                    this.postMessage({
                        type: 'chatChunk',
                        payload: { content: chunk },
                    });
                },
            });

            Logger.info('[WebviewProvider] Chat completed, sending chatEnd');
            this.postMessage({
                type: 'chatEnd',
                payload: {},
            });
        } catch (error: any) {
            Logger.error('[WebviewProvider] Chat error:', error);
            this.postMessage({
                type: 'chatError',
                payload: { error: error.message },
            });
        }
    }

    private async handleAddFile(payload: { uri: string }): Promise<void> {
        const uri = vscode.Uri.parse(payload.uri);
        await this.contextService.addFile(uri);
    }

    private async handleRemoveFile(payload: { uri: string }): Promise<void> {
        const uri = vscode.Uri.parse(payload.uri);
        this.contextService.removeFile(uri);
    }

    private postMessage(message: MessageToWebview): void {
        this.view?.webview.postMessage(message);
    }

    public show(): void {
        if (this.view) {
            this.view.show(true);
        }
    }

    public notifyContextUpdate(): void {
        const files = this.contextService.getFiles();
        this.postMessage({
            type: 'contextUpdate',
            payload: files.map((f) => ({
                name: f.name,
                uri: f.uri.toString(),
            })),
        });
    }

    private getHtmlForWebview(webview: vscode.Webview): string {
        const scriptUri = webview.asWebviewUri(
            vscode.Uri.joinPath(this.extensionUri, 'dist', 'webview.js')
        );
        const styleUri = webview.asWebviewUri(
            vscode.Uri.joinPath(this.extensionUri, 'dist', 'webview.css')
        );

        const nonce = this.getNonce();

        return `<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <meta http-equiv="Content-Security-Policy" content="default-src 'none'; style-src ${webview.cspSource} 'unsafe-inline'; script-src 'nonce-${nonce}';">
    <link href="${styleUri}" rel="stylesheet">
    <title>AI Dev Agent</title>
</head>
<body>
    <div id="root"></div>
    <script nonce="${nonce}" src="${scriptUri}"></script>
</body>
</html>`;
    }

    private getNonce(): string {
        let text = '';
        const possible = 'ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789';
        for (let i = 0; i < 32; i++) {
            text += possible.charAt(Math.floor(Math.random() * possible.length));
        }
        return text;
    }
}
