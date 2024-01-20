#[derive(PartialEq, Clone)]
pub struct GraphNode<N> {
    pub id: usize,
    pub item: N,
}

impl<N: PartialEq + Clone> GraphNode<N> {
    fn new(id: usize, item: N) -> Self {
        GraphNode { id, item }
    }
}

#[derive(PartialEq, Clone)]
pub enum GraphEdgeDirection {
    Duplex,
    Simplex,
}

#[derive(PartialEq, Clone)]
pub struct GraphEdge<E> {
    pub id: usize,
    pub start: usize,
    pub end: usize,
    pub value: E,
    pub direction: GraphEdgeDirection,
}

impl<E: PartialEq + Clone + Ord> GraphEdge<E> {
    fn new(id: usize, start: usize, end: usize, value: E, direction: GraphEdgeDirection) -> Self {
        GraphEdge {
            id,
            start,
            end,
            value,
            direction,
        }
    }
}

struct LevelOrderTraversal<'a, N, E> {
    graph: &'a Graph<N, E>,
    start_node: usize,
    visited_nodes: Vec<usize>,
    current_node: usize,
    traversal_level: usize,
    privious_level_nodes: Vec<usize>,
    traversal_level_nodes: Vec<usize>,
    finished: bool,
}

impl<'a, N: PartialEq + Clone, E: PartialEq + Clone + Ord> LevelOrderTraversal<'a, N, E> {
    fn new(graph: &'a Graph<N, E>, start_node: usize) -> Self {
        LevelOrderTraversal {
            graph,
            start_node,
            visited_nodes: Vec::with_capacity(graph.node_count),
            current_node: 0, // last element of visited_nodes?
            traversal_level: 0,
            privious_level_nodes: Vec::with_capacity(graph.node_count),
            traversal_level_nodes: Vec::with_capacity(graph.node_count),
            finished: graph.node_count == 0 || graph.get_node_by_id(start_node).is_err(),
        }
    }
    fn visited_node(&mut self, visited_node: usize) -> Option<(&'a GraphNode<N>, usize)> {
        self.visited_nodes.push(visited_node);
        self.traversal_level_nodes.push(visited_node);
        self.graph
            .get_node_by_id(visited_node)
            .map_or_else(|_| None, |n| Some((n, self.traversal_level)))
    }
}

impl<'a, N: PartialEq + Clone, E: PartialEq + Clone + Ord> Iterator
    for LevelOrderTraversal<'a, N, E>
{
    type Item = (&'a GraphNode<N>, usize);

    fn next(&mut self) -> Option<Self::Item> {
        if self.finished {
            return None;
        }
        loop {
            if self.visited_nodes.is_empty() {
                self.traversal_level += 1;
                self.current_node = self.start_node;
                self.visited_nodes.push(self.start_node);
                return self
                    .graph
                    .get_node_by_id(self.start_node)
                    .map_or_else(|_| None, |n| Some((n, 0)));
            }
            match self.graph.edges.iter().find(|e| match e.direction {
                GraphEdgeDirection::Duplex => {
                    (e.start == self.current_node
                        && !self.visited_nodes.iter().any(|&n| n == e.end))
                        || (e.end == self.current_node
                            && !self.visited_nodes.iter().any(|&n| n == e.start))
                }
                GraphEdgeDirection::Simplex => {
                    e.start == self.current_node && !self.visited_nodes.iter().any(|&n| n == e.end)
                }
            }) {
                Some(edge) => {
                    let next_node = if edge.start == self.current_node {
                        edge.end
                    } else {
                        edge.start
                    };
                    return self.visited_node(next_node);
                }
                None => {
                    if !self.privious_level_nodes.is_empty() {
                        self.current_node = self.privious_level_nodes.remove(0);
                    } else if !self.traversal_level_nodes.is_empty() {
                        for node in self.traversal_level_nodes.iter() {
                            self.privious_level_nodes.push(*node);
                        }
                        self.traversal_level_nodes.clear();
                        self.traversal_level += 1;
                        self.current_node = self.privious_level_nodes.remove(0);
                    } else {
                        self.finished = true;
                        return None;
                    }
                }
            }
        }
    }
}

struct LevelOrderEdgeTraversal<'a, N, E> {
    graph: &'a Graph<N, E>,
    start_node: usize,
    visited_nodes: Vec<usize>,
    current_node: usize,
    traversal_level: usize,
    privious_level_nodes: Vec<usize>,
    traversal_level_nodes: Vec<usize>,
    finished: bool,
}

