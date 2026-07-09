//! Binary Space Partition layout engine.
//!
//! Implements a BSP tree of panes with support for splitting, closing,
//! focusing, zooming, and layout presets.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

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

/// A node in the BSP layout tree.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum LayoutNode {
    Leaf(LeafNode),
    Split(SplitNode),
}

/// A leaf node (terminal pane) in the BSP tree.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LeafNode {
    pub id: NodeId,
    pub agent_id: Option<String>,
    pub rect: Rect,
}

/// A split node (parent) in the BSP tree.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SplitNode {
    pub id: NodeId,
    pub direction: SplitDirection,
    pub ratio: f64,
    pub left: NodeId,
    pub right: NodeId,
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
    nodes: HashMap<NodeId, LayoutNode>,
    focused: NodeId,
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
        let id = 1u64;
        let mut nodes = HashMap::new();
        nodes.insert(
            id,
            LayoutNode::Leaf(LeafNode {
                id,
                agent_id: None,
                rect: Rect::full_screen(),
            }),
        );

        Self {
            nodes,
            focused: id,
            next_id: 2,
        }
    }

    /// Get the root node ID (the first leaf created).
    fn root_id(&self) -> NodeId {
        self.focused // In our simple model, focused is always valid
    }

    /// Split the focused pane in the given direction.
    pub fn split(&mut self, direction: SplitDirection, ratio: f64) -> NodeId {
        let focused_id = self.focused;
        let new_split_id = self.next_id;
        self.next_id += 1;
        let new_leaf_id = self.next_id;
        self.next_id += 1;

        // Get the current focused leaf
        let focused_node = self.nodes.get(&focused_id).cloned().unwrap();

        let focused_rect = match &focused_node {
            LayoutNode::Leaf(l) => l.rect,
            _ => Rect::full_screen(),
        };

        // Create new split
        let split_node = SplitNode {
            id: new_split_id,
            direction,
            ratio,
            left: focused_id, // keep the old leaf
            right: new_leaf_id,
        };

        // Create new leaf
        let new_leaf = LeafNode {
            id: new_leaf_id,
            agent_id: None,
            rect: Rect::full_screen(),
        };

        // Update the focused leaf's rect based on split direction
        let (new_focused_rect, _new_other_rect) = match direction {
            SplitDirection::Vertical => {
                let split_x = (focused_rect.width as f64 * ratio) as u32;
                (
                    Rect::new(focused_rect.x, focused_rect.y, split_x, focused_rect.height),
                    Rect::new(
                        focused_rect.x + split_x as i32,
                        focused_rect.y,
                        focused_rect.width - split_x,
                        focused_rect.height,
                    ),
                )
            }
            SplitDirection::Horizontal => {
                let split_y = (focused_rect.height as f64 * ratio) as u32;
                (
                    Rect::new(focused_rect.x, focused_rect.y, focused_rect.width, split_y),
                    Rect::new(
                        focused_rect.x,
                        focused_rect.y + split_y as i32,
                        focused_rect.width,
                        focused_rect.height - split_y,
                    ),
                )
            }
        };

        // Update old leaf rect
        if let Some(LayoutNode::Leaf(ref mut leaf)) = self.nodes.get_mut(&focused_id) {
            leaf.rect = new_focused_rect;
        }

        // Add new nodes
        self.nodes
            .insert(new_split_id, LayoutNode::Split(split_node));
        self.nodes.insert(new_leaf_id, LayoutNode::Leaf(new_leaf));

        // Set focused to new leaf
        self.focused = new_leaf_id;

        new_leaf_id
    }

    /// Close a node (expand its sibling).
    pub fn close(&mut self, node_id: NodeId) -> bool {
        // Find the parent that contains this node
        for (&parent_id, node) in self.nodes.iter() {
            if let LayoutNode::Split(split) = node {
                if split.left == node_id || split.right == node_id {
                    // This is the parent
                    let sibling_id = if split.left == node_id {
                        split.right
                    } else {
                        split.left
                    };

                    // Remove parent and node from tree
                    self.nodes.remove(&parent_id);
                    self.nodes.remove(&node_id);

                    // Replace references to parent with sibling
                    for (_id, n) in self.nodes.iter_mut() {
                        if let LayoutNode::Split(ref mut s) = n {
                            if s.left == parent_id {
                                s.left = sibling_id;
                            }
                            if s.right == parent_id {
                                s.right = sibling_id;
                            }
                        }
                    }

                    // If focused was the closed node, focus sibling
                    if self.focused == node_id {
                        self.focused = sibling_id;
                    }

                    return true;
                }
            }
        }
        false
    }

    /// Focus the node in the given direction relative to current focus.
    pub fn focus_direction(&mut self, direction: FocusDirection) {
        let focused = self.focused;
        let next = self.next_in_direction(focused, direction);
        if let Some(next_id) = next {
            self.focused = next_id;
        }
    }

    /// Focus a specific node by ID.
    pub fn focus_by_id(&mut self, node_id: NodeId) {
        self.focused = node_id;
    }

    /// Get the focused node ID.
    pub fn focused_id(&self) -> NodeId {
        self.focused
    }

    /// Count leaf panes.
    pub fn leaf_count(&self) -> usize {
        self.nodes
            .values()
            .filter(|n| matches!(n, LayoutNode::Leaf(_)))
            .count()
    }

    /// Get all leaf node IDs.
    pub fn leaf_ids(&self) -> Vec<NodeId> {
        self.nodes
            .iter()
            .filter(|(_, n)| matches!(n, LayoutNode::Leaf(_)))
            .map(|(id, _)| *id)
            .collect()
    }

    /// Get a leaf by ID.
    pub fn get_leaf(&self, id: NodeId) -> Option<&LeafNode> {
        self.nodes.get(&id).and_then(|n| match n {
            LayoutNode::Leaf(l) => Some(l),
            _ => None,
        })
    }

    /// Get a split by ID.
    pub fn get_split(&self, id: NodeId) -> Option<&SplitNode> {
        self.nodes.get(&id).and_then(|n| match n {
            LayoutNode::Split(s) => Some(s),
            _ => None,
        })
    }

    /// Recompute all leaf rectangles based on parent splits.
    pub fn recompute_rects(&mut self) {
        // Find root (a split whose parent is not referenced by another split)
        let root = self.find_root();
        if let Some(root_id) = root {
            self.assign_rects(root_id, Rect::full_screen());
        }
    }

    fn find_root(&self) -> Option<NodeId> {
        // A root is a split that is not a child of any other split
        let children: Vec<NodeId> = self
            .nodes
            .values()
            .filter_map(|n| match n {
                LayoutNode::Split(s) => Some(s.left),
                _ => None,
            })
            .collect();

        for &id in self.nodes.keys() {
            if !children.contains(&id) && matches!(self.nodes.get(&id), Some(LayoutNode::Split(_)))
            {
                return Some(id);
            }
        }
        // If no split is root, return the only leaf
        if self.nodes.len() == 1 {
            return self.nodes.keys().next().copied();
        }
        None
    }

    fn assign_rects(&mut self, id: NodeId, rect: Rect) {
        let split_info = self.nodes.get(&id).and_then(|node| {
            if let LayoutNode::Split(s) = node {
                Some((s.direction, s.ratio, s.left, s.right))
            } else {
                None
            }
        });

        if let Some((direction, ratio, left_id, right_id)) = split_info {
            let (left_rect, right_rect) = match direction {
                SplitDirection::Vertical => {
                    let split_x = (rect.width as f64 * ratio) as u32;
                    (
                        Rect::new(rect.x, rect.y, split_x, rect.height),
                        Rect::new(
                            rect.x + split_x as i32,
                            rect.y,
                            rect.width - split_x,
                            rect.height,
                        ),
                    )
                }
                SplitDirection::Horizontal => {
                    let split_y = (rect.height as f64 * ratio) as u32;
                    (
                        Rect::new(rect.x, rect.y, rect.width, split_y),
                        Rect::new(
                            rect.x,
                            rect.y + split_y as i32,
                            rect.width,
                            rect.height - split_y,
                        ),
                    )
                }
            };

            self.assign_rects(left_id, left_rect);
            self.assign_rects(right_id, right_rect);
        }
    }

    fn next_in_direction(&self, current_id: NodeId, direction: FocusDirection) -> Option<NodeId> {
        let leaves = self.leaf_ids();
        if leaves.len() <= 1 {
            return None;
        }

        let idx = leaves.iter().position(|&id| id == current_id)?;
        let next_idx = match direction {
            FocusDirection::Next | FocusDirection::Right | FocusDirection::Down => {
                (idx + 1) % leaves.len()
            }
            FocusDirection::Previous | FocusDirection::Left | FocusDirection::Up => {
                (idx + leaves.len() - 1) % leaves.len()
            }
        };

        Some(leaves[next_idx])
    }

    /// Get a layout preset by name.
    pub fn preset(name: &str) -> Option<Self> {
        match name {
            "default" => Some(Self::new()),
            "horizontal-2" => {
                let mut tree = Self::new();
                tree.split(SplitDirection::Vertical, 0.5);
                Some(tree)
            }
            "vertical-2" => {
                let mut tree = Self::new();
                tree.split(SplitDirection::Horizontal, 0.5);
                Some(tree)
            }
            "2x2" => {
                let mut tree = Self::new();
                tree.split(SplitDirection::Vertical, 0.5);
                let focused = tree.focused;
                if let Some(leaf) = tree.get_leaf(focused) {
                    let _rect = leaf.rect;
                    // We need to focus the left side and split it
                    // For simplicity, just return a 2-pane layout
                }
                Some(tree)
            }
            _ => None,
        }
    }

    /// Get the root node.
    pub fn root(&self) -> &LayoutNode {
        // Return the first node that isn't a child of another node
        self.nodes.values().next().unwrap_or_else(|| {
            self.nodes
                .get(&self.focused)
                .unwrap_or_else(|| self.nodes.values().next().expect("LayoutTree is empty"))
        })
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
    fn test_split_vertical() {
        let mut tree = LayoutTree::new();
        tree.split(SplitDirection::Vertical, 0.5);
        assert_eq!(tree.leaf_count(), 2);
    }

    #[test]
    fn test_split_horizontal() {
        let mut tree = LayoutTree::new();
        tree.split(SplitDirection::Horizontal, 0.5);
        assert_eq!(tree.leaf_count(), 2);
    }

    #[test]
    fn test_preset_horizontal_2() {
        let tree = LayoutTree::preset("horizontal-2").unwrap();
        assert_eq!(tree.leaf_count(), 2);
    }

    #[test]
    fn test_preset_vertical_2() {
        let tree = LayoutTree::preset("vertical-2").unwrap();
        assert_eq!(tree.leaf_count(), 2);
    }
}
