import * as vscode from 'vscode';
import { LogLevel } from '../../shared/types';

/**
 * Logger utility for extension
 */
export class Logger {
    private static outputChannel: vscode.OutputChannel | null = null;
    private static logLevel: LogLevel = LogLevel.INFO;

    static initialize(context: vscode.ExtensionContext): void {
        this.outputChannel = vscode.window.createOutputChannel('AI Dev Agent');
        context.subscriptions.push(this.outputChannel);
    }

    static setLogLevel(level: LogLevel): void {
        this.logLevel = level;
    }

    static debug(message: string, ...args: unknown[]): void {
        if (this.shouldLog(LogLevel.DEBUG)) {
            this.log(LogLevel.DEBUG, message, ...args);
        }
    }

    static info(message: string, ...args: unknown[]): void {
        if (this.shouldLog(LogLevel.INFO)) {
            this.log(LogLevel.INFO, message, ...args);
        }
    }

    static warn(message: string, ...args: unknown[]): void {
        if (this.shouldLog(LogLevel.WARN)) {
            this.log(LogLevel.WARN, message, ...args);
        }
    }

    static error(message: string, ...args: unknown[]): void {
        if (this.shouldLog(LogLevel.ERROR)) {
            this.log(LogLevel.ERROR, message, ...args);
        }
    }

    private static shouldLog(level: LogLevel): boolean {
        const levels = [LogLevel.DEBUG, LogLevel.INFO, LogLevel.WARN, LogLevel.ERROR];
        return levels.indexOf(level) >= levels.indexOf(this.logLevel);
    }

    private static log(level: LogLevel, message: string, ...args: unknown[]): void {
        const timestamp = new Date().toISOString();
        const formattedArgs = args
            .map((arg) => (typeof arg === 'object' ? JSON.stringify(arg) : String(arg)))
            .join(' ');
        const logMessage = `[${timestamp}] [${level}] ${message} ${formattedArgs}`.trim();

        if (this.outputChannel) {
            this.outputChannel.appendLine(logMessage);
        }

        // Also log to console for debugging
        console.log(logMessage);
    }

    static show(): void {
        this.outputChannel?.show();
    }
}
