/**
 * Agent list sidebar component.
 */

import { useState } from "react";
import { useAgentStore } from "@/stores/agentStore";
import { useSettingsStore } from "@/stores/settingsStore";

export function AgentList() {
	const agents = useAgentStore((s) => s.agents);
	const focusedAgent = useAgentStore((s) => s.focusedAgent);
	const setFocusedAgent = useAgentStore((s) => s.setFocusedAgent);
	const sidebarVisible = useSettingsStore(
		(s) => s.settings.agents.show_agent_list,
	);

	const [searchQuery, setSearchQuery] = useState("");

	const statusColors: Record<string, string> = {
		launching: "bg-yellow-500",
		idle: "bg-gray-400",
		working: "bg-green-500",
		blocked: "bg-red-500",
		done: "bg-blue-500",
		error: "bg-red-600",
	};

	const filteredAgents = Array.from(agents.values()).filter((a) => {
		if (
			searchQuery &&
			!a.profile_id.toLowerCase().includes(searchQuery.toLowerCase())
		) {
			return false;
		}
		return true;
	});

	if (!sidebarVisible) return null;

	return (
		<div className="w-56 bg-gray-900 border-r border-white/10 flex flex-col h-full">
			{/* Header */}
			<div className="p-3 border-b border-white/10">
				<h2 className="text-sm font-semibold text-gray-200">Agents</h2>
				{/* Search */}
				<input
					type="text"
					placeholder="Search..."
					className="mt-2 w-full px-2 py-1 text-xs bg-gray-800 border border-white/10 rounded text-gray-300"
					value={searchQuery}
					onChange={(e) => setSearchQuery(e.target.value)}
				/>
			</div>

			{/* Agent list */}
			<div className="flex-1 overflow-y-auto">
				{filteredAgents.map((agent) => (
					<AgentItem
						key={agent.id}
						agent={agent}
						isFocused={focusedAgent === agent.id}
						onSelect={() => setFocusedAgent(agent.id)}
						statusColors={statusColors}
					/>
				))}
			</div>

			{/* New Agent button */}
			<div className="p-2 border-t border-white/10">
				<button className="w-full px-3 py-2 text-xs bg-blue-600 hover:bg-blue-700 text-white rounded">
					+ New Agent
				</button>
			</div>
		</div>
	);
}

interface AgentItemProps {
	agent: any;
	isFocused: boolean;
	onSelect: () => void;
	statusColors: Record<string, string>;
}

function AgentItem({
	agent,
	isFocused,
	onSelect,
	statusColors,
}: AgentItemProps) {
	return (
		<div
			className={`flex items-center gap-2 px-3 py-2 cursor-pointer ${
				isFocused ? "bg-white/10" : "hover:bg-white/5"
			}`}
			onClick={onSelect}
		>
			{/* Status dot */}
			<div
				className={`w-2 h-2 rounded-full ${statusColors[agent.status] || "bg-gray-400"}`}
			/>
			{/* Pinned indicator */}
			{agent.pinned && <span className="text-xs">📌</span>}
			{/* Agent name */}
			<span className="text-xs text-gray-300 truncate flex-1">
				{agent.profile_id}
			</span>
		</div>
	);
}