impl<'a, N: PartialEq + Clone, E: PartialEq + Clone + Ord> LevelOrderEdgeTraversal<'a, N, E> {
    fn new(graph: &'a Graph<N, E>, start_node: usize) -> Self {
        LevelOrderEdgeTraversal {
            graph,
            start_node,
            visited_nodes: Vec::with_capacity(graph.node_count),
            current_node: 0, // last element of visited_nodes?
            traversal_level: 0,
            privious_level_nodes: Vec::with_capacity(graph.node_count),
            traversal_level_nodes: Vec::with_capacity(graph.node_count),
            finished: graph.node_count == 0 || graph.get_node_by_id(start_node).is_err(),
        }
    }
    fn visited_node(
        &mut self,
        visited_node: usize,
        edge: &'a GraphEdge<E>,
    ) -> Option<(&'a GraphNode<N>, &'a GraphEdge<E>, usize)> {
        self.visited_nodes.push(visited_node);
        self.traversal_level_nodes.push(visited_node);
        self.graph
            .get_node_by_id(visited_node)
            .map_or_else(|_| None, |n| Some((n, edge, self.traversal_level)))
    }
}

impl<'a, N: PartialEq + Clone, E: PartialEq + Clone + Ord> Iterator
    for LevelOrderEdgeTraversal<'a, N, E>
{
    type Item = (&'a GraphNode<N>, &'a GraphEdge<E>, usize);

    fn next(&mut self) -> Option<Self::Item> {
        if self.finished {
            return None;
        }
        loop {
            if self.visited_nodes.is_empty() {
                self.traversal_level += 1;
                self.current_node = self.start_node;
                self.visited_nodes.push(self.start_node);
            }
            match self.graph.edges.iter().find(|e| match e.direction {
                GraphEdgeDirection::Duplex => {
                    (e.start == self.current_node
                        && !self.visited_nodes.iter().any(|&n| n == e.end))
                        || (e.end == self.current_node
                            && !self.visited_nodes.iter().any(|&n| n == e.start))
                }
                GraphEdgeDirection::Simplex => {
                    e.start == self.current_node && !self.visited_nodes.iter().any(|&n| n == e.end)
                }
            }) {
                Some(edge) => {
                    let next_node = if edge.start == self.current_node {
                        edge.end
                    } else {
                        edge.start
                    };
                    return self.visited_node(next_node, edge);
                }
                None => {
                    if !self.privious_level_nodes.is_empty() {
                        self.current_node = self.privious_level_nodes.remove(0);
                    } else if !self.traversal_level_nodes.is_empty() {
                        for node in self.traversal_level_nodes.iter() {
                            self.privious_level_nodes.push(*node);
                        }
                        self.traversal_level_nodes.clear();
                        self.traversal_level += 1;
                        self.current_node = self.privious_level_nodes.remove(0);
                    } else {
                        self.finished = true;
                        return None;
                    }
                }
            }
        }
    }
}

enum DFSEdgeChoice {
    MinValue,
    MaxValue,
}

struct DepthFirstSearchTraversal<'a, N, E> {
    graph: &'a Graph<N, E>,
    start_node: usize,
    visited_nodes: Vec<usize>,
    current_node: usize,
    edge_ids: Vec<usize>,
    edge_choice: DFSEdgeChoice,
    finished: bool,
}

impl<'a, N: PartialEq + Clone, E: PartialEq + Clone + Ord> DepthFirstSearchTraversal<'a, N, E> {
    fn new(graph: &'a Graph<N, E>, start_node: usize, edge_choice: DFSEdgeChoice) -> Self {
        let node_count = graph.iter_nodes().count();
        DepthFirstSearchTraversal {
            graph,
            start_node,
            visited_nodes: Vec::with_capacity(node_count),
            current_node: 0,
            edge_ids: Vec::with_capacity(node_count),
            edge_choice,
            finished: node_count == 0 || graph.get_node_by_id(start_node).is_err(),
        }
    }
    fn visited_node(&mut self, visited_node: usize, edge_id: usize) -> Option<&'a GraphNode<N>> {
        self.visited_nodes.push(visited_node);
        self.edge_ids.push(edge_id);
        self.current_node = visited_node;
        self.graph
            .get_node_by_id(visited_node)
            .map_or_else(|_| None, Some)
    }
}

