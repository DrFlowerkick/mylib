use std::cell::RefCell;
use std::cmp::Ordering;
use std::rc::Rc;
use std::rc::Weak;

struct PreOrderTraversal<N> {
    next_node: Rc<BinaryTreeNode<N>>,
    horizontal: bool, // false: left, true: right
    vertical: bool,   // false: children, true: parent
}

impl<N: Ord + Eq + PartialOrd + PartialEq + Copy + Clone> PreOrderTraversal<N> {
    fn new(root: Rc<BinaryTreeNode<N>>) -> Self {
        PreOrderTraversal {
            next_node: root,
            horizontal: false,
            vertical: false,
        }
    }
}

impl<N: Ord + Eq + PartialOrd + PartialEq + Copy + Clone> Iterator for PreOrderTraversal<N> {
    type Item = Rc<BinaryTreeNode<N>>;

    fn next(&mut self) -> Option<Self::Item> {
        let mut result: Option<Self::Item> = None;
        loop {
            match (self.horizontal, self.vertical) {
                (false, false) => {
                    if result.is_none() {
                        result = self.next_node.get_self();
                    }
                    match self.next_node.get_left() {
                        Some(node) => {
                            self.next_node = node;
                            break;
                        }
                        None => self.horizontal = true,
                    }
                }
                (true, false) => {
                    if result.is_none() {
                        result = self.next_node.get_self();
                    }
                    match self.next_node.get_right() {
                        Some(node) => {
                            self.next_node = node;
                            self.horizontal = false;
                            break;
                        }
                        None => {
                            self.vertical = true;
                            match self.next_node.get_parent() {
                                Some(node) => {
                                    self.horizontal = self.next_node.get_direction().unwrap();
                                    self.next_node = node;
                                }
                                None => {
                                    self.horizontal = true; // this only happens if right child of root is None
                                    break;
                                }
                            }
                        }
                    }
                }
                (false, true) => {
                    self.vertical = false;
                    self.horizontal = true;
                }
                (true, true) => {
                    match self.next_node.get_parent() {
                        Some(node) => {
                            self.horizontal = self.next_node.get_direction().unwrap();
                            self.next_node = node;
                        }
                        None => break, // end of tree
                    }
                }
            }
        }
        result
    }
}

struct InOrderTraversal<N> {
    current_node: Rc<BinaryTreeNode<N>>,
    horizontal: bool, // false: left, true: right
    vertical: bool,   // false: children, true: parent
}

impl<N: Ord + Eq + PartialOrd + PartialEq + Copy + Clone> InOrderTraversal<N> {
    fn new(root: Rc<BinaryTreeNode<N>>) -> Self {
        InOrderTraversal {
            current_node: root,
            horizontal: false,
            vertical: false,
        }
    }
}

impl<N: Ord + Eq + PartialOrd + PartialEq + Copy + Clone> Iterator for InOrderTraversal<N> {
    type Item = Rc<BinaryTreeNode<N>>;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            match (self.horizontal, self.vertical) {
                (false, false) => match self.current_node.get_left() {
                    Some(node) => self.current_node = node,
                    None => {
                        self.horizontal = true;
                        return self.current_node.get_self();
                    }
                },
                (true, false) => {
                    match self.current_node.get_right() {
                        Some(node) => {
                            self.current_node = node;
                            self.horizontal = false;
                        }
                        None => {
                            self.vertical = true;
                            match self.current_node.get_parent() {
                                Some(node) => {
                                    self.horizontal = self.current_node.get_direction().unwrap();
                                    self.current_node = node;
                                }
                                None => {
                                    self.horizontal = true; // this only happens if right child of root is None
                                    return None;
                                }
                            }
                        }
                    }
                }
                (false, true) => {
                    self.vertical = false;
                    self.horizontal = true;
                    return self.current_node.get_self();
                }
                (true, true) => {
                    match self.current_node.get_parent() {
                        Some(node) => {
                            self.horizontal = self.current_node.get_direction().unwrap();
                            self.current_node = node;
                        }
                        None => return None, // end of tree
                    }
                }
            }
        }
    }
}

