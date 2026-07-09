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
import type { LayoutNode } from "@/types";

interface LayoutProps {
	className?: string;
}

export function Layout({ className = "" }: LayoutProps) {
	const root = useLayoutStore((s) => s.root);
	const agents = useAgentStore((s) => s.agents);
	const setFocusedAgent = useAgentStore((s) => s.setFocusedAgent);

	useEffect(() => {
		// Fetch initial layout from Tauri backend
		(async () => {
			try {
				const layout = await invoke<LayoutNode | null>("get_layout");
				if (layout) {
					useLayoutStore.getState().setRoot(layout as LayoutNode);
				}
			} catch (err) {
				console.error("Failed to load layout:", err);
			}

			// Listen for layout-changed events
			const { listen } = await import("@tauri-apps/api/event");
			listen("layout-changed", () => {
				invoke<LayoutNode | null>("get_layout")
					.then((layout) => {
						if (layout) {
							useLayoutStore.getState().setRoot(layout);
						}
					})
					.catch(console.error);
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

	const panes = useLayoutStore.getState().panes;
	return renderNode(root, panes, agents, setFocusedAgent);
}

function renderNode(
	node: LayoutNode,
	panes: Map<number, { rect: { x: number; y: number; width: number; height: number }; agentId?: string }>,
	agents: Map<string, any>,
	setFocusedAgent: (id: string | null) => void,
): React.ReactNode {
	if (node.type === "leaf") {
		const pane = panes.get(node.id);
		if (!pane) return null;

		const agentId = pane.agentId || node.agent_id;

		return (
			<GhosttyTerminal
				key={node.id}
				paneId={node.id}
				agentId={agentId}
				rect={pane.rect || node.rect || { x: 0, y: 0, width: 800, height: 600 }}
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
				{node.left && renderNode(node.left, panes, agents, setFocusedAgent)}
				<div
					className="bg-gray-700/30"
					style={isVertical ? { width: "1px", height: "100%" } : { width: "1px", height: "100%" }}
				/>
				{node.right && renderNode(node.right, panes, agents, setFocusedAgent)}
			</div>
		);
	}

	return null;
}
