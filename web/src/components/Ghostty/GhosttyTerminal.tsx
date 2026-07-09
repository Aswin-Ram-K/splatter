/**
 * Ghostty terminal wrapper component.
 * Renders a real Ghostty WASM terminal with PTY input/output bridge.
 */

import { useEffect, useCallback, useRef } from "react";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { useGhostty } from "@/hooks/useGhostty";

export interface GhosttyTerminalProps {
	paneId: number;
	agentId?: string;
	rect: { x: number; y: number; width: number; height: number };
	isFocused: boolean;
	onAgentSelect: (agentId: string | null) => void;
}

export function GhosttyTerminal({
	agentId,
	rect,
	isFocused,
}: GhosttyTerminalProps) {
	const containerRef = useRef<HTMLDivElement>(null);

	const {
		writeOutput,
		resize,
		containerRef: termRef,
	} = useGhostty({
		cols: Math.max(10, Math.floor(rect.width / 8)),
		rows: Math.max(3, Math.floor(rect.height / 16)),
		agentId,
		// Keyboard input FROM terminal (user typing) → forward to PTY
		onInput: useCallback(
			(data: Uint8Array) => {
				if (agentId) {
					invoke("write_to_agent", {
						agent_id: agentId,
						data: Array.from(data),
					}).catch(console.error);
				}
			},
			[agentId],
		),
	});

	// Expose container ref for Ghostty
	useEffect(() => {
		if (containerRef.current) {
			(termRef as any).current = containerRef.current;
		}
	}, [containerRef, termRef]);

	// Listen for PTY output from Rust and forward to terminal
	useEffect(() => {
		if (!agentId) return;

		const unsubPromise = listen(
			"agent-output",
			(event: { payload: { agent_id: string; data: number[] } }) => {
				if (event.payload.agent_id === agentId) {
					writeOutput(new Uint8Array(event.payload.data));
				}
			},
		);

		// Tauri listen() returns a promise — unwrap it
		unsubPromise.then((unsub: () => void) => {
			return () => unsub();
		});
	}, [agentId, writeOutput]);

	// Handle window resize → terminal resize
	useEffect(() => {
		if (!resize) return;

		const handleResize = () => {
			const newCols = Math.max(10, Math.floor(rect.width / 8));
			const newRows = Math.max(3, Math.floor(rect.height / 16));
			resize(newCols, newRows);
		};

		window.addEventListener("resize", handleResize);
		return () => window.removeEventListener("resize", handleResize);
	}, [resize, rect.width, rect.height]);

	// Focus the terminal when clicked
	const handleFocus = useCallback(() => {
		if (containerRef.current) {
			// Ghostty creates a hidden textarea for input — focus it
			const textarea = containerRef.current.querySelector("textarea");
			if (textarea) {
				(textarea as HTMLTextAreaElement).focus();
			}
		}
	}, []);

	return (
		<div
			ref={containerRef}
			className="h-full w-full outline-none"
			style={{
				border: isFocused ? "1px solid #7aa2f7" : "1px solid transparent",
			}}
			tabIndex={0}
			onFocus={handleFocus}
			onMouseDown={handleFocus}
		>
			{/* Pane indicator — shows agent ID when active */}
			{agentId && (
				<div className="absolute top-0 left-0 right-0 h-4 bg-gray-800/50 flex items-center px-2 pointer-events-none">
					<span className="text-xs text-gray-400 truncate">{agentId}</span>
				</div>
			)}
		</div>
	);
}
