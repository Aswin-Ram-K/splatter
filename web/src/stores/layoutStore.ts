/**
 * Layout store — Zustand state for the BSP layout tree.
 */

import { create } from "zustand";
import { invoke } from "@tauri-apps/api/core";
import type { LayoutNode } from "@/types";

interface LayoutStore {
	root: LayoutNode | null;
	focusedNodeId: number | null;
	panes: Map<number, Pane>;
	sidebarVisible: boolean;

	setRoot: (node: LayoutNode | null) => void;
	splitPane: (direction: "vertical" | "horizontal", ratio: number) => number;
	closePane: (nodeId?: number) => void;
	focusPane: (nodeId: number) => void;
	focusDirection: (direction: "left" | "right" | "up" | "down") => void;
	toggleSidebar: () => void;
	setPreset: (name: string) => void;
	setPaneAgent: (paneId: number, agentId: string) => void;
	getPane: (nodeId: number) => Pane | undefined;
}

interface Pane {
	rect: { x: number; y: number; width: number; height: number };
	agentId?: string;
}

export const useLayoutStore = create<LayoutStore>((set, get) => ({
	root: null,
	focusedNodeId: null,
	panes: new Map(),
	sidebarVisible: true,

	setRoot: (node) => set({ root: node }),

	splitPane: (direction, ratio) => {
		const state = get();
		if (!state.focusedNodeId) return 0;

		// Create new split node
		const leftPane = state.panes.get(state.focusedNodeId);
		const currentRect = leftPane?.rect || {
			x: 0,
			y: 0,
			width: 1280,
			height: 720,
		};

		let newRightRect: { x: number; y: number; width: number; height: number };

		if (direction === "vertical") {
			const splitX = Math.floor(currentRect.width * ratio);
			newRightRect = {
				x: currentRect.x + splitX,
				y: currentRect.y,
				width: currentRect.width - splitX,
				height: currentRect.height,
			};
		} else {
			const splitY = Math.floor(currentRect.height * ratio);
			newRightRect = {
				x: currentRect.x,
				y: currentRect.y + splitY,
				width: currentRect.width,
				height: currentRect.height - splitY,
			};
		}

		const newLeftRect: typeof currentRect =
			direction === "vertical"
				? {
						x: currentRect.x,
						y: currentRect.y,
						width: Math.floor(currentRect.width * ratio),
						height: currentRect.height,
					}
				: {
						x: currentRect.x,
						y: currentRect.y,
						width: currentRect.width,
						height: Math.floor(currentRect.height * ratio),
					};

		const splitNodeId = Date.now();
		const rightNodeId = Date.now() + 1;

		const newSplit: LayoutNode = {
			type: "split",
			id: splitNodeId,
			direction,
			ratio,
			left: {
				type: "leaf",
				id: rightNodeId,
				rect: newLeftRect,
			},
			right: {
				type: "leaf",
				id: rightNodeId,
				rect: newRightRect,
			},
		};

		const newPanes = new Map(state.panes);
		newPanes.set(rightNodeId, { rect: newLeftRect });
		newPanes.set(rightNodeId, { rect: newRightRect });

		if (state.root) {
			// Replace focused leaf with split
			set({
				root: newSplit,
				focusedNodeId: rightNodeId,
				panes: newPanes,
			});
		}

		// Create a new pane with an agent via new_pane
		invoke<string>("new_pane", { profile_id: "pi-agent" })
			.then((agent_id: string) => {
				// Associate agent with the new pane
				useLayoutStore.getState().setPaneAgent(rightNodeId, agent_id);
			})
			.catch((err: unknown) => {
				console.error("Failed to create pane:", err);
			});

		return rightNodeId;
	},

	closePane: (nodeId) => {
		const id = nodeId || get().focusedNodeId;
		if (!id) return;

		const state = get();
		const panes = new Map(state.panes);
		panes.delete(id);

		// Update root
		if (state.root) {
			if (state.root.type === "leaf") {
				// Can't close the only pane, just clear
				set({ root: null, focusedNodeId: null, panes });
				return;
			}
			// Find and replace the node in the tree (simplified)
			const newRoot = { ...state.root };
			if (newRoot.left?.id === id) {
				(newRoot as any).left = (newRoot as any).right;
			} else {
				(newRoot as any).right = (newRoot as any).left;
			}
			set({ root: newRoot, focusedNodeId: null, panes });
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
	setPreset: (name) => {
		if (name === "default" || name === "2x2") {
			set({ root: null, focusedNodeId: null, panes: new Map() });
		} else if (name === "horizontal-2") {
			const splitNodeId = Date.now();
			const rightNodeId = Date.now() + 1;
			const newRoot: LayoutNode = {
				type: "split",
				id: splitNodeId,
				direction: "vertical",
				ratio: 0.5,
				left: {
					type: "leaf",
					id: rightNodeId,
					rect: { x: 0, y: 0, width: 640, height: 720 },
				},
				right: {
					type: "leaf",
					id: rightNodeId,
					rect: { x: 640, y: 0, width: 640, height: 720 },
				},
			};
			set({
				root: newRoot,
				focusedNodeId: rightNodeId,
				panes: new Map([
					[rightNodeId, { rect: { x: 0, y: 0, width: 640, height: 720 } }],
				]),
			});
		} else if (name === "vertical-2") {
			const splitNodeId = Date.now();
			const rightNodeId = Date.now() + 1;
			const newRoot: LayoutNode = {
				type: "split",
				id: splitNodeId,
				direction: "horizontal",
				ratio: 0.5,
				left: {
					type: "leaf",
					id: rightNodeId,
					rect: { x: 0, y: 0, width: 1280, height: 360 },
				},
				right: {
					type: "leaf",
					id: rightNodeId,
					rect: { x: 0, y: 360, width: 1280, height: 360 },
				},
			};
			set({
				root: newRoot,
				focusedNodeId: rightNodeId,
				panes: new Map([
					[rightNodeId, { rect: { x: 0, y: 0, width: 1280, height: 360 } }],
				]),
			});
		}
	},
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
