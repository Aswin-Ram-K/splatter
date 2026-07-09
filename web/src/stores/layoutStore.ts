/**
 * Layout store — Zustand state for the BSP layout tree.
 */

import { create } from "zustand";
import { invoke } from "@tauri-apps/api/core";
import type { LayoutNode } from "@/types";

interface Pane {
	rect: { x: number; y: number; width: number; height: number };
	agentId?: string;
}

interface LayoutStore {
	root: LayoutNode;
	focusedNodeId: number | null;
	panes: Map<number, Pane>;
	sidebarVisible: boolean;

	setRoot: (node: LayoutNode) => void;
	splitPane: (direction: "vertical" | "horizontal") => Promise<void>;
	closePane: (nodeId?: number) => Promise<void>;
	focusPane: (nodeId: number) => void;
	focusDirection: (direction: "left" | "right" | "up" | "down") => void;
	toggleSidebar: () => void;
	setPaneAgent: (paneId: number, agentId: string) => void;
	getPane: (nodeId: number) => Pane | undefined;
}

export const useLayoutStore = create<LayoutStore>((set, get) => ({
	root: { type: "leaf", id: 1, rect: { x: 0, y: 0, width: 1280, height: 720 } },
	focusedNodeId: 1,
	panes: new Map([[1, { rect: { x: 0, y: 0, width: 1280, height: 720 } }]]),
	sidebarVisible: true,

	setRoot: (node) => {
		// Extract all panes from the tree
		const panes = new Map<number, Pane>();
		const extractPanes = (n: LayoutNode) => {
			if (n.type === "leaf" && n.rect) {
				panes.set(n.id, { rect: n.rect, agentId: n.agent_id });
			}
			if (n.left) extractPanes(n.left);
			if (n.right) extractPanes(n.right);
		};
		extractPanes(node);
		const firstLeafId = panes.size > 0 ? Array.from(panes.keys())[0] : 1;
		set({ root: node, panes, focusedNodeId: firstLeafId });
	},

	splitPane: async (direction: "vertical" | "horizontal") => {
		try {
			await invoke<number>("split_pane", {
				direction,
				ratio: 0.5,
			});
		} catch (err) {
			console.error("Failed to split pane:", err);
		}
	},

	closePane: async (nodeId?: number) => {
		const id = nodeId || get().focusedNodeId;
		if (!id) return;

		try {
			const result = await invoke<boolean>("close_pane", { node_id: id });
			if (result) {
				const layout = await invoke<LayoutNode | null>("get_layout");
				if (layout) {
					useLayoutStore.getState().setRoot(layout);
				}
			}
		} catch (err) {
			console.error("Failed to close pane:", err);
		}
	},

	focusPane: (nodeId) => set({ focusedNodeId: nodeId }),
	focusDirection: (direction) => {
		const state = get();
		const panes = state.panes;
		const ids = Array.from(panes.keys());
		if (ids.length === 0) return;

		const currentIdx = ids.indexOf(state.focusedNodeId || -1);
		const nextIdx =
			direction === "right" || direction === "down"
				? (currentIdx + 1) % ids.length
				: (currentIdx - 1 + ids.length) % ids.length;
		set({ focusedNodeId: ids[nextIdx] });
	},
	toggleSidebar: () =>
		set((state) => ({ sidebarVisible: !state.sidebarVisible })),
	setPaneAgent: (paneId, agentId) =>
		set((state) => {
			const panes = new Map(state.panes);
			const pane = panes.get(paneId);
			if (pane) {
				panes.set(paneId, { ...pane, agentId });
			}
			return { panes };
		}),
	getPane: (nodeId) => get().panes.get(nodeId),
}));
