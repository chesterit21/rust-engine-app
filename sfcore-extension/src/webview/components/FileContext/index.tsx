import React from 'react';
import { FileContextItem } from '../../../shared/protocol';
import { FileChip } from './FileChip';
import './styles.css';

interface FileContextProps {
    files: FileContextItem[];
    onRemove: (uri: string) => void;
    onClear: () => void;
}

/**
 * File context container component
 */
export const FileContext: React.FC<FileContextProps> = ({ files, onRemove, onClear }) => {
    if (files.length === 0) {
        return null;
    }

    return (
        <div className="file-context">
            <div className="file-context-header">
                <span className="file-context-label">
                    📎 Context ({files.length} file{files.length > 1 ? 's' : ''})
                </span>
                <button className="clear-button" onClick={onClear} title="Clear all">
                    Clear
                </button>
            </div>
            <div className="file-chips">
                {files.map((file) => (
                    <FileChip key={file.uri} file={file} onRemove={() => onRemove(file.uri)} />
                ))}
            </div>
        </div>
    );
};
