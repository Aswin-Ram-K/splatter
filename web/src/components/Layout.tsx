/**
 * Multi-pane layout component.
 *
 * Renders the BSP layout tree as nested flex containers.
 * Each leaf pane hosts a Ghostty terminal connected to a PTY agent.
 */

import { useEffect } from "react";
import { invoke } from "@tauri-apps/api/core";
import { useLayoutStore } from "@/stores/layoutStore";
import { useAgentStore } from "@/stores/agentStore";
import { GhosttyTerminal } from "./Ghostty/GhosttyTerminal";

interface LayoutProps {
	className?: string;
}

interface LayoutPane {
	rect: { x: number; y: number; width: number; height: number };
	agentId?: string;
}

export function Layout({ className = "" }: LayoutProps) {
	const root = useLayoutStore((s) => s.root);
	const panes = useLayoutStore((s) => s.panes);
	const agents = useAgentStore((s) => s.agents);
	const setFocusedAgent = useAgentStore((s) => s.setFocusedAgent);

	useEffect(() => {
		// Fetch initial layout from Tauri backend
		(async () => {
			try {
				const layout = await invoke<any>("get_layout");
				useLayoutStore.getState().setRoot(layout);
			} catch (err) {
				console.error("Failed to load layout:", err);
			}

			// Listen for layout-changed events
			const { listen } = await import("@tauri-apps/api/event");
			listen("layout-changed", () => {
				invoke<any>("get_layout").then((layout: any) => {
					useLayoutStore.getState().setRoot(layout);
				}).catch(console.error);
			});
		})();
	}, []);

	if (!root) {
		return (
			<div
				className={`flex-1 bg-[#1a1b26] flex items-center justify-center ${className}`}
			>
				<div className="text-center text-gray-500">
					<p className="text-2xl mb-2">No panes</p>
					<p className="text-sm">Press Ctrl+N to create an agent</p>
				</div>
			</div>
		);
	}

	return renderNode(root, panes, agents, setFocusedAgent);
}

function renderNode(
	node: any,
	panes: Map<number, LayoutPane>,
	agents: Map<string, any>,
	setFocusedAgent: (id: string | null) => void,
): React.ReactNode {
	if (node.type === "leaf") {
		const pane = panes.get(node.id);
		if (!pane) return null;

		const agentId = pane.agentId;

		return (
			<GhosttyTerminal
				key={node.id}
				paneId={node.id}
				agentId={agentId}
				rect={pane.rect}
				isFocused={false}
				onAgentSelect={(id) => setFocusedAgent(id)}
			/>
		);
	}

	if (node.type === "split") {
		const isVertical = node.direction === "vertical";
		const splitStyle: React.CSSProperties = isVertical
			? { flexDirection: "column" as const }
			: { flexDirection: "row" as const };

		return (
			<div style={splitStyle} className="flex h-full">
				{renderNode(node.left, panes, agents, setFocusedAgent)}
				<div
					className="bg-gray-700/30 w-[1px]"
					style={!isVertical ? undefined : { width: "1px", height: "100%" }}
				/>
				{renderNode(node.right, panes, agents, setFocusedAgent)}
			</div>
		);
	}

	return null;
}
