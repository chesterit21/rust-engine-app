/**
 * Common shared types between extension and webview
 */

/**
 * Chat modes
 */
export type ChatMode = 'normal' | 'search';

/**
 * Transport type configuration
 */
export type TransportType = 'auto' | 'uds' | 'http';

/**
 * Extension configuration
 */
export interface ExtensionConfig {
    transport: {
        type: TransportType;
        uds: {
            socketPath: string;
        };
        http: {
            baseUrl: string;
        };
    };
}

/**
 * Log level
 */
export enum LogLevel {
    DEBUG = 'DEBUG',
    INFO = 'INFO',
    WARN = 'WARN',
    ERROR = 'ERROR',
}

/**
 * Connection status
 */
export enum ConnectionStatus {
    CONNECTED = 'connected',
    DISCONNECTED = 'disconnected',
    CONNECTING = 'connecting',
    ERROR = 'error',
}
