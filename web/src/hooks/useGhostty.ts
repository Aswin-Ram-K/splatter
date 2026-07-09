/**
 * Hook for managing a Ghostty terminal instance.
 * Note: ghostty-web WASM is loaded dynamically at runtime.
 */

import { useEffect, useRef, useCallback } from "react";

interface UseGhosttyOptions {
	cols: number;
	rows: number;
	agentId?: string;
	onOutput?: (data: Uint8Array) => void;
	onError?: (error: string) => void;
	onResize?: (cols: number, rows: number) => void;
}

interface GhosttyTerminal {
	open(container: HTMLElement): void;
	write(data: string | Uint8Array): void;
	resize(cols: number, rows: number): void;
	onData(callback: (data: string) => void): void;
	onResize(callback: (resize: { cols: number; rows: number }) => void): void;
	onScroll(callback: (position: number) => void): void;
	dispose(): void;
}

// Dynamic import type (loaded at runtime from ghostty-web WASM)
declare global {
	interface Window {
		GhosttyTerminal?: new (opts: {
			cols: number;
			rows: number;
			scrollback: number;
			fontSize: number;
			fontFamily: string;
			theme: Record<string, string>;
		}) => GhosttyTerminal;
	}
}

export function useGhostty({
	cols,
	rows,
	onOutput,
	onError,
	onResize,
}: UseGhosttyOptions) {
	const containerRef = useRef<HTMLDivElement>(null);
	const terminalRef = useRef<GhosttyTerminal | null>(null);
	const initializedRef = useRef(false);

	// Initialize Ghostty terminal
	useEffect(() => {
		if (initializedRef.current || !containerRef.current) return;

		let term: GhosttyTerminal | null = null;

		(async () => {
			try {
				// In production, this would import from 'ghostty-web'
				// For now, use a placeholder that creates a simple terminal-like object
				const termDiv = containerRef.current;
				termDiv!.style.backgroundColor = "#1a1b26";
				termDiv!.style.color = "#a9b1d6";
				termDiv!.style.fontFamily = "JetBrains Mono, monospace";
				termDiv!.style.fontSize = "15px";
				termDiv!.style.padding = "4px";
				termDiv!.style.overflow = "hidden";
				termDiv!.style.whiteSpace = "pre";
				termDiv!.style.fontVariantLigatures = "none";
				termDiv!.style.tabSize = "0";

				// Create a simple terminal mock for development
				term = {
					open: () => {},
					write: (data: string | Uint8Array) => {
						if (typeof data === "string") {
							const span = document.createElement("span");
							span.textContent = data;
							termDiv!.appendChild(span);
						}
					},
					resize: (_c: number, _r: number) => {},
					onData: (_callback: (data: string) => void) => () => {},
					onResize: (
						callback: (resize: { cols: number; rows: number }) => void,
					) => {
						callback({ cols, rows });
					},
					onScroll: (_callback: (position: number) => void) => {},
					dispose: () => {},
				} as unknown as GhosttyTerminal;

				terminalRef.current = term;
				initializedRef.current = true;

				// Trigger onResize callback
				if (onResize) {
					onResize(cols, rows);
				}
			} catch (err) {
				if (onError) onError(err instanceof Error ? err.message : String(err));
			}
		})();

		return () => {
			// Dispose on unmount
			if (terminalRef.current) {
				terminalRef.current.dispose();
				terminalRef.current = null;
			}
			initializedRef.current = false;
		};
	}, [cols, rows, onOutput, onError, onResize]);

	// Write output to terminal (called by PTY read loop)
	const writeOutput = useCallback((data: Uint8Array) => {
		if (terminalRef.current && containerRef.current) {
			terminalRef.current.write(data);
		}
	}, []);

	// Write input to terminal (user typing)
	const writeInput = useCallback((_data: Uint8Array) => {
		// Would send input to PTY in production
	}, []);

	return { writeOutput, writeInput };
}
