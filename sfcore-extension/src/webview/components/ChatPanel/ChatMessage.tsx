/**
 * Individual chat message component
 */
import React, { useMemo } from 'react';
import { ChatMessage as MessageType } from '../../../shared/protocol';
import './styles.css';

interface ChatMessageProps {
    message: MessageType;
    isStreaming?: boolean;
}

/**
 * Parse message content and convert <think> blocks into collapsible elements
 */
const parseThinkingBlocks = (content: string): React.ReactNode[] => {
    const parts: React.ReactNode[] = [];
    // Regex to match <think>...</think> blocks, including multiline
    const thinkRegex = /<think>([\s\S]*?)<\/think>/gi;
    let lastIndex = 0;
    let match;
    let keyIndex = 0;

    while ((match = thinkRegex.exec(content)) !== null) {
        // Add text before the think block
        if (match.index > lastIndex) {
            const textBefore = content.slice(lastIndex, match.index);
            if (textBefore.trim()) {
                parts.push(
                    <span key={`text-${keyIndex++}`} className="message-text-segment">
                        {textBefore}
                    </span>
                );
            }
        }

        // Add collapsible think block
        const thinkContent = match[1].trim();
        parts.push(
            <details key={`think-${keyIndex++}`} className="thinking-block">
                <summary className="thinking-summary">💭 Thinking...</summary>
                <div className="thinking-content">{thinkContent}</div>
            </details>
        );

        lastIndex = match.index + match[0].length;
    }

    // Add remaining text after last think block
    if (lastIndex < content.length) {
        const remaining = content.slice(lastIndex);
        if (remaining.trim()) {
            parts.push(
                <span key={`text-${keyIndex++}`} className="message-text-segment">
                    {remaining}
                </span>
            );
        }
    }

    // If no think blocks found, return the original content
    if (parts.length === 0) {
        return [<span key="text-0">{content}</span>];
    }

    return parts;
};

export const ChatMessage: React.FC<ChatMessageProps> = ({ 
    message,
    isStreaming = false
}) => {
    // Parse thinking blocks for assistant messages
    const parsedContent = useMemo(() => {
        if (message.role === 'assistant') {
            return parseThinkingBlocks(message.content);
        }
        return message.content;
    }, [message.content, message.role]);

    return (
        <div className={`chat-message chat-message-${message.role} ${isStreaming ? 'streaming' : ''}`}>
            <div className="message-content">
                {message.role === 'assistant' ? (
                    <div className="message-text">{parsedContent}</div>
                ) : (
                    <pre className="message-text">{message.content}</pre>
                )}
            </div>
        </div>
    );
};
