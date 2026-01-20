import * as net from 'net';
import { ITransport, TransportMessage, TransportResponse } from './types';
import { Logger } from '../utils/logger';

/**
 * Unix Domain Socket Transport implementation using NDJSON protocol
 * Compatible with sfcore-ai-server
 */
export class UDSTransport implements ITransport {
    private socket: net.Socket | null = null;
    private socketPath: string;
    private reconnectTimer: NodeJS.Timeout | null = null;
    private buffer: string = '';
    private pendingResolve: ((response: TransportResponse) => void) | null = null;
    private pendingReject: ((error: Error) => void) | null = null;
    private streamCallback: ((chunk: string) => void) | null = null;

    constructor(socketPath: string = '/tmp/sfcore-ai.sock') {
        this.socketPath = socketPath;
    }

    async connect(): Promise<void> {
        return new Promise((resolve, reject) => {
            Logger.info(`[UDSTransport] Connecting to: ${this.socketPath}`);
            
            this.socket = net.createConnection(this.socketPath, () => {
                Logger.info(`[UDSTransport] Connected successfully to: ${this.socketPath}`);
                resolve();
            });

            this.socket.setEncoding('utf8');

            this.socket.on('error', (error) => {
                Logger.error('[UDSTransport] Connection error:', error);
                if (this.pendingReject) {
                    this.pendingReject(error);
                    this.pendingReject = null;
                    this.pendingResolve = null;
                }
                this.scheduleReconnect();
                reject(error);
            });

            this.socket.on('data', (data: string) => {
                this.handleData(data);
            });

            this.socket.on('close', () => {
                Logger.warn('[UDSTransport] Connection closed');
                this.scheduleReconnect();
            });
        });
    }

    async send(message: TransportMessage): Promise<TransportResponse> {
        if (!this.socket || this.socket.destroyed) {
            await this.connect();
        }

        return new Promise((resolve, reject) => {
            // Build request in server-expected format
            const request = this.buildRequest(message);
            const jsonLine = JSON.stringify(request) + '\n';
            
            Logger.info(`[UDSTransport] Sending request: ${jsonLine.substring(0, 200)}...`);
            
            this.pendingResolve = resolve;
            this.pendingReject = reject;
            this.streamCallback = null;

            this.socket!.write(jsonLine, (error) => {
                if (error) {
                    Logger.error('[UDSTransport] Write error:', error);
                    this.pendingReject = null;
                    this.pendingResolve = null;
                    reject(error);
                }
            });

            // Timeout after 60s (inference can be slow)
            setTimeout(() => {
                if (this.pendingResolve) {
                    const err = new Error('Request timeout after 60s');
                    Logger.error('[UDSTransport] Timeout');
                    this.pendingReject?.(err);
                    this.pendingResolve = null;
                    this.pendingReject = null;
                }
            }, 60000);
        });
    }

    async sendStream(message: TransportMessage, onChunk: (chunk: string) => void): Promise<void> {
        if (!this.socket || this.socket.destroyed) {
            await this.connect();
        }

        return new Promise((resolve, reject) => {
            // Build request with stream: true
            const request = this.buildRequest(message, true);
            const jsonLine = JSON.stringify(request) + '\n';
            
            Logger.info(`[UDSTransport] Sending stream request: ${jsonLine.substring(0, 200)}...`);
            
            this.streamCallback = onChunk;
            this.pendingResolve = () => {
                this.streamCallback = null;
                resolve();
            };
            this.pendingReject = (err) => {
                this.streamCallback = null;
                reject(err);
            };

            this.socket!.write(jsonLine, (error) => {
                if (error) {
                    Logger.error('[UDSTransport] Write error:', error);
                    this.streamCallback = null;
                    this.pendingResolve = null;
                    this.pendingReject = null;
                    reject(error);
                }
            });

            // Timeout after 120s for stream
            setTimeout(() => {
                if (this.pendingResolve) {
                    const err = new Error('Stream timeout after 120s');
                    Logger.error('[UDSTransport] Stream timeout');
                    this.pendingReject?.(err);
                    this.streamCallback = null;
                    this.pendingResolve = null;
                    this.pendingReject = null;
                }
            }, 120000);
        });
    }

    private buildRequest(message: TransportMessage, stream: boolean = false): object {
        // Convert extension format to server format
        const payload = message.payload as any;
        
        // Map ChatMessage format from extension to server
        const messages = payload.messages?.map((m: any) => ({
            role: m.role,
            content: m.content,
        })) || [];

        return {
            messages,
            max_tokens: 2048,
            stream,
        };
    }

    private handleData(data: string): void {
        this.buffer += data;
        
        // Process complete lines (NDJSON)
        const lines = this.buffer.split('\n');
        this.buffer = lines.pop() || ''; // Keep incomplete line in buffer

        for (const line of lines) {
            if (!line.trim()) continue;
            
            try {
                const response = JSON.parse(line);
                Logger.info(`[UDSTransport] Received: ${JSON.stringify(response).substring(0, 200)}`);
                
                // Check if it's a stream chunk or final response
                if (response.token !== undefined && this.streamCallback) {
                    // Stream chunk: { "token": "..." }
                    this.streamCallback(response.token);
                } else if (response.done !== undefined) {
                    // Final response: { "output": "...", "done": true, "metrics": {...} }
                    if (this.pendingResolve) {
                        this.pendingResolve({
                            type: 'response',
                            payload: {
                                content: response.output || '',
                                done: response.done,
                                metrics: response.metrics,
                            },
                        });
                        this.pendingResolve = null;
                        this.pendingReject = null;
                    }
                } else if (response.error) {
                    // Error response
                    Logger.error('[UDSTransport] Server error:', response.error);
                    if (this.pendingReject) {
                        this.pendingReject(new Error(response.error));
                        this.pendingResolve = null;
                        this.pendingReject = null;
                    }
                }
            } catch (error) {
                Logger.error('[UDSTransport] Failed to parse response:', error, line);
            }
        }
    }

    private scheduleReconnect(): void {
        if (this.reconnectTimer) {
            return;
        }

        this.reconnectTimer = setTimeout(async () => {
            this.reconnectTimer = null;
            try {
                await this.connect();
            } catch (error) {
                Logger.error('[UDSTransport] Reconnection failed:', error);
            }
        }, 5000);
    }

    dispose(): void {
        if (this.reconnectTimer) {
            clearTimeout(this.reconnectTimer);
        }
        if (this.socket) {
            this.socket.destroy();
        }
        this.buffer = '';
        this.pendingResolve = null;
        this.pendingReject = null;
        this.streamCallback = null;
    }
}
