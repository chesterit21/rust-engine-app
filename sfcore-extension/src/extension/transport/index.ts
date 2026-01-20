import * as os from 'os';
import * as vscode from 'vscode';
import { ITransport } from './types';
import { UDSTransport } from './udsTransport';
import { HTTPTransport } from './httpTransport';
import { Logger } from '../utils/logger';

/**
 * Transport factory with auto-detect platform capability
 * Priority: UDS (Linux/Mac) → Named Pipes (Windows) → HTTP (Fallback)
 */
export class TransportFactory {
    static async create(): Promise<ITransport> {
        const config = vscode.workspace.getConfiguration('SFCoreAgent');
        const transportType = config.get<string>('transport.type', 'auto');

        if (transportType === 'http') {
            return await this.createHTTP();
        }

        if (transportType === 'uds' || transportType === 'auto') {
            // Try UDS first on Unix systems
            if (os.platform() !== 'win32') {
                try {
                    return await this.createUDS();
                } catch (error) {
                    Logger.warn('UDS failed, falling back to HTTP:', error);
                }
            }
        }

        // Fallback to HTTP
        return await this.createHTTP();
    }

    private static async createUDS(): Promise<ITransport> {
        const config = vscode.workspace.getConfiguration('SFCoreAgent');
        const socketPath = config.get<string>('transport.uds.socketPath', '/tmp/sfcore-ai.sock');

        const transport = new UDSTransport(socketPath);
        await transport.connect();
        return transport;
    }

    private static async createHTTP(): Promise<ITransport> {
        const config = vscode.workspace.getConfiguration('SFCoreAgent');
        const baseUrl = config.get<string>('transport.http.baseUrl', 'http://localhost:8080');

        const transport = new HTTPTransport(baseUrl);
        await transport.connect();
        return transport;
    }
}

export * from './types';
export { UDSTransport } from './udsTransport';
export { HTTPTransport } from './httpTransport';
