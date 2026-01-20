import * as vscode from 'vscode';
import { ContextService } from '../services/contextService';
import { Logger } from '../utils/logger';

/**
 * Context Provider for tree view (optional feature)
 */
export class ContextProvider implements vscode.TreeDataProvider<ContextItem> {
    private _onDidChangeTreeData: vscode.EventEmitter<ContextItem | undefined | null | void> =
        new vscode.EventEmitter<ContextItem | undefined | null | void>();
    readonly onDidChangeTreeData: vscode.Event<ContextItem | undefined | null | void> =
        this._onDidChangeTreeData.event;

    constructor(private contextService: ContextService) {
        this.contextService.onContextChange(() => {
            this.refresh();
        });
    }

    refresh(): void {
        this._onDidChangeTreeData.fire();
    }

    getTreeItem(element: ContextItem): vscode.TreeItem {
        return element;
    }

    getChildren(element?: ContextItem): Thenable<ContextItem[]> {
        if (!element) {
            const files = this.contextService.getFiles();
            return Promise.resolve(
                files.map(
                    (file) =>
                        new ContextItem(
                            file.name,
                            file.uri,
                            file.language,
                            vscode.TreeItemCollapsibleState.None
                        )
                )
            );
        }
        return Promise.resolve([]);
    }
}

/**
 * Context tree item
 */
export class ContextItem extends vscode.TreeItem {
    constructor(
        public readonly label: string,
        public readonly fileUri: vscode.Uri,
        public readonly language: string,
        public readonly collapsibleState: vscode.TreeItemCollapsibleState
    ) {
        super(label, collapsibleState);

        this.tooltip = this.fileUri.fsPath;
        this.description = this.language;
        this.resourceUri = this.fileUri;

        this.contextValue = 'contextFile';

        this.command = {
            command: 'vscode.open',
            title: 'Open File',
            arguments: [this.fileUri],
        };
    }
}
