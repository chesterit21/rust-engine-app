import { useEffect, useState } from 'react';
import { vscodeApi } from '../services/vscodeApi';

/**
 * VS Code hook for webview-extension communication
 * Tracks connection state and provides API methods
 */
export const useVSCode = () => {
    const [isConnected, setIsConnected] = useState(false);

    useEffect(() => {
        const handleMessage = (event: MessageEvent) => {
            const message = event.data;

            if (message.type === 'stateUpdate') {
                setIsConnected(message.payload.isConnected);
            }
        };

        window.addEventListener('message', handleMessage);

        // Notify extension that webview is ready
        vscodeApi.postMessage({
            type: 'ready',
            payload: {},
        });

        return () => window.removeEventListener('message', handleMessage);
    }, []);

    const postMessage = (type: string, payload: unknown = {}) => {
        vscodeApi.postMessage({ type, payload });
    };

    const getState = <T>(): T | undefined => {
        return vscodeApi.getState<T>();
    };

    const setState = <T>(state: T): void => {
        vscodeApi.setState(state);
    };

    return {
        isConnected,
        postMessage,
        getState,
        setState,
        vscode: vscodeApi,
    };
};
