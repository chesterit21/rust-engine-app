import React from 'react';
import { ChatMode } from '../../../shared/types';
import './styles.css';

interface ModeSelectorProps {
    mode: ChatMode;
    onModeChange: (mode: ChatMode) => void;
}

/**
 * Mode selector component for switching between normal and search modes
 */
export const ModeSelector: React.FC<ModeSelectorProps> = ({ mode, onModeChange }) => {
    return (
        <div className="mode-selector">
            <button
                className={`mode-button ${mode === 'normal' ? 'active' : ''}`}
                onClick={() => onModeChange('normal')}
                title="Normal chat mode"
            >
                💬 Chat
            </button>
            <button
                className={`mode-button ${mode === 'search' ? 'active' : ''}`}
                onClick={() => onModeChange('search')}
                title="Search mode"
            >
                🔍 Search
            </button>
        </div>
    );
};