struct PostOrderTraversal<N> {
    current_node: Rc<BinaryTreeNode<N>>,
    horizontal: bool, // false: left, true: right
    vertical: bool,   // false: children, true: parent
    finished: bool,   // true if iterator finished
}

impl<N: Ord + Eq + PartialOrd + PartialEq + Copy + Clone> PostOrderTraversal<N> {
    fn new(root: Rc<BinaryTreeNode<N>>) -> Self {
        PostOrderTraversal {
            current_node: root,
            horizontal: false,
            vertical: false,
            finished: false,
        }
    }
}

impl<N: Ord + Eq + PartialOrd + PartialEq + Copy + Clone> Iterator for PostOrderTraversal<N> {
    type Item = Rc<BinaryTreeNode<N>>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.finished {
            return None; // iterator finished
        }
        loop {
            match (self.horizontal, self.vertical) {
                (false, false) => match self.current_node.get_left() {
                    Some(node) => self.current_node = node,
                    None => self.horizontal = true,
                },
                (true, false) => {
                    match self.current_node.get_right() {
                        Some(node) => {
                            self.current_node = node;
                            self.horizontal = false;
                        }
                        None => {
                            self.vertical = true;
                            let result = self.current_node.get_self();
                            match self.current_node.get_parent() {
                                Some(node) => {
                                    self.horizontal = self.current_node.get_direction().unwrap();
                                    self.current_node = node;
                                }
                                None => self.finished = true, // this only happens if right child of root is None
                            }
                            return result;
                        }
                    }
                }
                (false, true) => {
                    self.vertical = false;
                    self.horizontal = true;
                }
                (true, true) => {
                    let result = self.current_node.get_self();
                    match self.current_node.get_parent() {
                        Some(node) => {
                            self.horizontal = self.current_node.get_direction().unwrap();
                            self.current_node = node;
                        }
                        None => self.finished = true, // end of tree
                    }
                    return result;
                }
            }
        }
    }
}

struct LevelOrderTraversal<N> {
    current_node: Rc<BinaryTreeNode<N>>,
    horizontal: bool, // false: left, true: right
    vertical: bool,   // false: children, true: parent
    finished: bool,   // true if iterator finished
    target_level: usize,
    current_level: usize,
    node_on_target_level: bool,
}

impl<N: Ord + Eq + PartialOrd + PartialEq + Copy + Clone> LevelOrderTraversal<N> {
    fn new(root: Rc<BinaryTreeNode<N>>) -> Self {
        LevelOrderTraversal {
            current_node: root,
            horizontal: false,
            vertical: false,
            finished: false,
            target_level: 0,
            current_level: 0,
            node_on_target_level: false,
        }
    }
}

impl<N: Ord + Eq + PartialOrd + PartialEq + Copy + Clone> Iterator for LevelOrderTraversal<N> {
    type Item = (Rc<BinaryTreeNode<N>>, usize);

