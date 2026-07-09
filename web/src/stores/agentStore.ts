/**
 * Agent store — Zustand state for agent management.
 */

import { create } from "zustand";
import type { AgentState, Profile } from "@/types";

interface AgentStore {
	agents: Map<string, AgentState>;
	profiles: Profile[];
	focusedAgent: string | null;

	setAgents: (agents: Map<string, AgentState>) => void;
	addAgent: (agent: AgentState) => void;
	updateAgentStatus: (agentId: string, status: AgentState["status"]) => void;
	updateAgentOutput: (agentId: string, bytes: number) => void;
	removeAgent: (agentId: string) => void;
	setProfiles: (profiles: Profile[]) => void;
	setFocusedAgent: (agentId: string | null) => void;
	togglePin: (agentId: string) => void;
	addNote: (agentId: string, note: string) => void;
}

export const useAgentStore = create<AgentStore>((set) => ({
	agents: new Map(),
	profiles: [],
	focusedAgent: null,

	setAgents: (agents) => set({ agents }),
	addAgent: (agent) =>
		set((state) => {
			const newAgents = new Map(state.agents);
			newAgents.set(agent.id, agent);
			return { agents: newAgents };
		}),
	updateAgentStatus: (agentId, status) =>
		set((state) => {
			const agent = state.agents.get(agentId);
			if (!agent) return state;
			const updated = { ...agent, status };
			const newAgents = new Map(state.agents);
			newAgents.set(agentId, updated);
			return { agents: newAgents };
		}),
	updateAgentOutput: (agentId, bytes) =>
		set((state) => {
			const agent = state.agents.get(agentId);
			if (!agent) return state;
			const updated = {
				...agent,
				output_bytes: agent.output_bytes + bytes,
				output_lines: agent.output_lines + Math.floor(bytes / 40),
				status: agent.status === "idle" ? "working" : agent.status,
			};
			const newAgents = new Map(state.agents);
			newAgents.set(agentId, updated);
			return { agents: newAgents };
		}),
	removeAgent: (agentId) =>
		set((state) => {
			const newAgents = new Map(state.agents);
			newAgents.delete(agentId);
			return { agents: newAgents };
		}),
	setProfiles: (profiles) => set({ profiles }),
	setFocusedAgent: (agentId) => set({ focusedAgent: agentId }),
	togglePin: (agentId) =>
		set((state) => {
			const agent = state.agents.get(agentId);
			if (!agent) return state;
			const updated = { ...agent, pinned: !agent.pinned };
			const newAgents = new Map(state.agents);
			newAgents.set(agentId, updated);
			return { agents: newAgents };
		}),
	addNote: (agentId, note) =>
		set((state) => {
			const agent = state.agents.get(agentId);
			if (!agent) return state;
			const updated = { ...agent, notes: [...agent.notes, note] };
			const newAgents = new Map(state.agents);
			newAgents.set(agentId, updated);
			return { agents: newAgents };
		}),
}));