impl<'a, N: PartialEq + Clone, E: PartialEq + Clone + Ord> Iterator
    for DepthFirstSearchTraversal<'a, N, E>
{
    type Item = &'a GraphNode<N>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.finished {
            return None;
        }
        loop {
            if self.visited_nodes.is_empty() {
                self.current_node = self.start_node;
                self.visited_nodes.push(self.start_node);
                return self
                    .graph
                    .get_node_by_id(self.start_node)
                    .map_or_else(|_| None, Some);
            }
            // filter edges by visited nodes
            let edge_iterator = self.graph.iter_edges().filter(|(e, ..)| match e.direction {
                GraphEdgeDirection::Duplex => {
                    (e.start == self.current_node
                        && !self.visited_nodes.iter().any(|&n| n == e.end))
                        || (e.end == self.current_node
                            && !self.visited_nodes.iter().any(|&n| n == e.start))
                }
                GraphEdgeDirection::Simplex => {
                    e.start == self.current_node && !self.visited_nodes.iter().any(|&n| n == e.end)
                }
            });
            // get edge depending on edge_choice
            let edge_result = match self.edge_choice {
                DFSEdgeChoice::MinValue => edge_iterator.min_by_key(|(e, ..)| e.value.clone()),
                DFSEdgeChoice::MaxValue => edge_iterator.max_by_key(|(e, ..)| e.value.clone()),
            };
            match edge_result {
                Some((edge, _, _)) => {
                    let next_node = if edge.start == self.current_node {
                        edge.end
                    } else {
                        edge.start
                    };
                    return self.visited_node(next_node, edge.id);
                }
                None => match self.edge_ids.pop() {
                    Some(last_edge) => {
                        let last_edge = self.graph.get_edge_by_id(last_edge).unwrap();
                        self.current_node = if last_edge.start == self.current_node {
                            last_edge.end
                        } else {
                            last_edge.start
                        }
                    }
                    None => {
                        self.finished = true;
                        return None;
                    }
                },
            }
        }
    }
}

#[derive(PartialEq, Clone)]
pub struct Graph<N, E> {
    nodes: Vec<GraphNode<N>>,
    edges: Vec<GraphEdge<E>>,
    node_count: usize,
    edge_count: usize,
    force_unambiguous: bool,
}

