/**
 * Status bar component.
 */

import { useAgentStore } from "@/stores/agentStore";
import { useLayoutStore } from "@/stores/layoutStore";

export function StatusBar() {
	const agents = useAgentStore((s) => s.agents);
	const focusedNodeId = useLayoutStore((s) => s.focusedNodeId);
	const panes = useLayoutStore((s) => s.panes);

	const workingCount = Array.from(agents.values()).filter(
		(a) => a.status === "working",
	).length;

	const doneCount = Array.from(agents.values()).filter(
		(a) => a.status === "done",
	).length;

	return (
		<div className="h-6 bg-gray-900 border-t border-white/10 flex items-center px-3 text-xs text-gray-400">
			{/* Left side */}
			<div className="flex items-center gap-4">
				<span>{workingCount} working</span>
				<span>{doneCount} done</span>
				<span>{panes.size} panes</span>
			</div>

			{/* Center */}
			<div className="flex-1" />

			{/* Right side */}
			<div className="flex items-center gap-4">
				{focusedNodeId && <span>Pane {focusedNodeId}</span>}
				<span>Splatter v0.1.0</span>
			</div>
		</div>
	);
}