    fn next(&mut self) -> Option<Self::Item> {
        if self.finished {
            return None; // iterator finished
        }
        loop {
            if self.current_level == self.target_level {
                let result = self.current_node.get_self().map(|n| (n, self.target_level));
                match self.current_node.get_parent() {
                    Some(node) => {
                        self.node_on_target_level = true;
                        self.vertical = true;
                        self.horizontal = self.current_node.get_direction().unwrap();
                        self.current_level -= 1;
                        self.current_node = node;
                    }
                    None => self.target_level += 1, // this does only happen if target_level == 0!
                }
                return result;
            }
            match (self.horizontal, self.vertical) {
                (false, false) => match self.current_node.get_left() {
                    Some(node) => {
                        self.current_node = node;
                        self.current_level += 1;
                    }
                    None => self.horizontal = true,
                },
                (true, false) => {
                    match self.current_node.get_right() {
                        Some(node) => {
                            self.current_node = node;
                            self.current_level += 1;
                            self.horizontal = false;
                        }
                        None => {
                            self.vertical = true;
                            match self.current_node.get_parent() {
                                Some(node) => {
                                    self.horizontal = self.current_node.get_direction().unwrap();
                                    self.current_level -= 1;
                                    self.current_node = node;
                                }
                                None => {
                                    if self.node_on_target_level {
                                        // None parent only happens if right child of root is None
                                        self.node_on_target_level = false;
                                        self.horizontal = false;
                                        self.target_level += 1;
                                    } else {
                                        self.finished = true;
                                        return None;
                                    }
                                }
                            }
                        }
                    }
                }
                (false, true) => {
                    self.vertical = false;
                    self.horizontal = true;
                }
                (true, true) => {
                    match self.current_node.get_parent() {
                        Some(node) => {
                            self.horizontal = self.current_node.get_direction().unwrap();
                            self.current_level -= 1;
                            self.current_node = node;
                        }
                        None => {
                            if self.node_on_target_level {
                                // None if root
                                self.node_on_target_level = false;
                                self.horizontal = false;
                                self.vertical = false;
                                self.target_level += 1;
                            } else {
                                self.finished = true;
                                return None;
                            }
                        }
                    }
                }
            }
        }
    }
}

struct PathToNode<N> {
    current_node: Rc<BinaryTreeNode<N>>,
    target_value: N,
    finished: bool, // true if iterator finished
}

impl<N: Ord + Eq + PartialOrd + PartialEq + Copy + Clone> PathToNode<N> {
    fn new(start: Rc<BinaryTreeNode<N>>, target_value: N) -> Self {
        PathToNode {
            current_node: start,
            target_value,
            finished: false,
        }
    }
}

impl<N: Ord + Eq + PartialOrd + PartialEq + Copy + Clone> Iterator for PathToNode<N> {
    type Item = Rc<BinaryTreeNode<N>>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.finished {
            return None; // iterator finished
        }
        let result = self.current_node.get_self();
        match self.target_value.cmp(&self.current_node.value) {
            Ordering::Equal => self.finished = true,
            Ordering::Greater => match self.current_node.get_next_bigger() {
                Some(node) => self.current_node = node,
                None => self.finished = true,
            },
            Ordering::Less => match self.current_node.get_next_smaller() {
                Some(node) => self.current_node = node,
                None => self.finished = true,
            },
        }
        result
    }
}

pub struct BinaryTreeNode<N> {
    value: N,
    count: RefCell<usize>,
    node: RefCell<Weak<BinaryTreeNode<N>>>,
    parent: RefCell<Weak<BinaryTreeNode<N>>>,
    left: RefCell<Option<Rc<BinaryTreeNode<N>>>>,
    right: RefCell<Option<Rc<BinaryTreeNode<N>>>>,
}

