import { useState, useEffect, useCallback } from 'react';
import { vscodeApi } from '../services/vscodeApi';
import { ChatMessage } from '../../shared/protocol';
import { ChatMode } from '../../shared/types';

/**
 * Chat hook for managing chat state
 */
export const useChat = (mode: ChatMode) => {
    const [messages, setMessages] = useState<ChatMessage[]>([]);
    const [isLoading, setIsLoading] = useState(false);
    const [currentResponse, setCurrentResponse] = useState('');

    useEffect(() => {
        const handleMessage = (event: MessageEvent) => {
            const message = event.data;

            console.log('[useChat] Received message:', message.type, message.payload);

            switch (message.type) {
                case 'chatStart':
                    console.log('[useChat] Chat started, setting loading state');
                    setIsLoading(true);
                    setCurrentResponse('');
                    break;

                case 'chatChunk':
                    // console.log('[useChat] Chunk received:', message.payload.content); // Too verbose
                    setCurrentResponse((prev) => prev + message.payload.content);
                    break;

                case 'chatEnd':
                    console.log('[useChat] Chat response ended');
                    setIsLoading(false);
                    setMessages((prev) => [
                        ...prev,
                        { role: 'assistant', content: currentResponse || message.payload?.content || '' },
                    ]);
                    setCurrentResponse('');
                    break;

                case 'chatError':
                    console.error('[useChat] Chat error:', message.payload.error);
                    setIsLoading(false);
                    setMessages((prev) => [
                        ...prev,
                        {
                            role: 'assistant',
                            content: `Error: ${message.payload.error}`,
                        },
                    ]);
                    break;
            }
        };

        window.addEventListener('message', handleMessage);
        return () => window.removeEventListener('message', handleMessage);
    }, [currentResponse]);

    const sendMessage = useCallback(
        (content: string) => {
            if (!content.trim()) return;

            const userMessage: ChatMessage = { role: 'user', content };
            console.log('[useChat] Sending message:', userMessage);
            
            setMessages((prev) => [...prev, userMessage]);

            vscodeApi.postMessage({
                type: 'chat',
                payload: {
                    messages: [...messages, userMessage],
                    mode,
                },
            });
        },
        [messages, mode]
    );

    const clearMessages = useCallback(() => {
        setMessages([]);
        setCurrentResponse('');
    }, []);

    return {
        messages,
        sendMessage,
        isLoading,
        currentResponse,
        clearMessages,
    };
};
