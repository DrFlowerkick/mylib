// tree node type definition, functions and tests

use std::cell::RefCell;
use std::cmp::Ordering;
use std::rc::Rc;
use std::rc::Weak;

use super::{
    BackTrack, IterChildren, IterSelf, LevelOrderTraversal, PostOrderTraversal, PreOrderTraversal,
};

pub struct TreeNode<N> {
    value: RefCell<N>,
    level: usize,
    node: RefCell<Weak<TreeNode<N>>>,
    parent: RefCell<Weak<TreeNode<N>>>,
    children: RefCell<Vec<Rc<TreeNode<N>>>>,
}

impl<N: PartialEq> TreeNode<N> {
    pub fn seed_root(value: N, children_capacity: usize) -> Rc<TreeNode<N>> {
        TreeNode::new(value, 0, children_capacity)
    }
    fn new(value: N, level: usize, children_capacity: usize) -> Rc<TreeNode<N>> {
        let result = Rc::new(TreeNode {
            value: RefCell::new(value),
            level,
            node: RefCell::new(Weak::new()), // weak reference on itself!
            parent: RefCell::new(Weak::new()),
            children: RefCell::new(Vec::with_capacity(children_capacity)),
        });
        let node = Rc::downgrade(&result);
        *result.node.borrow_mut() = node;
        result
    }
    pub fn add_child_to_parent(
        &self,
        child_value: N,
        parent_value: &N,
        children_capacity: usize,
    ) -> Option<Rc<TreeNode<N>>> {
        // search always from root to make sure that parent will be found
        self.get_root()
            .get_node(parent_value)
            .map(|parent| parent.add_child(child_value, children_capacity))
    }
    pub fn add_child(&self, value: N, children_capacity: usize) -> Rc<TreeNode<N>> {
        match self.iter_children().find(|n| *n.value.borrow() == value) {
            Some(node) => node,
            None => {
                let child = TreeNode::new(value, self.level + 1, children_capacity);
                *child.parent.borrow_mut() = self.node.borrow().clone();
                self.children.borrow_mut().push(child.clone());
                child
            }
        }
    }
    pub fn insert_child_at_parent(
        &self,
        child_value: N,
        parent_value: &N,
        index: usize,
        children_capacity: usize,
    ) -> Option<Rc<TreeNode<N>>> {
        // search always from root to make sure that parent will be found
        self.get_root()
            .get_node(parent_value)
            .map(|parent| parent.insert_child(child_value, index, children_capacity))
    }
    pub fn insert_child(
        &self,
        value: N,
        index: usize,
        children_capacity: usize,
    ) -> Rc<TreeNode<N>> {
        match self.iter_children().find(|n| *n.value.borrow() == value) {
            Some(node) => node,
            None => {
                let child = TreeNode::new(value, self.level + 1, children_capacity);
                *child.parent.borrow_mut() = self.node.borrow().clone();
                let number_of_children = self.children.borrow().len();
                if index < number_of_children {
                    self.children.borrow_mut().insert(index, child.clone());
                } else {
                    self.children.borrow_mut().push(child.clone());
                }
                child
            }
        }
    }
    pub fn add_unambiguous_child_to_parent(
        &self,
        child_value: N,
        parent_value: &N,
        children_capacity: usize,
    ) -> Option<Rc<TreeNode<N>>> {
        let root = self.get_root();
        if root.get_node(&child_value).is_some() {
            return None; // child already exists
        }
        root.get_node(parent_value)
            .map(|parent| parent.add_child(child_value, children_capacity))
    }
    pub fn add_unambiguous_child(
        &self,
        value: N,
        children_capacity: usize,
    ) -> Option<Rc<TreeNode<N>>> {
        // search always from root to make sure that all added children values are checked
        match self
            .get_root()
            .iter_pre_order_traversal()
            .find(|n| *n.value.borrow() == value)
        {
            Some(_) => None, // child already exists
            None => {
                let child = TreeNode::new(value, self.level + 1, children_capacity);
                *child.parent.borrow_mut() = self.node.borrow().clone();
                self.children.borrow_mut().push(child.clone());
                Some(child)
            }
        }
    }
    pub fn insert_unambiguous_child_at_parent(
        &self,
        child_value: N,
        parent_value: &N,
        index: usize,
        children_capacity: usize,
    ) -> Option<Rc<TreeNode<N>>> {
        let root = self.get_root();
        if root.get_node(&child_value).is_some() {
            return None; // child already exists
        }
        root.get_node(parent_value)
            .map(|parent| parent.insert_child(child_value, index, children_capacity))
    }
    pub fn insert_unambiguous_child(
        &self,
        value: N,
        index: usize,
        children_capacity: usize,
    ) -> Option<Rc<TreeNode<N>>> {
        match self
            .get_root()
            .iter_pre_order_traversal()
            .find(|n| *n.value.borrow() == value)
        {
            Some(_) => None, // child already exists,
            None => {
                let child = TreeNode::new(value, self.level + 1, children_capacity);
                *child.parent.borrow_mut() = self.node.borrow().clone();
                let number_of_children = self.children.borrow().len();
                if index < number_of_children {
                    self.children.borrow_mut().insert(index, child.clone());
                } else {
                    self.children.borrow_mut().push(child.clone());
                }
                Some(child)
            }
        }
    }
    pub fn swap_remove_child(&self, index: usize) -> Option<Rc<TreeNode<N>>> {
        if index >= self.len_children() {
            return None;
        }
        let result = self.children.borrow_mut().swap_remove(index);
        Some(result)
    }
    pub fn split_off_children(&self, at: usize, keep_split_off: bool) {
        let split_off = self.children.borrow_mut().split_off(at);
        if keep_split_off {
            *self.children.borrow_mut() = split_off;
        }
    }
    pub fn reserve_children(&self, additional_children: usize) {
        // increases capacity of children by additional_children
        self.children.borrow_mut().reserve(additional_children);
    }
    pub fn clear_children(&self, children_capacity: usize) {
        *self.children.borrow_mut() = Vec::with_capacity(children_capacity);
    }
    pub fn clear_parent(&self) {
        // removing parent makes this node to a root node. If no reference or variable exists, which holds at least
        // one node above this node, then all nodes above this node are released from memory
        *self.parent.borrow_mut() = Weak::new();
    }
    pub fn get_value(&self) -> std::cell::Ref<'_, N> {
        self.value.borrow()
    }
    pub fn get_mut_value(&self) -> std::cell::RefMut<'_, N> {
        self.value.borrow_mut()
    }
    pub fn get_level(&self) -> usize {
        self.level
    }
    pub fn get_self(&self) -> Option<Rc<TreeNode<N>>> {
        self.node.borrow().upgrade().as_ref().cloned()
    }
    pub fn get_child(&self, index: usize) -> Option<Rc<TreeNode<N>>> {
        self.children.borrow().get(index).cloned()
    }
    pub fn len_children(&self) -> usize {
        self.children.borrow().len()
    }
    pub fn get_parent(&self) -> Option<Rc<TreeNode<N>>> {
        self.parent.borrow().upgrade().as_ref().cloned()
    }
    pub fn get_node(&self, value: &N) -> Option<Rc<TreeNode<N>>> {
        self.iter_pre_order_traversal()
            .find(|n| *n.value.borrow() == *value)
    }
    pub fn get_root(&self) -> Rc<TreeNode<N>> {
        let mut node = self.get_self().unwrap();
        loop {
            match node.get_parent() {
                Some(parent) => node = parent.clone(),
                None => return node,
            }
        }
    }
    pub fn is_root(&self) -> bool {
        self.get_self().unwrap().get_parent().is_none()
    }
    pub fn is_leave(&self) -> bool {
        self.len_children() == 0
    }
    pub fn sort_children_by<F>(&self, compare: F)
    where
        F: Fn(&N, &N) -> Ordering,
    {
        self.children
            .borrow_mut()
            .sort_by(|a, b| compare(&a.value.borrow(), &b.value.borrow()));
    }
    pub fn get_max_level(&self) -> (usize, usize) {
        // tuple of absolute level and relative level
        self.get_root()
            .iter_level_order_traversal()
            .max_by_key(|(_, l)| *l)
            .map(|(n, l)| (n.level, l))
            .unwrap()
    }
    pub fn iter_self(&self) -> impl Iterator<Item = Rc<TreeNode<N>>> {
        IterSelf::new(self.get_self().unwrap()) // iterator over single node; useful for functions, which have an iterator as output and you want to be able to iterate over different outcomes
    }
    pub fn iter_children(&self) -> impl Iterator<Item = Rc<TreeNode<N>>> {
        IterChildren::new(self.get_self().unwrap())
    }
    pub fn iter_back_track(&self) -> impl Iterator<Item = Rc<TreeNode<N>>> {
        BackTrack::new(self.get_self().unwrap())
    }
    pub fn iter_pre_order_traversal(&self) -> impl Iterator<Item = Rc<TreeNode<N>>> {
        PreOrderTraversal::new(self.get_self().unwrap())
    }
    pub fn iter_post_order_traversal(&self) -> impl Iterator<Item = Rc<TreeNode<N>>> {
        PostOrderTraversal::new(self.get_self().unwrap())
    }
    // second return value is level of node relative to start node, from which iter_level_order_traversal() was called
    pub fn iter_level_order_traversal(&self) -> impl Iterator<Item = (Rc<TreeNode<N>>, usize)> {
        LevelOrderTraversal::new(self.get_self().unwrap(), 0, None)
    }
    pub fn iter_level_order_traversal_with_borders(
        &self,
        start_level: usize,
        end_level: Option<usize>,
    ) -> impl Iterator<Item = (Rc<TreeNode<N>>, usize)> {
        LevelOrderTraversal::new(self.get_self().unwrap(), start_level, end_level)
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn test_my_tree() {
        // Build test tree
        // tree structure is inspired from Wikipedia: https://en.wikipedia.org/wiki/Tree_traversal#Depth-first_search
        let test_tree = TreeNode::seed_root('F', 2);
        test_tree.add_child('G', 1);
        test_tree.insert_child('B', 0, 2);
        test_tree.add_child_to_parent('D', &'B', 2);
        test_tree.insert_child_at_parent('A', &'B', 0, 0);
        test_tree.add_child_to_parent('C', &'D', 0);
        test_tree.add_child_to_parent('E', &'D', 0);
        test_tree.add_child_to_parent('I', &'G', 1);
        test_tree.add_child_to_parent('H', &'I', 0);

        assert_eq!(*test_tree.get_child(0).unwrap().get_value(), 'B');

        let child_h = test_tree.get_node(&'H').unwrap();
        assert_eq!(*child_h.get_value(), 'H');
        assert_eq!(*child_h.get_root().get_value(), 'F');

        let mut backtrack_iterator = child_h.iter_back_track();
        assert_eq!(*backtrack_iterator.next().unwrap().get_value(), 'H');
        assert_eq!(*backtrack_iterator.next().unwrap().get_value(), 'I');
        assert_eq!(*backtrack_iterator.next().unwrap().get_value(), 'G');
        assert_eq!(*backtrack_iterator.next().unwrap().get_value(), 'F');
        assert!(backtrack_iterator.next().is_none());

        assert_eq!(test_tree.add_unambiguous_child('I', 0).is_none(), true);
        assert_eq!(child_h.insert_unambiguous_child('F', 0, 0).is_none(), true);
        assert_eq!(
            test_tree
                .add_unambiguous_child_to_parent('I', &'Z', 0)
                .is_none(),
            true
        );
        assert_eq!(
            test_tree
                .add_unambiguous_child_to_parent('I', &'B', 0)
                .is_none(),
            true
        );
        assert_eq!(
            child_h
                .insert_unambiguous_child_at_parent('F', &'Z', 0, 0)
                .is_none(),
            true
        );
        assert_eq!(
            child_h
                .insert_unambiguous_child_at_parent('F', &'A', 0, 0)
                .is_none(),
            true
        );

        assert!(child_h.is_leave());
        let child_d = test_tree.get_node(&'D').unwrap();
        assert!(!child_d.is_leave());
        assert!(!child_d.is_root());
        let child_f = test_tree.get_node(&'F').unwrap();
        assert!(child_f.is_root());
        assert_eq!(
            test_tree
                .iter_pre_order_traversal()
                .filter(|n| n.is_leave())
                .count(),
            4
        );
        assert_eq!(
            test_tree
                .iter_post_order_traversal()
                .filter(|n| n.is_leave())
                .count(),
            4
        );
        assert_eq!(
            test_tree
                .iter_level_order_traversal()
                .filter(|(n, _)| n.is_leave())
                .count(),
            4
        );

        let pre_order_iterator = test_tree.iter_pre_order_traversal();
        let pre_order_vector: Vec<char> = pre_order_iterator.map(|n| *n.get_value()).collect();
        assert_eq!(
            pre_order_vector,
            ['F', 'B', 'A', 'D', 'C', 'E', 'G', 'I', 'H']
        );
        let child_b = test_tree.get_node(&'B').unwrap();
        let pre_order_iterator = child_b.iter_pre_order_traversal();
        let pre_order_vector: Vec<char> = pre_order_iterator.map(|n| *n.get_value()).collect();
        assert_eq!(pre_order_vector, ['B', 'A', 'D', 'C', 'E']);

        let post_order_iterator = test_tree.iter_post_order_traversal();
        let post_order_vector: Vec<char> = post_order_iterator.map(|n| *n.get_value()).collect();
        assert_eq!(
            post_order_vector,
            ['A', 'C', 'E', 'D', 'B', 'H', 'I', 'G', 'F']
        );
        let child_b = test_tree.get_node(&'B').unwrap();
        let post_order_iterator = child_b.iter_post_order_traversal();
        let post_order_vector: Vec<char> = post_order_iterator.map(|n| *n.get_value()).collect();
        assert_eq!(post_order_vector, ['A', 'C', 'E', 'D', 'B']);

        let level_order_iterator = test_tree.iter_level_order_traversal();
        let level_order_vector: Vec<char> =
            level_order_iterator.map(|(n, _)| *n.get_value()).collect();
        assert_eq!(
            level_order_vector,
            ['F', 'B', 'G', 'A', 'D', 'I', 'C', 'E', 'H']
        );
        let child_b = test_tree.get_node(&'B').unwrap();
        let level_order_iterator = child_b.iter_level_order_traversal();
        let level_order_vector: Vec<char> =
            level_order_iterator.map(|(n, _)| *n.get_value()).collect();
        assert_eq!(level_order_vector, ['B', 'A', 'D', 'C', 'E']);
        let level_order_iterator = test_tree.iter_level_order_traversal_with_borders(2, None);
        let level_order_vector: Vec<char> =
            level_order_iterator.map(|(n, _)| *n.get_value()).collect();
        assert_eq!(level_order_vector, ['A', 'D', 'I', 'C', 'E', 'H']);
        let level_order_iterator = test_tree.iter_level_order_traversal_with_borders(1, Some(2));
        let level_order_vector: Vec<char> =
            level_order_iterator.map(|(n, _)| *n.get_value()).collect();
        assert_eq!(level_order_vector, ['B', 'G', 'A', 'D', 'I']);

        let child_b = test_tree.get_child(0).unwrap();
        assert_eq!(*child_b.get_child(0).unwrap().get_value(), 'A');
        child_b.sort_children_by(|a, b| b.cmp(&a));
        assert_eq!(*child_b.get_child(0).unwrap().get_value(), 'D');

        // changing value of node
        {
            let mut child_b_value = child_b.get_mut_value();
            *child_b_value = 'X';
        }
        assert_eq!(*child_b.get_value(), 'X');
    }
}
