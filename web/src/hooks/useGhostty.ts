/**
 * Hook for managing a Ghostty terminal instance.
 * Uses real ghostty-web WASM with xterm-compatible API.
 */

import { useEffect, useRef, useCallback } from "react";
import { init, Terminal, type ITerminalOptions } from "ghostty-web";

export interface UseGhosttyOptions {
	cols: number;
	rows: number;
	agentId?: string;
	onInput?: (data: Uint8Array) => void;
	onOutput?: (data: Uint8Array) => void;
	onError?: (error: string) => void;
	onResize?: (cols: number, rows: number) => void;
}

export function useGhostty({
	cols,
	rows,
	onInput,
	onError,
	onResize,
}: UseGhosttyOptions) {
	const containerRef = useRef<HTMLDivElement>(null);
	const termRef = useRef<Terminal | null>(null);
	const initializedRef = useRef(false);
	const resizeSentRef = useRef(false);

	// Initialize Ghostty terminal
	useEffect(() => {
		if (initializedRef.current || !containerRef.current) return;

		let term: Terminal;

		(async () => {
			try {
				// Initialize WASM (idempotent — safe to call multiple times)
				await init();

				// Terminal options matching our Dark theme
				const options: ITerminalOptions = {
					cols,
					rows,
					scrollback: 10000,
					cursorBlink: true,
					cursorStyle: "block",
					fontFamily:
						'"JetBrains Mono", "Fira Code", "Cascadia Code", monospace',
					fontSize: 15,
					theme: {
						background: "#1a1b26",
						foreground: "#a9b1d6",
						cursor: "#c0caf5",
						selectionBackground: "#3b4261",
						black: "#32344a",
						red: "#f7768e",
						green: "#9ece6a",
						yellow: "#e0af68",
						blue: "#7aa2f7",
						magenta: "#bb9af7",
						cyan: "#7dcfff",
						white: "#a9b1d6",
						brightBlack: "#44466a",
						brightRed: "#ff7a93",
						brightGreen: "#b9f27c",
						brightYellow: "#ff9e64",
						brightBlue: "#7da6ff",
						brightMagenta: "#c0a3e3",
						brightCyan: "#0dbbd1",
						brightWhite: "#acb0d0",
					},
				};

				term = new Terminal(options);
				if (containerRef.current) {
					term.open(containerRef.current);
				}
				termRef.current = term;
				initializedRef.current = true;

				// Listen for terminal input → forward to PTY
				term.onData((data: string) => {
					if (onInput) {
						onInput(new TextEncoder().encode(data));
					}
				});

				// Report initial size
				if (onResize) {
					onResize(term.cols, term.rows);
				}

				term.onResize((resize) => {
					// Only fire onResize for actual resize events (not initial)
					if (resizeSentRef.current && onResize) {
						onResize(resize.cols, resize.rows);
					}
				});

				// Report initial size (single call)
				if (onResize) {
					resizeSentRef.current = true;
					onResize(term.cols, term.rows);
				}
			} catch (err) {
				const msg = err instanceof Error ? err.message : String(err);
				console.error("[Ghostty] Init failed:", msg);
				if (onError) onError(msg);
			}
		})();

		return () => {
			// Dispose on unmount
			if (termRef.current) {
				termRef.current.dispose();
				termRef.current = null;
			}
			initializedRef.current = false;
		};
	}, []);

	// Write output to terminal (called by PTY read loop)
	const writeOutput = useCallback((data: Uint8Array) => {
		if (termRef.current) {
			termRef.current.write(data);
		}
	}, []);

	// Resize terminal
	const resize = useCallback((newCols: number, newRows: number) => {
		if (termRef.current) {
			termRef.current.resize(newCols, newRows);
		}
	}, []);

	// Write input to terminal (user typing — forwarded to PTY)
	const writeInput = useCallback((data: Uint8Array) => {
		if (termRef.current) {
			// Convert bytes to string and use Ghostty's input() which triggers onData
			const text = new TextDecoder().decode(data);
			termRef.current.input(text, true);
		}
	}, []);

	return { writeOutput, writeInput, resize, containerRef };
}
