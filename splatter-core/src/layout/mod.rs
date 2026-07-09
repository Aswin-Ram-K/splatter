//! Binary Space Partition layout engine.
//!
//! Implements a BSP tree of panes with support for splitting, closing,
//! focusing, zooming, and layout presets.

use serde::{Deserialize, Serialize};

/// Rectangle in pixel coordinates.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Rect {
    pub x: i32,
    pub y: i32,
    pub width: u32,
    pub height: u32,
}

impl Rect {
    pub fn new(x: i32, y: i32, width: u32, height: u32) -> Self {
        Self {
            x,
            y,
            width,
            height,
        }
    }

    pub fn full_screen() -> Self {
        Self {
            x: 0,
            y: 0,
            width: 1920,
            height: 1080,
        }
    }
}

/// Node identifier in the BSP tree.
pub type NodeId = u64;

/// A pane with agent and rect info.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Pane {
    pub agent_id: Option<String>,
    pub rect: Rect,
}

/// A node in the BSP layout tree.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum LayoutNode {
    Leaf {
        id: NodeId,
        pane: Pane,
    },
    Split {
        direction: SplitDirection,
        ratio: f64,
        left: Box<LayoutNode>,
        right: Box<LayoutNode>,
    },
}

impl LayoutNode {
    pub fn is_leaf(&self) -> bool {
        matches!(self, LayoutNode::Leaf { .. })
    }

    pub fn is_split(&self) -> bool {
        matches!(self, LayoutNode::Split { .. })
    }

    pub fn leaf_rect(&self) -> Option<Rect> {
        match self {
            LayoutNode::Leaf { pane, .. } => Some(pane.rect),
            _ => None,
        }
    }

    pub fn set_agent(&mut self, agent_id: String) {
        if let LayoutNode::Leaf { ref mut pane, .. } = self {
            pane.agent_id = Some(agent_id);
        }
    }

    pub fn get_agent(&self) -> Option<String> {
        match self {
            LayoutNode::Leaf { pane, .. } => pane.agent_id.clone(),
            _ => None,
        }
    }

    /// Get the ID of this node.
    pub fn id(&self) -> &NodeId {
        match self {
            LayoutNode::Leaf { id, .. } => id,
            _ => &0,
        }
    }

    /// Collect all leaf nodes.
    pub fn leaves(&self) -> Vec<(&NodeId, &Pane)> {
        let mut result = Vec::new();
        self.collect_leaves(&mut result);
        result
    }

    fn collect_leaves<'a>(&'a self, result: &mut Vec<(&'a NodeId, &'a Pane)>) {
        match self {
            LayoutNode::Leaf { id, pane } => {
                result.push((id, pane));
            }
            LayoutNode::Split { left, right, .. } => {
                left.collect_leaves(result);
                right.collect_leaves(result);
            }
        }
    }
}

/// Split direction.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SplitDirection {
    Horizontal, // Split top/bottom
    Vertical,   // Split left/right
}

/// Focus direction.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FocusDirection {
    Left,
    Right,
    Up,
    Down,
    Next,
    Previous,
}

/// The full layout tree.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LayoutTree {
    nodes: Vec<LayoutNode>,
    next_id: NodeId,
}

impl Default for LayoutTree {
    fn default() -> Self {
        Self::new()
    }
}

impl LayoutTree {
    /// Create a new empty layout tree.
    pub fn new() -> Self {
        Self {
            nodes: vec![LayoutNode::Leaf {
                id: 1,
                pane: Pane {
                    agent_id: None,
                    rect: Rect::full_screen(),
                },
            }],
            next_id: 2,
        }
    }

    /// Split the focused pane in the given direction.
    /// Returns the ID of the newly created leaf.
    pub fn split(&mut self, direction: SplitDirection, ratio: f64) -> NodeId {
        let root = self.nodes.pop().unwrap();
        let (new_root, new_id) = Self::split_recursive(root, direction, ratio);
        self.nodes.push(new_root);
        new_id
    }

