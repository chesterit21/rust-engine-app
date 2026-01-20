import React from 'react';

interface FilePickerProps {
    onSelect: (uri: string) => void;
}

/**
 * File picker component (placeholder - actual file picking done via VS Code API)
 */
export const FilePicker: React.FC<FilePickerProps> = ({ onSelect }) => {
    // Note: File picking is handled by VS Code commands, not by webview
    // This component can be used for drag-and-drop functionality in the future

    return (
        <div className="file-picker">
            <p className="picker-hint">
                Use the context menu in Explorer to add files, or press <kbd>Ctrl+Shift+A</kbd>
            </p>
        </div>
    );
};
