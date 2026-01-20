import React from 'react';
import './common.css';

interface LoadingProps {
    text?: string;
    size?: 'small' | 'medium' | 'large';
}

/**
 * Loading spinner component
 */
export const Loading: React.FC<LoadingProps> = ({ text = 'Loading...', size = 'medium' }) => {
    const sizeStyles = {
        small: { dotSize: 6, gap: 3 },
        medium: { dotSize: 8, gap: 4 },
        large: { dotSize: 12, gap: 6 },
    }[size];

    return (
        <div className="loading-container">
            <div className="loading-dots" style={{ gap: sizeStyles.gap }}>
                <span
                    className="dot"
                    style={{ width: sizeStyles.dotSize, height: sizeStyles.dotSize }}
                />
                <span
                    className="dot"
                    style={{ width: sizeStyles.dotSize, height: sizeStyles.dotSize }}
                />
                <span
                    className="dot"
                    style={{ width: sizeStyles.dotSize, height: sizeStyles.dotSize }}
                />
            </div>
            {text && <span className="loading-text">{text}</span>}
        </div>
    );
};
