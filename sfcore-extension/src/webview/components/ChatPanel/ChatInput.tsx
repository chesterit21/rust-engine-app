/**
 * Chat input component
 */
import React, { useState, useRef, KeyboardEvent, useEffect } from 'react';
import './styles.css';

interface ChatInputProps {
    onSend: (message: string) => void;
    disabled?: boolean;
    placeholder?: string;
}

export const ChatInput: React.FC<ChatInputProps> = ({ 
    onSend, 
    disabled = false,
    placeholder = 'Type a message...'
}) => {
    const [message, setMessage] = useState('');
    const textareaRef = useRef<HTMLTextAreaElement>(null);

    // Auto-resize textarea logic
    useEffect(() => {
        if (textareaRef.current) {
            textareaRef.current.style.height = 'auto';
            textareaRef.current.style.height = `${Math.min(textareaRef.current.scrollHeight, 200)}px`;
            
            // Show scrollbar only if content exceeds max-height
            if (textareaRef.current.scrollHeight > 200) {
                textareaRef.current.style.overflowY = 'auto';
            } else {
                textareaRef.current.style.overflowY = 'hidden';
            }
        }
    }, [message]);

    const handleSend = () => {
        const trimmed = message.trim();
        if (trimmed && !disabled) {
            onSend(trimmed);
            setMessage('');
        }
    };

    const handleKeyDown = (e: KeyboardEvent<HTMLTextAreaElement>) => {
        if (e.key === 'Enter' && !e.shiftKey) {
            e.preventDefault();
            handleSend();
        }
    };

    return (
        <div className="chat-input-container">
            <textarea
                ref={textareaRef}
                className="chat-input"
                value={message}
                onChange={(e) => setMessage(e.target.value)}
                onKeyDown={handleKeyDown}
                placeholder={placeholder}
                disabled={disabled}
                rows={1}
            />
            <button
                className={`send-button ${disabled ? 'loading' : ''}`}
                onClick={handleSend}
                disabled={!message.trim() || disabled}
                title="Send"
            >
                {disabled ? (
                    // Loading Spinner Icon
                    <svg viewBox="0 0 24 24" fill="none" xmlns="http://www.w3.org/2000/svg">
                        <path d="M12 22C17.5228 22 22 17.5228 22 12C22 6.47715 17.5228 2 12 2C6.47715 2 2 6.47715 2 12C2 17.5228 6.47715 22 12 22Z" strokeOpacity="0.3" strokeWidth="2" stroke="currentColor"/>
                        <path d="M12 2C6.47715 2 2 6.47715 2 12C2 14.7614 3.11929 17.2614 4.92893 19.0711" strokeWidth="2" stroke="currentColor"/>
                    </svg>
                ) : (
                    // Play/Send Icon (Triangle like Play button)
                    <svg viewBox="0 0 24 24" xmlns="http://www.w3.org/2000/svg">
                        <path d="M8 5V19L19 12L8 5Z" />
                    </svg>
                )}
            </button>
        </div>
    );
};