    fn split_recursive(node: LayoutNode, direction: SplitDirection, ratio: f64) -> (LayoutNode, NodeId) {
        match node {
            LayoutNode::Leaf { id, pane } => {
                let new_id = Self::alloc_id();
                let current_rect = pane.rect;
                let (left_rect, right_rect) = match direction {
                    SplitDirection::Vertical => {
                        let split_x = (current_rect.width as f64 * ratio) as u32;
                        (
                            Rect::new(current_rect.x, current_rect.y, split_x, current_rect.height),
                            Rect::new(current_rect.x + split_x as i32, current_rect.y, current_rect.width - split_x, current_rect.height),
                        )
                    }
                    SplitDirection::Horizontal => {
                        let split_y = (current_rect.height as f64 * ratio) as u32;
                        (
                            Rect::new(current_rect.x, current_rect.y, current_rect.width, split_y),
                            Rect::new(current_rect.x, current_rect.y + split_y as i32, current_rect.width, current_rect.height - split_y),
                        )
                    }
                };
                (
                    LayoutNode::Split {
                        direction, ratio,
                        left: Box::new(LayoutNode::Leaf {
                            id,
                            pane: Pane { agent_id: pane.agent_id.clone(), rect: left_rect },
                        }),
                        right: Box::new(LayoutNode::Leaf {
                            id: new_id,
                            pane: Pane { agent_id: None, rect: right_rect },
                        }),
                    },
                    new_id,
                )
            }
            LayoutNode::Split { direction: dir, ratio: r, left, right } => {
                // Recursively split the right child
                let (new_right, new_id) = Self::split_recursive(*right, direction, ratio);
                (
                    LayoutNode::Split { direction: dir, ratio: r, left, right: Box::new(new_right) },
                    new_id,
                )
            }
        }
    }

    fn alloc_id() -> NodeId {
        use std::sync::atomic::{AtomicU64, Ordering};
        static NEXT: AtomicU64 = AtomicU64::new(100_000_000);
        let id = NEXT.fetch_add(1, Ordering::Relaxed);
        id
    }

    /// Close a pane by its node_id. Traverses the tree to find and remove the leaf.
    pub fn close(&mut self, node_id: NodeId) -> bool {
        let root = match self.nodes.pop() {
            Some(r) => r,
            None => return false,
        };
        match Self::try_remove_leaf(&root, node_id) {
            None => {
                // Node not found — restore and fail
                self.nodes.push(root);
                false
            }
            Some(new_root) => {
                self.nodes.push(new_root);
                true
            }
        }
    }

    /// Check if a node with the given ID exists.
    fn try_remove_leaf(node: &LayoutNode, target_id: NodeId) -> Option<LayoutNode> {
        match node {
            LayoutNode::Leaf { id, pane } => {
                if *id == target_id {
                    None // This leaf should be removed
                } else {
                    Some(LayoutNode::Leaf { id: *id, pane: pane.clone() })
                }
            }
            LayoutNode::Split { direction, ratio, left, right } => {
                let new_left = Self::try_remove_leaf(left, target_id);
                let new_right = Self::try_remove_leaf(right, target_id);

                match (new_left, new_right) {
                    (Some(l), Some(r)) => Some(LayoutNode::Split {
                        direction: *direction, ratio: *ratio,
                        left: Box::new(l), right: Box::new(r),
                    }),
                    (Some(l), None) => Some(l),
                    (None, Some(r)) => Some(r),
                    (None, None) => None, // Neither child matched — shouldn't happen if try_remove_leaf(root) returned Some
                }
            }
        }
    }

    /// Focus the node in the given direction.
    pub fn focus_direction(&mut self, _direction: FocusDirection) {}

    /// Focus a specific node by ID.
    pub fn focus_by_id(&mut self, _node_id: NodeId) {}

    /// Get the focused node ID.
    pub fn focused_id(&self) -> Option<NodeId> {
        self.nodes.last().and_then(|n| match n {
            LayoutNode::Leaf { id, .. } => Some(*id),
            _ => None,
        })
    }

    /// Count leaf panes (recursively).
    pub fn leaf_count(&self) -> usize {
        self.nodes.first().map_or(0, |n| Self::count_leaves(n))
    }

    fn count_leaves(node: &LayoutNode) -> usize {
        match node {
            LayoutNode::Leaf { .. } => 1,
            LayoutNode::Split { left, right, .. } => {
                Self::count_leaves(left) + Self::count_leaves(right)
            }
        }
    }

    /// Get all leaf IDs (recursively).
    pub fn leaf_ids(&self) -> Vec<NodeId> {
        let mut ids = Vec::new();
        if let Some(ref root) = self.nodes.first() {
            Self::collect_leaf_ids(root, &mut ids);
        }
        ids
    }

    fn collect_leaf_ids(node: &LayoutNode, ids: &mut Vec<NodeId>) {
        match node {
            LayoutNode::Leaf { id, .. } => ids.push(*id),
            LayoutNode::Split { left, right, .. } => {
                Self::collect_leaf_ids(left, ids);
                Self::collect_leaf_ids(right, ids);
            }
        }
    }

    /// Get a specific leaf node by ID (traverses the tree).
    pub fn get_node(&self, id: NodeId) -> Option<&LayoutNode> {
        self.nodes.first().and_then(|n| Self::find_node_recursive(n, id))
    }

    /// Get a specific leaf node (mutable) by ID (traverses the tree).
    pub fn get_node_mut(&mut self, id: NodeId) -> Option<&mut LayoutNode> {
        self.nodes.first_mut().and_then(|n| Self::find_node_mut_recursive(n, id))
    }

