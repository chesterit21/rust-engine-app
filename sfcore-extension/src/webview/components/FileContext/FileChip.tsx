import React from 'react';
import { FileContextItem } from '../../../shared/protocol';

interface FileChipProps {
    file: FileContextItem;
    onRemove: () => void;
}

/**
 * File chip component for displaying file in context
 */
export const FileChip: React.FC<FileChipProps> = ({ file, onRemove }) => {
    return (
        <div className="file-chip" title={file.uri}>
            <span className="file-icon">📄</span>
            <span className="file-name">{file.name}</span>
            <button className="remove-button" onClick={onRemove} title="Remove from context">
                ✕
            </button>
        </div>
    );
};
