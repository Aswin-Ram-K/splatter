/**
 * Main App component.
 */

import { useEffect, useState, useRef } from "react";
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
	const spawnedRef = useRef(false);

	// Debug: log app initialization
	useEffect(() => {
		console.log("[App] Initializing...");
		console.log("[App] location:", window.location.href);
		console.log("[App] document ready:", document.readyState);

		// Debug: verify Tauri IPC is available
		if (typeof (window as any).__TAURI__ !== "undefined") {
			console.log("[App] Tauri API available");
		} else {
			console.warn("[App] Tauri API NOT available — running in browser?");
		}
	}, []);

	useEffect(() => {
		console.log("[App] useEffect triggered");

		// Load profiles from Tauri backend
		invoke<string[]>("list_profiles")
			.then((profiles: string[]) => {
				console.log("[App] list_profiles succeeded, profiles:", profiles);
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
			.catch((err: unknown) => {
				console.error("[App] list_profiles failed:", err);
			});

		// Set up agent-spawned event listener
		listen(
			"agent-spawned",
			(event: { payload: { agent_id: string; layout_node_id?: number } }) => {
				console.log("[App] agent-spawned:", event.payload);
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

		// Create initial single-pane layout with agent (once)
		if (!spawnedRef.current) {
			spawnedRef.current = true;
			console.log("[App] Creating initial pane...");
			invoke<string>("new_pane", { profile_id: "pi-agent" })
				.then((agent_id: string) => {
					console.log("[App] new_pane succeeded, agent_id:", agent_id);
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
				})
				.catch((err: unknown) => {
					console.error("[App] new_pane failed:", err);
				});
		}
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
			<Settings
				visible={settingsVisible}
				onClose={() => setSettingsVisible(false)}
			/>
		</div>
	);
}
