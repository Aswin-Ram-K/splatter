/**
 * Ghostty terminal wrapper component.
 */

import { useEffect, useRef } from "react";
import { useGhostty } from "@/hooks/useGhostty";

interface GhosttyTerminalProps {
	paneId: number;
	agentId?: string;
	agentState?: any;
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
	const { writeOutput } = useGhostty({
		cols: Math.floor(rect.width / 8),
		rows: Math.floor(rect.height / 16),
		agentId: agentId ?? "",
	});

	// Forward PTY output to terminal
	useEffect(() => {
		if (!agentId || !writeOutput) return;

		const interval = setInterval(() => {
			// Read output from PTY (mock for now)
			// In production, this would be triggered by Tauri IPC events
		}, 100);

		return () => clearInterval(interval);
	}, [agentId, writeOutput]);

	return (
		<div
			ref={containerRef}
			style={{
				flex: 1,
				overflow: "hidden",
				position: "relative",
				border: isFocused ? "1px solid #7aa2f7" : "1px solid transparent",
			}}
			tabIndex={0}
		>
			{/* Terminal canvas rendered by ghostty-web */}
			{/* Pane indicator */}
			{agentId && (
				<div className="absolute top-0 left-0 right-0 h-4 bg-gray-800/50 flex items-center px-2">
					<span className="text-xs text-gray-400">{agentId}</span>
				</div>
			)}
		</div>
	);
}