impl<N: PartialEq + Clone, E: PartialEq + Clone + Ord> Graph<N, E> {
    pub fn new(
        expected_number_of_nodes: usize,
        expected_number_of_edges: usize,
        force_unambiguous: bool,
    ) -> Self {
        Graph {
            nodes: Vec::with_capacity(expected_number_of_nodes),
            edges: Vec::with_capacity(expected_number_of_edges),
            node_count: 0,
            edge_count: 0,
            force_unambiguous,
        }
    }
    pub fn add_node(&mut self, item: N) -> usize {
        // Creates a new node if item does not exist in graph. Otherwise return id of existing graph.
        if self.force_unambiguous {
            if let Ok(node) = self.get_node_by_item(item.clone()) {
                return node.id;
            }
        }
        let node = GraphNode::new(self.node_count, item);
        let result = self.node_count;
        self.nodes.push(node);
        self.node_count += 1;
        result
    }
    pub fn remove_node_by_item(&mut self, item: N) -> Result<N, &str> {
        let id = match self.get_node_by_item(item) {
            Ok(node) => node.id,
            Err(_) => return Err("node with item does not exist"),
        };
        self.remove_node_by_id(id)
    }
    pub fn remove_node_by_id(&mut self, id: usize) -> Result<N, &str> {
        let position = self
            .nodes
            .iter()
            .position(|n| n.id == id)
            .ok_or("node id does not exist")?;
        let item = self.nodes.remove(position).item;
        while let Some(position) = self.edges.iter().position(|e| e.start == id || e.end == id) {
            self.edges.remove(position);
        }
        Ok(item)
    }
    pub fn get_node_by_item(&self, item: N) -> Result<&GraphNode<N>, &str> {
        let mut iterator = self.nodes.iter().filter(|n| n.item == item);
        let result = iterator.next();
        if iterator.next().is_some() {
            return Err("Unambigous item resolution"); // change this later to Result<> with Graph specific error codes
        }
        result.ok_or("item does not exist")
    }
    pub fn get_node_by_id(&self, id: usize) -> Result<&GraphNode<N>, &str> {
        self.nodes
            .iter()
            .find(|n| n.id == id)
            .ok_or("id does not exist")
    }
    pub fn get_node_item_mut_by_id(&mut self, id: usize) -> Result<&mut N, &str> {
        self.nodes
            .iter_mut()
            .filter(|n| n.id == id)
            .map(|n| &mut n.item)
            .next()
            .ok_or("id does not exist")
    }
    pub fn add_edge(
        &mut self,
        start_id: usize,
        end_id: usize,
        value: E,
        direction: GraphEdgeDirection,
    ) -> Result<usize, &str> {
        if !self.nodes.iter().any(|n| n.id == start_id) {
            return Err("node of id start not found");
        }
        if !self.nodes.iter().any(|n| n.id == end_id) {
            return Err("node of id end not found");
        }
        if start_id == end_id && direction != GraphEdgeDirection::Duplex {
            return Err("looping edge is only allowed with Duplex");
        }
        let edge = GraphEdge::new(self.edge_count, start_id, end_id, value, direction);
        let result = Ok(edge.id);
        self.edges.push(edge);
        self.edge_count += 1;
        result
    }
    pub fn get_edge(
        &self,
        start_id: usize,
        end_id: usize,
        direction: GraphEdgeDirection,
    ) -> Result<&GraphEdge<E>, &str> {
        let mut iterator = self
            .edges
            .iter()
            .filter(|e| e.start == start_id && e.end == end_id && e.direction == direction);
        let result = iterator.next();
        if iterator.next().is_some() {
            return Err("Unambigous edge resolution");
        }
        result.ok_or("edge does not exist")
    }
    pub fn get_edge_by_id(&self, id: usize) -> Result<&GraphEdge<E>, &str> {
        self.edges
            .iter()
            .find(|e| e.id == id)
            .ok_or("edge does not exist")
    }
    pub fn get_edge_value_mut(
        &mut self,
        start_id: usize,
        end_id: usize,
        direction: GraphEdgeDirection,
    ) -> Result<&mut E, &str> {
        let mut iterator = self
            .edges
            .iter_mut()
            .filter(|e| e.start == start_id && e.end == end_id && e.direction == direction)
            .map(|e| &mut e.value);
        let result = iterator.next();
        if iterator.next().is_some() {
            return Err("Unambigous edge resolution");
        }
        result.ok_or("edge does not exist")
    }
    pub fn get_edge_value_mut_by_id(&mut self, id: usize) -> Result<&mut E, &str> {
        self.edges
            .iter_mut()
            .filter(|e| e.id == id)
            .map(|e| &mut e.value)
            .next()
            .ok_or("edge does not exist")
    }
    pub fn remove_edge(&mut self, id: usize) -> Result<E, &str> {
        let position = self.edges.iter().position(|e| e.id == id);
        match position {
            Some(index) => Ok(self.edges.remove(index).value),
            None => Err("Edge not found"),
        }
    }
    pub fn iter_nodes(&self) -> impl Iterator<Item = &GraphNode<N>> {
        self.nodes.iter()
    }
    pub fn iter_edges(
        &self,
    ) -> impl Iterator<Item = (&GraphEdge<E>, &GraphNode<N>, &GraphNode<N>)> {
        // edge, edge start node, edge end node
        self.edges
            .iter()
            .map(move |e| (e, &self.nodes[e.start], &self.nodes[e.end]))
    }
    pub fn iter_level_order_traversal(
        &self,
        start_node: usize,
    ) -> impl Iterator<Item = (&GraphNode<N>, usize)> {
        LevelOrderTraversal::new(self, start_node)
    }
    pub fn iter_level_order_edge_traversal(
        &self,
        start_node: usize,
    ) -> impl Iterator<Item = (&GraphNode<N>, &GraphEdge<E>, usize)> {
        LevelOrderEdgeTraversal::new(self, start_node)
    }
    pub fn iter_depth_first_search_min_value_traversal(
        &self,
        start_node: usize,
    ) -> impl Iterator<Item = &GraphNode<N>> {
        DepthFirstSearchTraversal::new(self, start_node, DFSEdgeChoice::MinValue)
    }
    pub fn iter_depth_first_search_max_value_traversal(
        &self,
        start_node: usize,
    ) -> impl Iterator<Item = &GraphNode<N>> {
        DepthFirstSearchTraversal::new(self, start_node, DFSEdgeChoice::MaxValue)
    }
}