impl<N: Ord + Eq + PartialOrd + PartialEq + Copy + Clone> BinaryTreeNode<N> {
    pub fn new(value: N) -> Rc<BinaryTreeNode<N>> {
        let result = Rc::new(BinaryTreeNode {
            value,
            count: RefCell::new(1),
            node: RefCell::new(Weak::new()), // weak reference on itself!
            parent: RefCell::new(Weak::new()),
            left: RefCell::new(None),
            right: RefCell::new(None),
        });
        let node = Rc::downgrade(&result);
        *result.node.borrow_mut() = node;
        result
    }
    pub fn append_value(&self, value: N) -> Rc<BinaryTreeNode<N>> {
        match self.value.cmp(&value) {
            Ordering::Equal => {
                // value already in tree -> increment count and return node
                let node = self.get_self().unwrap();
                *node.count.borrow_mut() += 1;
                node
            }
            Ordering::Greater => {
                let left = match *self.left.borrow() {
                    Some(ref node) => {
                        node.append_value(value);
                        node.get_self()
                    }
                    _ => {
                        let left = BinaryTreeNode::new(value);
                        *left.parent.borrow_mut() = self.node.borrow().clone();
                        let node = Rc::downgrade(&left);
                        *left.node.borrow_mut() = node;
                        Some(left)
                    }
                };
                *self.left.borrow_mut() = left;
                self.get_left().unwrap()
            }
            Ordering::Less => {
                let right = match *self.right.borrow() {
                    Some(ref node) => {
                        node.append_value(value);
                        node.get_self()
                    }
                    _ => {
                        let right = BinaryTreeNode::new(value);
                        *right.parent.borrow_mut() = self.node.borrow().clone();
                        let node = Rc::downgrade(&right);
                        *right.node.borrow_mut() = node;
                        Some(right)
                    }
                };
                *self.right.borrow_mut() = right;
                self.get_right().unwrap()
            }
        }
    }
    pub fn get_value(&self) -> N {
        self.value
    }
    pub fn get_count(&self) -> usize {
        *self.count.borrow()
    }
    pub fn get_self(&self) -> Option<Rc<BinaryTreeNode<N>>> {
        self.node.borrow().upgrade().as_ref().cloned()
    }
    pub fn get_left(&self) -> Option<Rc<BinaryTreeNode<N>>> {
        self.left.borrow().as_ref().cloned()
    }
    pub fn get_right(&self) -> Option<Rc<BinaryTreeNode<N>>> {
        self.right.borrow().as_ref().cloned()
    }
    pub fn get_parent(&self) -> Option<Rc<BinaryTreeNode<N>>> {
        self.parent.borrow().upgrade().as_ref().cloned()
    }
    pub fn get_direction(&self) -> Option<bool> {
        self.parent
            .borrow()
            .upgrade()
            .map(|node| self.value > node.value)
    }
    pub fn get_next_smaller(&self) -> Option<Rc<BinaryTreeNode<N>>> {
        if let Some(node) = self.get_left() {
            return Some(node);
        }
        let mut current_node = self.get_self().unwrap();
        while let Some(node) = current_node.get_parent() {
            if current_node.get_direction().unwrap() {
                // current node is right -> parent is smaller
                return Some(node);
            } else {
                current_node = node;
            }
        }
        None
    }
    pub fn get_next_bigger(&self) -> Option<Rc<BinaryTreeNode<N>>> {
        if let Some(node) = self.get_right() {
            return Some(node);
        }
        let mut current_node = self.get_self().unwrap();
        while let Some(node) = current_node.get_parent() {
            if current_node.get_direction().unwrap() {
                current_node = node;
            } else {
                // current node is left -> parent is bigger
                return Some(node);
            }
        }
        None
    }
    pub fn get_node(&self, value: N) -> Option<Rc<BinaryTreeNode<N>>> {
        self.iter_path_to_node(value).find(|n| n.value == value)
    }
    pub fn get_max_level(&self) -> usize {
        self.iter_level_order_traversal()
            .map(|(_, l)| l)
            .max()
            .unwrap()
    }
    pub fn iter_pre_order_traversal(&self) -> impl Iterator<Item = Rc<BinaryTreeNode<N>>> + use<N> {
        PreOrderTraversal::new(self.get_self().unwrap())
    }
    pub fn iter_in_order_traversal(&self) -> impl Iterator<Item = Rc<BinaryTreeNode<N>>> + use<N> {
        InOrderTraversal::new(self.get_self().unwrap())
    }
    pub fn iter_post_order_traversal(
        &self,
    ) -> impl Iterator<Item = Rc<BinaryTreeNode<N>>> + use<N> {
        PostOrderTraversal::new(self.get_self().unwrap())
    }
    pub fn iter_level_order_traversal(
        &self,
    ) -> impl Iterator<Item = (Rc<BinaryTreeNode<N>>, usize)> + use<N> {
        LevelOrderTraversal::new(self.get_self().unwrap())
    }
    pub fn iter_path_to_node(
        &self,
        value: N,
    ) -> impl Iterator<Item = Rc<BinaryTreeNode<N>>> + use<N> {
        PathToNode::new(self.get_self().unwrap(), value)
    }
}
