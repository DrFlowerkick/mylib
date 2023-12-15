use std::rc::Rc;

use crate::my_binary_tree::*;


pub trait SteadySolutionItem: Copy + Clone + Eq + Ord + PartialEq + PartialOrd {
    // resulting output must fulfill either self > Output > item or self < Output < item
    fn combine(&self, item: Self) -> Self;
}
pub struct SteadySolutionTree<S: SteadySolutionItem> {
    pub tree: Rc<BinaryTreeNode<S>>,
    min: S,
    max: S,
}

impl<S: SteadySolutionItem> SteadySolutionTree<S> {
    pub fn new(min: S, max: S) -> SteadySolutionTree<S> {
        assert!(min < max);
        let tree_mid = min.combine(max);
        assert!(min < tree_mid);
        assert!(tree_mid < max);
        SteadySolutionTree {
            tree: BinaryTreeNode::new(tree_mid),
            min,
            max,
        }
    }
    pub fn append_next_level(&self) {
        let max_level = self.tree.get_max_level();
        let max_level_nodes: Vec<Rc<BinaryTreeNode<S>>> = self.tree.iter_level_order_traversal().filter(|(_, l)| *l == max_level).map(|(n, _)| n).collect();
        for node in max_level_nodes {
            let left = match node.get_next_smaller() {
                Some(node) => node.get_value(),
                None => self.min,
            };
            node.append_value(node.get_value().combine(left));
            let right = match node.get_next_bigger() {
                Some(node) => node.get_value(),
                None => self.max,
            };
            node.append_value(node.get_value().combine(right));
        }
    }
    pub fn build_tree_to_level(&self, level: usize) {
        for (node, current_level) in self.tree.iter_level_order_traversal() {
            if current_level == level {
                break;
            }
            let left = match node.get_next_smaller() {
                Some(node) => node.get_value(),
                None => self.min,
            };
            node.append_value(node.get_value().combine(left));
            let right = match node.get_next_bigger() {
                Some(node) => node.get_value(),
                None => self.max,
            };
            node.append_value(node.get_value().combine(right));
        }
    }
    pub fn add_steady_solution_item(&self, rn: S) -> Rc<BinaryTreeNode<S>> {
        let mut current_node = self.tree.get_self().unwrap();
        while current_node.get_value() != rn {
            if rn < current_node.get_value() {
                current_node = match current_node.get_left() {
                    Some(node) => node,
                    None => {
                        let left = match current_node.get_next_smaller() {
                            Some(node) => node.get_value(),
                            None => self.min,
                        };
                        current_node.append_value(current_node.get_value().combine(left))
                    },
                };
            } else {
                current_node = match current_node.get_right() {
                    Some(node) => node,
                    None => {
                        let right = match current_node.get_next_bigger() {
                            Some(node) => node.get_value(),
                            None => self.max,
                        };
                        current_node.append_value(current_node.get_value().combine(right))
                    },
                };
            }
        }
        current_node
    }
    pub fn add_lr_path(&self, lr_path: String) -> Rc<BinaryTreeNode<S>> {
        let mut current_node = self.tree.get_self().unwrap();
        for direction in lr_path.chars() {
            match direction {
                'L' => {
                    current_node = match current_node.get_left() {
                        Some(node) => node,
                        None => {
                            let left = match current_node.get_next_smaller() {
                                Some(node) => node.get_value(),
                                None => self.min,
                            };
                            current_node.append_value(current_node.get_value().combine(left))
                        },
                    };
                },
                'R' => {
                    current_node = match current_node.get_right() {
                        Some(node) => node,
                        None => {
                            let right = match current_node.get_next_bigger() {
                                Some(node) => node.get_value(),
                                None => self.max,
                            };
                            current_node.append_value(current_node.get_value().combine(right))
                        },
                    };
                },
                _ => panic!("Unknown char in L-R path.")
            }
        }
        current_node
    }
}