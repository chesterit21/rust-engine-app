import { ITransport, TransportMessage, TransportResponse } from './types';
import { Logger } from '../utils/logger';

/**
 * HTTP/SSE Transport implementation
 * Fallback transport with cross-platform support
 */
export class HTTPTransport implements ITransport {
    private baseUrl: string;
    private abortControllers: Map<string, AbortController>;

    constructor(baseUrl: string = 'http://localhost:8080') {
        this.baseUrl = baseUrl;
        this.abortControllers = new Map();
    }

    async connect(): Promise<void> {
        // Test connection with health check
        try {
            const response = await fetch(`${this.baseUrl}/health`);
            if (!response.ok) {
                throw new Error('Health check failed');
            }
            Logger.info(`Connected to HTTP: ${this.baseUrl}`);
        } catch (error) {
            Logger.error('HTTP connection error:', error);
            throw error;
        }
    }

    async send(message: TransportMessage): Promise<TransportResponse> {
        const controller = new AbortController();
        const messageId = this.generateMessageId();
        this.abortControllers.set(messageId, controller);

        try {
            const response = await fetch(`${this.baseUrl}/api/chat`, {
                method: 'POST',
                headers: {
                    'Content-Type': 'application/json',
                },
                body: JSON.stringify(message),
                signal: controller.signal,
            });

            if (!response.ok) {
                throw new Error(`HTTP ${response.status}: ${response.statusText}`);
            }

            const payload = await response.json();
            return {
                id: messageId,
                type: 'response',
                payload,
            };
        } finally {
            this.abortControllers.delete(messageId);
        }
    }

    async sendStream(message: TransportMessage, onChunk: (chunk: string) => void): Promise<void> {
        const controller = new AbortController();
        const messageId = this.generateMessageId();
        this.abortControllers.set(messageId, controller);

        try {
            const response = await fetch(`${this.baseUrl}/api/chat/stream`, {
                method: 'POST',
                headers: {
                    'Content-Type': 'application/json',
                },
                body: JSON.stringify(message),
                signal: controller.signal,
            });

            if (!response.ok) {
                throw new Error(`HTTP ${response.status}`);
            }

            const reader = response.body?.getReader();
            const decoder = new TextDecoder();

            if (!reader) {
                throw new Error('No response body');
            }

            while (true) {
                const { done, value } = await reader.read();
                if (done) {
                    break;
                }

                const chunk = decoder.decode(value, { stream: true });
                const lines = chunk.split('\n');

                for (const line of lines) {
                    if (line.startsWith('data: ')) {
                        const data = line.slice(6);
                        if (data === '[DONE]') {
                            continue;
                        }

                        try {
                            const parsed = JSON.parse(data);
                            onChunk(parsed.content || '');
                        } catch (e) {
                            Logger.error('Failed to parse SSE data:', e);
                        }
                    }
                }
            }
        } finally {
            this.abortControllers.delete(messageId);
        }
    }

    private generateMessageId(): string {
        return `${Date.now()}-${Math.random().toString(36).substr(2, 9)}`;
    }

    dispose(): void {
        this.abortControllers.forEach((controller) => controller.abort());
        this.abortControllers.clear();
    }
}
