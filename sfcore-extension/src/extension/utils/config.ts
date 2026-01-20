import * as vscode from 'vscode';
import { ExtensionConfig, TransportType } from '../../shared/types';

/**
 * Configuration utility
 */
export function getConfig(): ExtensionConfig {
    const config = vscode.workspace.getConfiguration('SFCoreAgent');

    return {
        transport: {
            type: config.get<TransportType>('transport.type', 'auto'),
            uds: {
                socketPath: config.get<string>('transport.uds.socketPath', '/tmp/llm-server.sock'),
            },
            http: {
                baseUrl: config.get<string>('transport.http.baseUrl', 'http://localhost:8080'),
            },
        },
    };
}

/**
 * Update configuration
 */
export async function updateConfig<T>(key: string, value: T): Promise<void> {
    const config = vscode.workspace.getConfiguration('SFCoreAgent');
    await config.update(key, value, vscode.ConfigurationTarget.Global);
}
