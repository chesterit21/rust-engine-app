/**
 * Chat message structure
 */
export interface ChatMessage {
    role: 'user' | 'assistant' | 'system';
    content: string;
}

/**
 * Request to LLM server
 */
export interface ChatRequest {
    messages: ChatMessage[];
    is_search: boolean;
    context: string[];
    stream: boolean;
}

/**
 * Response from LLM server
 */
export interface ChatResponse {
    content: string;
    model: string;
    usage: {
        prompt_tokens: number;
        completion_tokens: number;
    };
}

/**
 * File context item for UI display
 */
export interface FileContextItem {
    name: string;
    uri: string;
}

/**
 * Messages from Webview to Extension Host
 */
export type MessageFromWebview =
    | { type: 'chat'; payload: { messages: ChatMessage[]; mode: string } }
    | { type: 'addFile'; payload: { uri: string } }
    | { type: 'removeFile'; payload: { uri: string } }
    | { type: 'clearContext'; payload: Record<string, never> }
    | { type: 'ready'; payload: Record<string, never> };

/**
 * Messages from Extension Host to Webview
 */
export type MessageToWebview =
    | { type: 'chatStart'; payload: Record<string, never> }
    | { type: 'chatChunk'; payload: { content: string } }
    | { type: 'chatEnd'; payload: Record<string, never> }
    | { type: 'chatError'; payload: { error: string } }
    | { type: 'contextUpdate'; payload: FileContextItem[] };
