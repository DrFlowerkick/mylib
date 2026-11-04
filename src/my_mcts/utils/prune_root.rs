// if root of tree is set to new root, this util may be used to remove all nodes before new root

pub trait PruneToRoot {
    fn prune_to_root(&mut self);
}
