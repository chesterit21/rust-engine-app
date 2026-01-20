import { ITransport, TransportMessage } from '../transport/types';
import { ChatMessage, ChatRequest, ChatResponse } from '../../shared/protocol';
import { Logger } from '../utils/logger';

/**
 * LLM Service for communication with Rust LLM Server
 */
export class LLMService {
    private transport: ITransport;

    constructor(transport: ITransport) {
        this.transport = transport;
    }

    /**
     * Send chat message to LLM
     */
    async chat(
        messages: ChatMessage[],
        options: {
            isSearch?: boolean;
            context?: string[];
            onStream?: (chunk: string) => void;
        } = {}
    ): Promise<ChatResponse> {
        const request: ChatRequest = {
            messages,
            is_search: options.isSearch || false,
            context: options.context || [],
            stream: !!options.onStream,
        };

        if (options.onStream) {
            let fullResponse = '';
            Logger.info(`[LLMService] Sending stream request: ${JSON.stringify(request.messages[request.messages.length - 1])}`);

            try {
                await this.transport.sendStream({ type: 'chat', payload: request }, (chunk) => {
                    // Logger.debug(`[LLMService] Received chunk: ${chunk.length} chars`);
                    fullResponse += chunk;
                    options.onStream!(chunk);
                });
                
                Logger.info('[LLMService] Stream completed successfully');
            } catch (error) {
                Logger.error('[LLMService] Stream error:', error);
                throw error;
            }

            return {
                content: fullResponse,
                model: 'unknown',
                usage: { prompt_tokens: 0, completion_tokens: 0 },
            };
        } else {
            Logger.info(`[LLMService] Sending request: ${JSON.stringify(request.messages[request.messages.length - 1])}`);
            
            try {
                const response = await this.transport.send({
                    type: 'chat',
                    payload: request,
                });
                Logger.info('[LLMService] Received response successfully');
                return response.payload as ChatResponse;
            } catch (error) {
                 Logger.error('[LLMService] Request error:', error);
                 throw error;
            }
        }
    }

    /**
     * Cancel current request
     */
    async cancelRequest(): Promise<void> {
        await this.transport.send({
            type: 'cancel',
            payload: {},
        });
    }

    /**
     * Dispose service
     */
    dispose(): void {
        this.transport.dispose();
    }
}
