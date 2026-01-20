/**
 * Transport layer types
 */

/**
 * Transport message structure
 */
export interface TransportMessage {
    type: string;
    payload: unknown;
    stream?: boolean;
}

/**
 * Transport response structure
 */
export interface TransportResponse {
    id?: string;
    type: string;
    payload: unknown;
    stream?: AsyncIterable<string>;
    error?: string;
}

/**
 * Transport interface that must be implemented by all transport providers
 */
export interface ITransport {
    /**
     * Connect to the transport
     */
    connect(): Promise<void>;

    /**
     * Send a message and wait for response
     */
    send(message: TransportMessage): Promise<TransportResponse>;

    /**
     * Send a message with streaming response
     */
    sendStream(message: TransportMessage, onChunk: (chunk: string) => void): Promise<void>;

    /**
     * Dispose the transport
     */
    dispose(): void;
}
