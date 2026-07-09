/**
 * Settings component — Renders the full settings UI.
 */

import { useEffect } from "react";
import { invoke } from "@tauri-apps/api/core";
import { useSettingsStore } from "@/stores/settingsStore";
import { PluginList } from "./PluginList";

interface SettingsProps {
	visible: boolean;
	onClose: () => void;
}

export function Settings({ visible, onClose }: SettingsProps) {
	const settings = useSettingsStore((s) => s.settings);
	const updateSettings = useSettingsStore((s) => s.updateSettings);

	useEffect(() => {
		if (visible) {
			// Load settings from Tauri backend
			invoke<any>("get_config")
				.then((config: any) => {
					updateSettings(config.settings || {});
				})
				.catch(console.error);
		}
	}, [visible, updateSettings]);

	if (!visible) return null;

	const saveSettings = async () => {
		try {
			await invoke("save_config", {
				settings: {
					terminal: settings.terminal,
					agents: settings.agents,
					notifications: settings.notifications,
					hotkeys: settings.hotkeys,
					crash_reporting: settings.crash_reporting,
				},
			});
			onClose();
		} catch (err) {
			console.error("Failed to save settings:", err);
		}
	};

	return (
		<div className="fixed inset-0 z-50 flex items-center justify-center bg-black/50">
			<div className="bg-[#24283b] rounded-lg shadow-xl w-full max-w-2xl max-h-[80vh] overflow-y-auto">
				{/* Header */}
				<div className="flex items-center justify-between p-4 border-b border-gray-700">
					<h2 className="text-lg font-semibold text-white">Settings</h2>
					<button onClick={onClose} className="text-gray-400 hover:text-white">
						✕
					</button>
				</div>

				{/* Tabs */}
				<div className="flex border-b border-gray-700">
					{["terminal", "agents", "notifications", "hotkeys", "plugins"].map(
						(tab) => (
							<button
								key={tab}
								className="px-4 py-2 text-sm font-medium text-gray-300 hover:text-white border-b-2 border-transparent hover:border-blue-500"
							>
								{tab.charAt(0).toUpperCase() + tab.slice(1)}
							</button>
						),
					)}
				</div>

				{/* Settings Content */}
				<div className="p-6 space-y-6">
					{/* Terminal Settings */}
					<SettingsSection title="Terminal">
						<SettingsInput
							label="Font Family"
							value={settings.terminal.font_family}
							onChange={(v) =>
								updateSettings({
									terminal: { ...settings.terminal, font_family: v },
								})
							}
						/>
						<SettingsInput
							label="Font Size"
							value={settings.terminal.font_size}
							type="number"
							onChange={(v) =>
								updateSettings({
									terminal: {
										...settings.terminal,
										font_size: parseInt(v) || 15,
									},
								})
							}
						/>
						<SettingsInput
							label="Scrollback Buffer (lines)"
							value={settings.terminal.scrollback}
							type="number"
							onChange={(v) =>
								updateSettings({
									terminal: {
										...settings.terminal,
										scrollback: parseInt(v) || 10000,
									},
								})
							}
						/>
						<SettingsToggle
							label="Cursor Blink"
							checked={true}
							onChange={() => {}}
						/>
						<SettingsToggle
							label="Mouse Tracking"
							checked={settings.terminal.mouse_tracking}
							onChange={() =>
								updateSettings({
									terminal: {
										...settings.terminal,
										mouse_tracking: !settings.terminal.mouse_tracking,
									},
								})
							}
						/>
						<SettingsToggle
							label="Bracketed Paste"
							checked={settings.terminal.bracketed_paste}
							onChange={() =>
								updateSettings({
									terminal: {
										...settings.terminal,
										bracketed_paste: !settings.terminal.bracketed_paste,
									},
								})
							}
						/>
					</SettingsSection>

					{/* Plugins Section }*
						<SettingsSection title="Plugins">
							<PluginList />
						</SettingsSection>

					{/* Agent Settings */}
					<SettingsSection title="Agents">
						<SettingsInput
							label="Max Sessions"
							value={settings.agents.max_sessions}
							type="number"
							onChange={(v) =>
								updateSettings({
									agents: {
										...settings.agents,
										max_sessions: parseInt(v) || 50,
									},
								})
							}
						/>
						<SettingsInput
							label="Output Buffer (MB)"
							value={settings.agents.output_buffer_mb}
							type="number"
							onChange={(v) =>
								updateSettings({
									agents: {
										...settings.agents,
										output_buffer_mb: parseInt(v) || 512,
									},
								})
							}
						/>
						<SettingsToggle
							label="Auto Focus on Spawn"
							checked={settings.agents.auto_focus_on_spawn}
							onChange={() =>
								updateSettings({
									agents: {
										...settings.agents,
										auto_focus_on_spawn: !settings.agents.auto_focus_on_spawn,
									},
								})
							}
						/>
					</SettingsSection>

					{/* Plugins Section }*
						<SettingsSection title="Plugins">
							<PluginList />
						</SettingsSection>

					{/* Notification Settings */}
					<SettingsSection title="Notifications">
						<SettingsToggle
							label="Enable Notifications"
							checked={settings.notifications.enabled}
							onChange={() =>
								updateSettings({
									notifications: {
										...settings.notifications,
										enabled: !settings.notifications.enabled,
									},
								})
							}
						/>
						<SettingsToggle
							label="Play Sound"
							checked={settings.notifications.sound}
							onChange={() =>
								updateSettings({
									notifications: {
										...settings.notifications,
										sound: !settings.notifications.sound,
									},
								})
							}
						/>
						<SettingsInput
							label="Coalesce Window (seconds)"
							value={settings.notifications.coalesce_window_seconds}
							type="number"
							onChange={(v) =>
								updateSettings({
									notifications: {
										...settings.notifications,
										coalesce_window_seconds: parseInt(v) || 30,
									},
								})
							}
						/>
					</SettingsSection>

					{/* Plugins Section }*
						<SettingsSection title="Plugins">
							<PluginList />
						</SettingsSection>


					{/* Plugins Section */}
					<SettingsSection title="Plugins">
						<PluginList />
					</SettingsSection>
				</div>
				{/* Hotkey Settings */}
				<SettingsSection title="Hotkeys">
					{Object.entries(settings.hotkeys).map(([key, value]) => (
						<SettingsInput
							key={key}
							label={key
								.replace(/_/g, " ")
								.replace(/\b\w/g, (l) => l.toUpperCase())}
							value={value}
							onChange={(v) =>
								updateSettings({
									hotkeys: { ...settings.hotkeys, [key]: v },
								})
							}
						/>
					))}
				</SettingsSection>

				{/* Plugins Section }*
						<SettingsSection title="Plugins">
							<PluginList />
						</SettingsSection>
				</div>

				{/* Footer */}
				<div className="flex justify-end gap-2 p-4 border-t border-gray-700">
					<button
						onClick={onClose}
						className="px-4 py-2 text-sm font-medium text-gray-300 hover:text-white"
					>
						Cancel
					</button>
					<button
						onClick={saveSettings}
						className="px-4 py-2 text-sm font-medium bg-blue-600 text-white rounded hover:bg-blue-700"
					>
						Save
					</button>
				</div>
			</div>
		</div>
	);
}

