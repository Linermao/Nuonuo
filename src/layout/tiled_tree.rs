use slotmap::{new_key_type, Key, SlotMap};
use smithay::{desktop::Window, utils::{Logical, Rectangle}};

use crate::space::window::WindowExt;


const RATE: f32 = 2.0;
const GAP: i32 = 12;

#[derive(Debug, Clone)]
pub enum LayoutScheme {
    Default,
    BinaryTree,
}

#[derive(Debug, Clone, Copy)]
pub enum Direction {
    Horizontal,
    Vertical,
}

new_key_type! {
    pub struct NodeId;
}

#[derive(Debug, Clone)]
pub enum NodeData {
    Leaf { window: Window },
    Split {
        direction: Direction,
        rec: Rectangle<i32, Logical>,
        left: NodeId,
        right: NodeId,
    }
}

#[derive(Debug)]
pub struct TiledTree {
    nodes: SlotMap<NodeId, NodeData>,
    root: Option<NodeId>,
}

impl TiledTree {
    pub fn new(window: Window) -> Self {
        let mut nodes = SlotMap::with_key();
        let root = Some(nodes.insert(NodeData::Leaf { window }));
        Self { 
            nodes,
            root
       }
    }

    pub fn is_empty(&self) -> bool {
        self.root.is_none()
    }

    pub fn find_node(&self, window: &Window) -> Option<NodeId> {
        self.nodes.iter()
            .find_map(|(id, data)| match data {
                NodeData::Leaf { window: w } if w == window => Some(id),
                _ => None,
            })
    }

    pub fn insert(&mut self, target: &Window, new_window: Window) -> bool {
        if let Some(target_id) = self.find_node(target) {
            // resize
            let rec = target.get_rec().unwrap();
            let (direction, l_rec, r_rec) = get_new_rec(&rec);
            target.set_rec(l_rec);
            new_window.set_rec(r_rec);

            // adjust tree
            let original = self.nodes[target_id].clone();
            let new_leaf = self.nodes.insert(NodeData::Leaf { window: new_window });
            let old_leaf = match original {
                NodeData::Leaf { window } => self.nodes.insert(NodeData::Leaf { window }),
                _ => return false,
            };

            self.nodes[target_id] = NodeData::Split {
                direction,
                rec,
                left: old_leaf,
                right: new_leaf,
            };
            true
        } else {
            false
        }
    }

    pub fn remove(&mut self, target: &Window) -> bool {
        let target_id = self.find_node(target).unwrap();

        // remove last node
        if let Some(root_id) = self.root {
            if target_id == root_id {
                if let NodeData::Leaf { .. } = self.nodes[target_id] {
                    self.nodes.remove(target_id);
                    self.root = None;
                    return true;
                }
            }
        }

        let (parent_id, sibling_id) = self.find_parent_and_sibling(target_id).unwrap();

        match self.nodes[parent_id] {
            NodeData::Split { rec, .. } => {
                let sibling_data = self.nodes.remove(sibling_id).unwrap();

                match sibling_data {
                    NodeData::Leaf { window } => {
                        window.set_rec(rec.clone());
                        self.nodes[parent_id] = NodeData::Leaf { window };
                    },
                    NodeData::Split { direction, left, right, .. } => {
                        self.nodes[parent_id] = NodeData::Split { 
                            direction, 
                            rec, // from parent
                            left, 
                            right,
                        };
                        self.modify(parent_id, &rec);
                    }
                }

                self.nodes.remove(target_id);

                true
            },
            NodeData::Leaf { .. } => { 
                false 
            }
        }
    }

    fn modify(&mut self, node_id: NodeId, rec: &Rectangle<i32, Logical>) {
        match &mut self.nodes[node_id] {
            NodeData::Leaf { window } => {
                window.set_rec(*rec);
            },
            NodeData::Split { left, right, direction, rec: current_rec } => {
                let (l_rec, r_rec) = recover_new_rec(rec, direction);
                
                *current_rec = rec.clone();

                let left_id = *left;
                let right_id = *right;

                self.modify(left_id, &l_rec);
                self.modify(right_id, &r_rec);
            }
        }
    }

    fn find_parent_and_sibling(&self, target: NodeId) -> Option<(NodeId, NodeId)> {
        self.nodes.iter().find_map(|(id, data)| match data {
            NodeData::Split { left, right, .. } => {
                if *left == target {
                    Some((id, *right))
                } else if *right == target {
                    Some((id, *left))
                } else {
                    None
                }
            }
            _ => None,
        })
    }

    #[cfg(feature="trace_layout")]
    pub fn print_tree(&self) {
        fn print(nodes: &SlotMap<NodeId, NodeData>, id: NodeId, depth: usize) {
            let indent = "  ".repeat(depth);
            match &nodes[id] {
                NodeData::Leaf { window } => tracing::info!("{indent}- Leaf: {:?}", window.get_id()),
                NodeData::Split { left, right, .. } => {
                    tracing::info!("{indent}- Split:");
                    print(nodes, *left, depth + 1);
                    print(nodes, *right, depth + 1);
                }
            }
        }

        print(&self.nodes, self.root.unwrap(), 0);
    }
}

fn get_new_rec(rec: &Rectangle<i32, Logical>) -> (Direction, Rectangle<i32, Logical>, Rectangle<i32, Logical>) {

    let mut l_rec = *rec;
    let mut r_rec = *rec;

    let gap = (GAP as f32 * 0.5) as i32;
    
    if rec.size.h as f32 / rec.size.w as f32 > RATE {
        let half = rec.size.h / 2 - gap;
        l_rec.size.h = half;
        r_rec.size.h = half;
        r_rec.loc.y += half + GAP;
        (Direction::Vertical, l_rec, r_rec)
    } else {
        let half = rec.size.w / 2 - gap;
        l_rec.size.w = half;
        r_rec.size.w = half;
        r_rec.loc.x += half + GAP;
        (Direction::Horizontal, l_rec, r_rec)
    }
}

fn recover_new_rec(rec: &Rectangle<i32, Logical>, direction: &Direction) -> (Rectangle<i32, Logical>, Rectangle<i32, Logical>) {
    let mut l_rec = *rec;
    let mut r_rec = *rec;

    let gap = (GAP as f32 * 0.5) as i32;

    match direction {
        Direction::Horizontal => {
            let half = rec.size.w / 2 - gap;
            l_rec.size.w = half;
            r_rec.size.w = half;
            r_rec.loc.x += half + GAP;
            (l_rec, r_rec)
        },
        Direction::Vertical => {
            let half = rec.size.h / 2 - gap;
            l_rec.size.h = half;
            r_rec.size.h = half;
            r_rec.loc.y += half + GAP;
            (l_rec, r_rec)
        }
    }
}
