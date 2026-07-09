/**
 * Plugin list component — Shows available plugins with toggle.
 */

import { useEffect, useState } from "react";
import { invoke } from "@tauri-apps/api/core";

interface PluginItem {
	name: string;
	enabled: boolean;
	state: string;
}

export function PluginList() {
	const [plugins, setPlugins] = useState<PluginItem[]>([]);
	const [loading, setLoading] = useState(true);

	useEffect(() => {
		// Load plugins from Tauri backend
		invoke<string[]>("list_plugins")
			.then((list: string[]) => {
				const items = list.map((name) => ({
					name,
					enabled: true,
					state: "Ready",
				}));
				setPlugins(items);
				setLoading(false);
			})
			.catch((err) => {
				console.error("Failed to load plugins:", err);
				setLoading(false);
			});
	}, []);

	const togglePlugin = async (name: string, enabled: boolean) => {
		try {
			const result = await invoke<boolean>("toggle_plugin", { name, enabled });
			if (result) {
				setPlugins((prev) =>
					prev.map((p) => (p.name === name ? { ...p, enabled } : p)),
				);
			}
		} catch (err) {
			console.error("Failed to toggle plugin:", err);
		}
	};

	if (loading) {
		return <div className="p-4 text-gray-400">Loading plugins...</div>;
	}

	return (
		<div className="p-4">
			{plugins.length === 0 ? (
				<div className="text-center text-gray-500 py-8">
					<p>No plugins installed</p>
					<p className="text-xs mt-2">
						Place plugin folders in your plugins directory
					</p>
				</div>
			) : (
				<ul className="space-y-2">
					{plugins.map((plugin) => (
						<li
							key={plugin.name}
							className="flex items-center justify-between bg-[#1e1f29] rounded p-3"
						>
							<div>
								<div className="font-medium text-white">{plugin.name}</div>
								<div className="text-xs text-gray-400">
									Status: {plugin.state}
								</div>
							</div>
							<button
								onClick={() => togglePlugin(plugin.name, !plugin.enabled)}
								className={`px-3 py-1 rounded text-sm font-medium transition-colors ${
									plugin.enabled
										? "bg-green-600 text-white"
										: "bg-gray-600 text-gray-300"
								}`}
							>
								{plugin.enabled ? "On" : "Off"}
							</button>
						</li>
					))}
				</ul>
			)}
		</div>
	);
}
