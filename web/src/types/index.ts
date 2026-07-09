/**
 * Types for Splatter frontend.
 */

export interface AgentState {
	id: string;
	profile_id: string;
	status: AgentStatus;
	started_at: string;
	duration_ms: number;
	output_bytes: number;
	output_lines: number;
	cols: number;
	rows: number;
	notes: string[];
	activity_log: ActivityEntry[];
	pinned: boolean;
	groups: string[];
	tags: string[];
}

export type AgentStatus =
	| "launching"
	| "idle"
	| "working"
	| "blocked"
	| "done"
	| "error";

export interface ActivityEntry {
	timestamp: string;
	event: ActivityEvent;
}

export interface ActivityEvent {
	event_type: string;
	[key: string]: unknown;
}

export interface Profile {
	id: string;
	name: string;
	description?: string;
	command: string;
	args: string[];
	env: Record<string, string>;
	cwd?: string;
	scrollback?: number;
	cols?: number;
	rows?: number;
}

export interface LayoutNode {
	type: "leaf" | "split";
	id: number;
	direction?: "horizontal" | "vertical";
	ratio?: number;
	agent_id?: string;
	rect?: { x: number; y: number; width: number; height: number };
	left?: LayoutNode;
	right?: LayoutNode;
}

export interface TerminalSettings {
	font_family: string;
	font_size: number;
	scrollback: number;
	theme: string;
	cursor_style: string;
	mouse_tracking: boolean;
	bracketed_paste: boolean;
	input_batch_delay_ms: number;
}

export interface AppSettings {
	terminal: TerminalSettings;
	agents: {
		max_sessions: number;
		output_buffer_mb: number;
		auto_focus_on_spawn: boolean;
		show_agent_list: boolean;
	};
	notifications: {
		enabled: boolean;
		sound: boolean;
		focus_when_focused: boolean;
		coalesce_window_seconds: number;
		triggers: string[];
	};
	hotkeys: Record<string, string>;
	crash_reporting: {
		enabled: boolean;
		dsn: string;
	};
}

export interface TrayStatus {
	working: number;
	done: number;
	blocked: number;
	error: number;
}
