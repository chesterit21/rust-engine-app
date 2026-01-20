/**
 * Webview types
 */

import { ChatMessage, FileContextItem } from '../../shared/protocol';

/**
 * Chat panel props
 */
export interface ChatPanelProps {
    messages: ChatMessage[];
    onSend: (message: string) => void;
    isLoading: boolean;
    currentResponse?: string;
}

/**
 * File context props
 */
export interface FileContextProps {
    files: FileContextItem[];
    onRemove: (uri: string) => void;
    onClear: () => void;
}

/**
 * Mode selector props
 */
export interface ModeSelectorProps {
    mode: 'normal' | 'search';
    onModeChange: (mode: 'normal' | 'search') => void;
}

/**
 * Chat input props
 */
export interface ChatInputProps {
    onSend: (message: string) => void;
    disabled?: boolean;
    placeholder?: string;
}

/**
 * Chat message props
 */
export interface ChatMessageProps {
    message: ChatMessage;
    isStreaming?: boolean;
}

/**
 * File chip props
 */
export interface FileChipProps {
    file: FileContextItem;
    onRemove: () => void;
}