function SettingsSection({
	title,
	children,
}: {
	title: string;
	children: React.ReactNode;
}) {
	return (
		<div className="space-y-3">
			<h3 className="text-sm font-semibold text-gray-200">{title}</h3>
			<div className="grid grid-cols-1 gap-3">{children}</div>
		</div>
	);
}

function SettingsInput({
	label,
	value,
	type = "text",
	onChange,
}: {
	label: string;
	value: string | number;
	type?: string;
	onChange: (value: string) => void;
}) {
	return (
		<div className="flex items-center justify-between">
			<label className="text-sm text-gray-300">{label}</label>
			<input
				type={type}
				value={value}
				onChange={(e) => onChange(e.target.value)}
				className="w-32 bg-[#1a1b26] border border-gray-600 rounded px-2 py-1 text-sm text-white"
			/>
		</div>
	);
}

function SettingsToggle({
	label,
	checked,
	onChange,
}: {
	label: string;
	checked: boolean;
	onChange: () => void;
}) {
	return (
		<div className="flex items-center justify-between">
			<label className="text-sm text-gray-300">{label}</label>
			<button
				onClick={onChange}
				className={`relative inline-flex h-6 w-11 items-center rounded-full transition-colors ${
					checked ? "bg-blue-600" : "bg-gray-600"
				}`}
			>
				<span
					className={`inline-block h-4 w-4 transform rounded-full bg-white transition-transform ${
						checked ? "translate-x-6" : "translate-x-1"
					}`}
				/>
			</button>
		</div>
	);
}