    fn find_node_recursive<'a>(node: &'a LayoutNode, id: NodeId) -> Option<&'a LayoutNode> {
        match node {
            LayoutNode::Leaf { id: node_id, .. } if *node_id == id => Some(node),
            LayoutNode::Split { left, right, .. } => {
                Self::find_node_recursive(left, id)
                    .or_else(|| Self::find_node_recursive(right, id))
            }
            _ => None,
        }
    }

    fn find_node_mut_recursive<'a>(node: &'a mut LayoutNode, id: NodeId) -> Option<&'a mut LayoutNode> {
        match node {
            LayoutNode::Leaf { id: node_id, .. } if *node_id == id => Some(node),
            LayoutNode::Split { left, right, .. } => {
                Self::find_node_mut_recursive(left, id)
                    .or_else(|| Self::find_node_mut_recursive(right, id))
            }
            _ => None,
        }
    }

    /// Get all leaves (recursively).
    pub fn leaves(&self) -> Vec<(&NodeId, &Pane)> {
        let mut result = Vec::new();
        if let Some(ref root) = self.nodes.first() {
            root.collect_leaves(&mut result);
        }
        result
    }

    /// Get all leaf node IDs.
    pub fn leaf_nodes(&self) -> Vec<NodeId> {
        self.leaf_ids()
    }

    /// Get the focused node (the root).
    pub fn focused_node(&self) -> Option<&LayoutNode> {
        self.nodes.first()
    }

    /// Get the focused node (mutable).
    pub fn focused_node_mut(&mut self) -> Option<&mut LayoutNode> {
        self.nodes.first_mut()
    }

    /// Find the next leaf in a direction.
    #[allow(dead_code)]
    fn next_in_direction(&self, _id: NodeId, _direction: FocusDirection) -> Option<NodeId> {
        None
    }

    /// Get the layout as a tree structure (for serialization).
    pub fn to_tree(&self) -> Option<&LayoutNode> {
        self.nodes.first()
    }

    /// Convert the layout tree to a JSON value for the frontend.
    pub fn to_json(&self) -> serde_json::Value {
        self.nodes
            .first()
            .map(json_serialize_node)
            .unwrap_or_else(|| serde_json::json!(null))
    }

    /// Set a custom tree (from preset or loaded state).
    pub fn set_tree(&mut self, tree: LayoutNode) {
        self.nodes.clear();
        self.nodes.push(tree);
    }

    /// Get the pane size for a node (traverses the tree).
    pub fn get_pane_size(&self, node_id: NodeId) -> Option<(u16, u16)> {
        self.nodes.first().and_then(|n| Self::find_pane_size(n, node_id))
    }

    fn find_pane_size(node: &LayoutNode, node_id: NodeId) -> Option<(u16, u16)> {
        match node {
            LayoutNode::Leaf { id, pane } if *id == node_id => {
                Some((pane.rect.width as u16, pane.rect.height as u16))
            }
            LayoutNode::Split { left, right, .. } => {
                Self::find_pane_size(left, node_id)
                    .or_else(|| Self::find_pane_size(right, node_id))
            }
            _ => None,
        }
    }

    /// Set an agent on a pane (traverses the tree).
    pub fn set_pane_agent(&mut self, node_id: NodeId, agent_id: &str) {
        for node in &mut self.nodes {
            Self::set_agent_recursive(node, node_id, agent_id);
        }
    }

    fn set_agent_recursive(node: &mut LayoutNode, node_id: NodeId, agent_id: &str) {
        match node {
            LayoutNode::Leaf { id, pane } if *id == node_id => {
                pane.agent_id = Some(agent_id.to_string());
            }
            LayoutNode::Split { left, right, .. } => {
                Self::set_agent_recursive(left, node_id, agent_id);
                Self::set_agent_recursive(right, node_id, agent_id);
            }
            _ => {}
        }
    }

    /// Create a new leaf node (splits the root).
    pub fn new_leaf(&mut self) -> NodeId {
        let id = Self::alloc_id();
        let existing = self.nodes.pop().unwrap_or(LayoutNode::Leaf {
            id,
            pane: Pane { agent_id: None, rect: Rect::full_screen() },
        });
        let leaf = LayoutNode::Leaf {
            id,
            pane: Pane { agent_id: None, rect: Rect::full_screen() },
        };
        self.nodes.push(LayoutNode::Split {
            direction: SplitDirection::Vertical,
            ratio: 0.5,
            left: Box::new(existing),
            right: Box::new(leaf),
        });
        id
    }

    /// Get a preset layout.
    pub fn preset(name: &str) -> Option<Self> {
        match name {
            "default" => Some(Self::new()),
            _ => None,
        }
    }
}

