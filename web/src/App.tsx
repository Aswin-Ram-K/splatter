/**
 * Main App component.
 */

import { useEffect } from "react";
import { AgentList } from "./components/AgentList";
import { Layout } from "./components/Layout";
import { StatusBar } from "./components/StatusBar";
import { useAgentStore } from "./stores/agentStore";
import { useLayoutStore } from "./stores/layoutStore";

export default function App() {
	const listProfiles = useAgentStore((s) => s.setProfiles);
	const setRoot = useLayoutStore((s) => s.setRoot);

	useEffect(() => {
		// Load profiles from Tauri backend
		// listProfiles(...);

		// Set default layout (single pane)
		setRoot(null);
	}, [listProfiles, setRoot]);

	return (
		<div className="h-screen w-screen flex flex-col bg-[#1a1b26] text-gray-200">
			{/* Main content */}
			<div className="flex-1 flex min-h-0">
				{/* Sidebar */}
				<AgentList />
				{/* Terminal Grid */}
				<Layout className="flex-1 min-w-0" />
			</div>
			{/* Status Bar */}
			<StatusBar />
		</div>
	);
}
