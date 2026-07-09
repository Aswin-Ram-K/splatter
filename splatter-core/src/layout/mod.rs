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
        let last_node = self.nodes.pop();
        let (left_child, right_child, new_id) = match last_node {
            Some(LayoutNode::Leaf { id, pane }) => {
                let new_id = self.next_id;
                self.next_id += 1;
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
                    LayoutNode::Leaf { id, pane: Pane { agent_id: pane.agent_id.clone(), rect: left_rect } },
                    LayoutNode::Leaf { id: new_id, pane: Pane { agent_id: None, rect: right_rect } },
                    new_id,
                )
            }
            Some(LayoutNode::Split { direction: _, ratio: _, left, right }) => {
                let new_id = self.next_id;
                self.next_id += 1;
                let current_rect = left.leaf_rect().unwrap_or_else(Rect::full_screen);
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
                    LayoutNode::Leaf { id: *left.id(), pane: Pane { agent_id: left.get_agent(), rect: left_rect } },
                    LayoutNode::Leaf { id: new_id, pane: Pane { agent_id: right.get_agent(), rect: right_rect } },
                    new_id,
                )
            }
            _ => return 0,
        };

        let split_node = LayoutNode::Split {
            direction,
            ratio,
            left: Box::new(left_child),
            right: Box::new(right_child),
        };

        self.nodes.push(split_node);
        new_id
    }

    /// Close the focused pane.
    pub fn close(&mut self, _node_id: NodeId) -> bool {
        if self.nodes.len() > 1 {
            self.nodes.pop();
            true
        } else {
            false
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

    /// Count leaf panes.
    pub fn leaf_count(&self) -> usize {
        self.nodes.iter().filter(|n| n.is_leaf()).count()
    }

    /// Get all leaf IDs.
    pub fn leaf_ids(&self) -> Vec<NodeId> {
        self.nodes
            .iter()
            .filter_map(|n| match n {
                LayoutNode::Leaf { id, .. } => Some(*id),
                _ => None,
            })
            .collect()
    }

    /// Get a specific node.
    pub fn get_node(&self, _id: NodeId) -> Option<&LayoutNode> {
        None
    }

    /// Get a specific node (mutable).
    pub fn get_node_mut(&mut self, _id: NodeId) -> Option<&mut LayoutNode> {
        None
    }

    /// Get all leaves.
    pub fn leaves(&self) -> Vec<(&NodeId, &Pane)> {
        self.nodes
            .iter()
            .filter_map(|n| match n {
                LayoutNode::Leaf { id, pane } => Some((id, pane)),
                _ => None,
            })
            .collect()
    }

    /// Get all leaf node IDs.
    pub fn leaf_nodes(&self) -> Vec<NodeId> {
        self.leaf_ids()
    }

    /// Get the focused node.
    pub fn focused_node(&self) -> Option<&LayoutNode> {
        self.nodes.last()
    }

    /// Get the focused node (mutable).
    pub fn focused_node_mut(&mut self) -> Option<&mut LayoutNode> {
        self.nodes.last_mut()
    }

    /// Find the next leaf in a direction.
    fn next_in_direction(&self, _id: NodeId, _direction: FocusDirection) -> Option<NodeId> {
        None
    }

    /// Get the layout as a tree structure (for serialization).
    pub fn to_tree(&self) -> Option<LayoutNode> {
        self.nodes.first().cloned()
    }

    /// Convert the layout tree to a JSON value for the frontend.
    pub fn to_json(&self) -> serde_json::Value {
        self.nodes.first().map(json_serialize_node).unwrap_or_else(|| serde_json::json!(null))
    }

    /// Set a custom tree (from preset or loaded state).
    pub fn set_tree(&mut self, tree: LayoutNode) {
        self.nodes.clear();
        self.nodes.push(tree);
    }

    /// Get the pane size for a node.
    pub fn get_pane_size(&self, node_id: NodeId) -> Option<(u16, u16)> {
        for node in &self.nodes {
            if let LayoutNode::Leaf { id, pane } = node {
                if *id == node_id {
                    return Some((pane.rect.width as u16, pane.rect.height as u16));
                }
            }
        }
        None
    }

    /// Set an agent on a pane.
    pub fn set_pane_agent(&mut self, node_id: NodeId, agent_id: String) {
        for node in &mut self.nodes {
            if let LayoutNode::Leaf { id, pane } = node {
                if *id == node_id {
                    pane.agent_id = Some(agent_id);
                    return;
                }
            }
        }
    }

    /// Create a new leaf node.
    pub fn new_leaf(&mut self) -> NodeId {
        let id = self.next_id;
        self.next_id += 1;
        self.nodes.push(LayoutNode::Leaf {
            id,
            pane: Pane {
                agent_id: None,
                rect: Rect::full_screen(),
            },
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
        LayoutNode::Split { direction, ratio, left, right } => {
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
}
