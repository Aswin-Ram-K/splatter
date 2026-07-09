/**
 * Main App component.
 */

import { useEffect, useState } from "react";
import { listen } from "@tauri-apps/api/event";
import { invoke } from "@tauri-apps/api/core";
import { AgentList } from "./components/AgentList";
import { Layout } from "./components/Layout";
import { StatusBar } from "./components/StatusBar";
import { Settings } from "./components/Settings";
import { useAgentStore } from "./stores/agentStore";
import { useLayoutStore } from "./stores/layoutStore";

export default function App() {
	const [settingsVisible, setSettingsVisible] = useState(false);
	const listProfiles = useAgentStore((s) => s.setProfiles);
	const setRoot = useLayoutStore((s) => s.setRoot);
	const addAgent = useAgentStore((s) => s.addAgent);
	const setPaneAgent = useLayoutStore((s) => s.setPaneAgent);

	useEffect(() => {
		// Load profiles from Tauri backend
		invoke<string[]>("list_profiles")
			.then((profiles: string[]) => {
				listProfiles(
					profiles.map((id) => ({
						id,
						name: id,
						description: "",
						command: "",
						args: [],
						env: {},
					})),
				);
			})
			.catch(console.error);

		// Set up agent-spawned event listener
		listen(
			"agent-spawned",
			(event: { payload: { agent_id: string; layout_node_id?: number } }) => {
				const { agent_id, layout_node_id } = event.payload;
				// Create agent state
				addAgent({
					id: agent_id,
					profile_id: "pi-agent",
					status: "idle",
					started_at: new Date().toISOString(),
					duration_ms: 0,
					output_bytes: 0,
					output_lines: 0,
					cols: 80,
					rows: 24,
					notes: [],
					activity_log: [],
					pinned: false,
					groups: [],
					tags: [],
				});

				// Associate agent with layout node
				if (layout_node_id) {
					setPaneAgent(layout_node_id, agent_id);
				}
			},
		);

		// Listen for layout-changed events
		listen("layout-changed", () => {
			invoke<any>("get_layout")
				.then((layout: any) => {
					setRoot(layout);
				})
				.catch(console.error);
		});

		// Create initial single-pane layout with agent
		invoke<string>("new_pane", { profile_id: "pi-agent" })
			.then((agent_id: string) => {
				useAgentStore.getState().addAgent({
					id: agent_id,
					profile_id: "pi-agent",
					status: "idle",
					started_at: new Date().toISOString(),
					duration_ms: 0,
					output_bytes: 0,
					output_lines: 0,
					cols: 80,
					rows: 24,
					notes: [],
					activity_log: [],
					pinned: false,
					groups: [],
					tags: [],
				});
			})
			.catch(console.error);
	}, [listProfiles, setRoot, addAgent, setPaneAgent]);

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
			{/* Settings Modal */}
			<Settings visible={settingsVisible} onClose={() => setSettingsVisible(false)} />
		</div>
	);
}