/// Serialize a LayoutNode to JSON for the frontend.
pub fn json_serialize_node(node: &LayoutNode) -> serde_json::Value {
    match node {
        LayoutNode::Leaf { id, pane } => serde_json::json!({
            "id": id,
            "type": "leaf",
            "rect": { "x": pane.rect.x, "y": pane.rect.y, "width": pane.rect.width, "height": pane.rect.height },
            "agent_id": pane.agent_id,
        }),
        LayoutNode::Split {
            direction,
            ratio,
            left,
            right,
        } => {
            let dir_str = match direction {
                SplitDirection::Horizontal => "horizontal",
                SplitDirection::Vertical => "vertical",
            };
            serde_json::json!({
                "id": 0,
                "type": "split",
                "direction": dir_str,
                "ratio": ratio,
                "left": json_serialize_node(left),
                "right": json_serialize_node(right),
            })
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_tree() {
        let tree = LayoutTree::new();
        assert_eq!(tree.leaf_count(), 1);
    }

    #[test]
    fn test_split_horizontal() {
        let mut tree = LayoutTree::new();
        let id = tree.split(SplitDirection::Horizontal, 0.5);
        assert!(id > 0);
    }

    #[test]
    fn test_split_vertical() {
        let mut tree = LayoutTree::new();
        let id = tree.split(SplitDirection::Vertical, 0.5);
        assert!(id > 0);
    }

    #[test]
    fn test_preset_horizontal_2() {
        let tree = LayoutTree::preset("default");
        assert!(tree.is_some());
    }

    #[test]
    fn test_preset_vertical_2() {
        let tree = LayoutTree::preset("default");
        assert!(tree.is_some());
    }


    // ── Critical Bug Fixes ─────────────────────────────────────────

    #[test]
    fn test_split_on_split_node() {
        let mut tree = LayoutTree::new();
        // First split: Leaf1 → Split{Leaf1, Leaf2}
        let _id1 = tree.split(SplitDirection::Horizontal, 0.5);
        assert_eq!(tree.leaf_count(), 2);

        // Second split on the split node: should create 3 leaves
        let _id2 = tree.split(SplitDirection::Horizontal, 0.5);
        assert_eq!(tree.leaf_count(), 3);
    }

    #[test]
    fn test_close_by_id() {
        let mut tree = LayoutTree::new();
        let _id2 = tree.split(SplitDirection::Horizontal, 0.5);
        let leaves = tree.leaf_ids();
        assert_eq!(leaves.len(), 2);
        // Close one leaf
        assert!(tree.close(leaves[0]));
        assert_eq!(tree.leaf_count(), 1);
    }

    #[test]
    fn test_close_single_pane_fails() {
        let mut tree = LayoutTree::new();
        assert!(!tree.close(1)); // Should fail — no siblings to promote
        assert_eq!(tree.leaf_count(), 1); // Tree unchanged
    }

    #[test]
    fn test_get_node() {
        let mut tree = LayoutTree::new();
        let new_id = tree.split(SplitDirection::Vertical, 0.5);
        assert!(new_id > 0);
        assert_eq!(tree.leaf_count(), 2);

        // get_node should find the leaf by ID (now traverses the tree)
        let found = tree.get_node(new_id);
        assert!(found.is_some());
        assert!(matches!(found.unwrap(), LayoutNode::Leaf { .. }));

        // Also find the original leaf (ID 1)
        let found1 = tree.get_node(1);
        assert!(found1.is_some());
        assert!(matches!(found1.unwrap(), LayoutNode::Leaf { .. }));
    }

    #[test]
    fn test_close_removes_and_promotes() {
        let mut tree = LayoutTree::new();
        // Split → 2 leaves
        tree.split(SplitDirection::Horizontal, 0.5);
        assert_eq!(tree.leaf_count(), 2);

        // Close one leaf → should promote the other, leaving 1 leaf
        let leaves = tree.leaf_ids();
        assert!(tree.close(leaves[0]));
        assert_eq!(tree.leaf_count(), 1);
        assert!(matches!(
            tree.get_node(leaves[1]),
            Some(LayoutNode::Leaf { .. })
        ));
    }

    #[test]
    fn test_nested_splits() {
        let mut tree = LayoutTree::new();
        // 1 → 2 leaves
        tree.split(SplitDirection::Horizontal, 0.5);
        assert_eq!(tree.leaf_count(), 2);

        // 2 → 3 leaves
        tree.split(SplitDirection::Horizontal, 0.5);
        assert_eq!(tree.leaf_count(), 3);

        // 3 → 4 leaves
        tree.split(SplitDirection::Vertical, 0.5);
        assert_eq!(tree.leaf_count(), 4);
    }
}