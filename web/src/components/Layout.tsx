/**
 * Multi-pane layout component.
 *
 * Renders the BSP layout tree as nested flex containers.
 */

import { useLayoutStore } from "@/stores/layoutStore";
import { useAgentStore } from "@/stores/agentStore";
import { GhosttyTerminal } from "./Ghostty/GhosttyTerminal";

interface LayoutProps {
	className?: string;
}

export function Layout({ className = "" }: LayoutProps) {
	const root = useLayoutStore((s) => s.root);
	const focusedNodeId = useLayoutStore((s) => s.focusedNodeId);
	const panes = useLayoutStore((s) => s.panes);
	const agents = useAgentStore((s) => s.agents);
	const setFocusedAgent = useAgentStore((s) => s.setFocusedAgent);

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

	return renderNode(root, panes, agents, focusedNodeId, setFocusedAgent);
}

function renderNode(
	node: any,
	panes: Map<number, any>,
	agents: Map<string, any>,
	focusedNodeId: number | null,
	setFocusedAgent: (id: string | null) => void,
): React.ReactNode {
	if (node.type === "leaf") {
		const pane = panes.get(node.id);
		if (!pane) return null;

		const agentId = pane.agentId;
		const agent = agentId ? agents.get(agentId) : null;

		return (
			<GhosttyTerminal
				key={node.id}
				paneId={node.id}
				agentId={agentId}
				agentState={agent}
				rect={pane.rect}
				isFocused={focusedNodeId === node.id}
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
				{renderNode(node.left, panes, agents, focusedNodeId, setFocusedAgent)}
				<div
					className="bg-gray-700/30 w-[1px]"
					style={!isVertical ? undefined : { width: "1px", height: "100%" }}
				/>
				{renderNode(node.right, panes, agents, focusedNodeId, setFocusedAgent)}
			</div>
		);
	}

	return null;
}
