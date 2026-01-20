import { useState, useEffect, useCallback } from 'react';
import { vscodeApi } from '../services/vscodeApi';
import { FileContextItem } from '../../shared/protocol';

/**
 * File context hook
 */
export const useFileContext = () => {
    const [files, setFiles] = useState<FileContextItem[]>([]);

    useEffect(() => {
        const handleMessage = (event: MessageEvent) => {
            const message = event.data;

            if (message.type === 'contextUpdate') {
                setFiles(message.payload);
            } else if (message.type === 'init') {
                setFiles(message.payload.files || []);
            }
        };

        window.addEventListener('message', handleMessage);
        return () => window.removeEventListener('message', handleMessage);
    }, []);

    const addFile = useCallback((uri: string) => {
        vscodeApi.postMessage({
            type: 'addFile',
            payload: { uri },
        });
    }, []);

    const removeFile = useCallback((uri: string) => {
        vscodeApi.postMessage({
            type: 'removeFile',
            payload: { uri },
        });
    }, []);

    const clearFiles = useCallback(() => {
        vscodeApi.postMessage({
            type: 'clearContext',
            payload: {},
        });
    }, []);

    return {
        files,
        addFile,
        removeFile,
        clearFiles,
    };
};
