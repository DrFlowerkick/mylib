use std::cell::RefCell;
use std::rc::Rc;

pub struct SimpleBinaryTreeNode<N> {
    value: N,
    left: RefCell<Option<Rc<SimpleBinaryTreeNode<N>>>>,
    right: RefCell<Option<Rc<SimpleBinaryTreeNode<N>>>>,
}

impl<N: Eq + PartialEq + Copy + Clone> SimpleBinaryTreeNode<N> {
    pub fn new(value: N) -> Rc<SimpleBinaryTreeNode<N>> {
        Rc::new(SimpleBinaryTreeNode {
            value,
            left: RefCell::new(None),
            right: RefCell::new(None),
        })
    }
    pub fn append_left(&self, value: N) -> Rc<SimpleBinaryTreeNode<N>> {
        if let Some(left) = self.get_left() {
            return left;
        }
        let left = SimpleBinaryTreeNode::new(value);
        *self.left.borrow_mut() = Some(left);
        self.get_left().unwrap()
    }
    pub fn link_left(
        &self,
        node: Rc<SimpleBinaryTreeNode<N>>,
    ) -> Option<Rc<SimpleBinaryTreeNode<N>>> {
        if self.get_left().is_some() {
            // left is already linked
            return None;
        }
        *self.left.borrow_mut() = Some(node);
        Some(self.get_left().unwrap())
    }
    pub fn link_right(
        &self,
        node: Rc<SimpleBinaryTreeNode<N>>,
    ) -> Option<Rc<SimpleBinaryTreeNode<N>>> {
        if self.get_right().is_some() {
            // right is already linked
            return None;
        }
        *self.right.borrow_mut() = Some(node);
        Some(self.get_right().unwrap())
    }
    pub fn append_right(&self, value: N) -> Rc<SimpleBinaryTreeNode<N>> {
        if let Some(right) = self.get_right() {
            return right;
        }
        let right = SimpleBinaryTreeNode::new(value);
        *self.right.borrow_mut() = Some(right);
        self.get_right().unwrap()
    }
    pub fn get_value(&self) -> N {
        self.value
    }
    pub fn get_left(&self) -> Option<Rc<SimpleBinaryTreeNode<N>>> {
        self.left.borrow().as_ref().cloned()
    }
    pub fn get_right(&self) -> Option<Rc<SimpleBinaryTreeNode<N>>> {
        self.right.borrow().as_ref().cloned()
    }
}
