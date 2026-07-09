/**
 * Type declarations for ghostty-web WASM module.
 * This provides TypeScript types for the Ghostty Web API.
 */

export interface Terminal {
	open(container: HTMLElement): void;
	write(data: string): void;
	resize(cols: number, rows: number): void;
	onData(callback: (data: string) => void): void;
	onScroll(callback: (position: number) => void): void;
	onResize(callback: (resize: { cols: number; rows: number }) => void): void;
	blur(): void;
	focus(): void;
	clear(): void;
	reset(): void;
	getCols(): number;
	getRows(): number;
	destroy(): void;
}

export interface GhosttyOptions {
	cols?: number;
	rows?: number;
	scrollback?: number;
	fontSize?: number;
	fontFamily?: string;
	theme?: Record<string, string>;
	[key: string]: unknown;
}

export function init(options?: Record<string, unknown>): Promise<void>;
export function Terminal(options: GhosttyOptions): Terminal;
