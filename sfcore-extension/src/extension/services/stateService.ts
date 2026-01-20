import * as vscode from 'vscode';
import { Logger } from '../utils/logger';

/**
 * State Service for extension state management
 */
export class StateService {
    private context: vscode.ExtensionContext;

    constructor(context: vscode.ExtensionContext) {
        this.context = context;
    }

    /**
     * Get value from global state
     */
    get<T>(key: string, defaultValue: T): T {
        return this.context.globalState.get<T>(key, defaultValue);
    }

    /**
     * Set value in global state
     */
    async set<T>(key: string, value: T): Promise<void> {
        await this.context.globalState.update(key, value);
        Logger.debug(`State updated: ${key}`);
    }

    /**
     * Get value from workspace state
     */
    getWorkspace<T>(key: string, defaultValue: T): T {
        return this.context.workspaceState.get<T>(key, defaultValue);
    }

    /**
     * Set value in workspace state
     */
    async setWorkspace<T>(key: string, value: T): Promise<void> {
        await this.context.workspaceState.update(key, value);
        Logger.debug(`Workspace state updated: ${key}`);
    }

    /**
     * Clear all state
     */
    async clear(): Promise<void> {
        const keys = this.context.globalState.keys();
        for (const key of keys) {
            await this.context.globalState.update(key, undefined);
        }
        Logger.info('State cleared');
    }
}
