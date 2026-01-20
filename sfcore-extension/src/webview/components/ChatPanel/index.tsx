import React, { useRef, useEffect } from 'react';
import { ChatMessage as Message } from '../../../shared/protocol';
import { ChatMessage } from './ChatMessage';
import { ChatInput } from './ChatInput';
import './styles.css';

interface ChatPanelProps {
    messages: Message[];
    onSend: (message: string) => void;
    isLoading: boolean;
    currentResponse?: string;
}

/**
 * Chat Panel container component
 */
export const ChatPanel: React.FC<ChatPanelProps> = ({
    messages,
    onSend,
    isLoading,
    currentResponse,
}) => {
    const messagesEndRef = useRef<HTMLDivElement>(null);

    // Auto-scroll to bottom when new messages arrive
    useEffect(() => {
        messagesEndRef.current?.scrollIntoView({ behavior: 'smooth' });
    }, [messages, currentResponse]);

    return (
        <div className="chat-panel">
            <div className="messages-container">
                {messages.length === 0 && !currentResponse && (
                    <div className="empty-state">
                        <p>SFCore-Agent</p>
                        <p className="hint">AI Agent Planner Spesialisasi untuk pengembangan perangkat lunak</p>
                    </div>
                )}

                {messages.map((message, index) => (
                    <ChatMessage key={index} message={message} />
                ))}

                {currentResponse && (
                    <ChatMessage
                        message={{ role: 'assistant', content: currentResponse }}
                        isStreaming={true}
                    />
                )}

                {isLoading && !currentResponse && (
                    <div className="loading-indicator">
                        <span className="dot"></span>
                        <span className="dot"></span>
                        <span className="dot"></span>
                    </div>
                )}

                <div ref={messagesEndRef} />
            </div>

            <ChatInput onSend={onSend} disabled={isLoading} placeholder="Rencanan apapun..." />
        </div>
    );
};
