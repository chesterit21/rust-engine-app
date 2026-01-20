import React, { useState, useCallback } from 'react';
import { ChatPanel } from './components/ChatPanel';
import { FileContext } from './components/FileContext';
import { ModeSelector } from './components/ModeSelector';
import { useChat } from './hooks/useChat';
import { useFileContext } from './hooks/useFileContext';
import { ChatMode } from '../shared/types';

/**
 * Main App component
 */
export const App: React.FC = () => {
    const [mode, setMode] = useState<ChatMode>('normal');
    const { messages, sendMessage, isLoading, currentResponse } = useChat(mode);
    const { files, removeFile, clearFiles } = useFileContext();

    const handleSend = useCallback(
        (message: string) => {
            sendMessage(message);
        },
        [sendMessage]
    );

    return (
        <div className="app-container">
            <div className="header">
                <h2>SFCore-Agent</h2>
                <ModeSelector mode={mode} onModeChange={setMode} />
            </div>

            <FileContext files={files} onRemove={removeFile} onClear={clearFiles} />

            <ChatPanel
                messages={messages}
                onSend={handleSend}
                isLoading={isLoading}
                currentResponse={currentResponse}
            />
        </div>
    );
};
