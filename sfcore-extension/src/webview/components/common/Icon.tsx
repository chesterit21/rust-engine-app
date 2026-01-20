import React from 'react';

interface IconProps {
    name: string;
    size?: number;
    className?: string;
}

/**
 * Icon component using emoji (can be replaced with actual icon library)
 */
export const Icon: React.FC<IconProps> = ({ name, size = 16, className = '' }) => {
    const icons: Record<string, string> = {
        send: '📤',
        file: '📄',
        folder: '📁',
        close: '✕',
        clear: '🗑️',
        settings: '⚙️',
        user: '👤',
        bot: '🤖',
        search: '🔍',
        chat: '💬',
        loading: '⏳',
        error: '❌',
        success: '✅',
        warning: '⚠️',
        info: 'ℹ️',
    };

    return (
        <span
            className={`icon ${className}`}
            style={{ fontSize: size }}
            role="img"
            aria-label={name}
        >
            {icons[name] || '❓'}
        </span>
    );
};
