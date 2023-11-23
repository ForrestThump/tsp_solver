use ordered_float::OrderedFloat;
use std::cmp::Ordering;

pub struct Edge {
    pub node1: u32,
    pub node2: u32,
    pub distance: f64,
}

#[derive(Clone)]
pub struct DisjointSet {
    parent: Vec<u32>,
}

impl DisjointSet {
    pub fn new(size: usize) -> DisjointSet {
        DisjointSet { parent: (0..size as u32).collect() }
    }

    pub fn find(&mut self, i: u32) -> u32 {
        if self.parent[i as usize] != i {
            self.parent[i as usize] = self.find(self.parent[i as usize]);
        }
        self.parent[i as usize]
    }

    pub fn union(&mut self, x: u32, y: u32) {
        let xset = self.find(x);
        let yset = self.find(y);
        if xset != yset {
            self.parent[xset as usize] = yset;
        }
    }
}

#[derive(Clone, Debug)]
pub struct Branch {
    pub route: Vec<u32>,
    pub total_distance: f64,
    pub heuristic_estimate: f64,
}

impl PartialEq for Branch {
    fn eq(&self, other: &Self) -> bool {
        OrderedFloat(self.total_priority()) == OrderedFloat(other.total_priority())
    }
}

impl Eq for Branch {}

impl PartialOrd for Branch {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        OrderedFloat(self.total_priority()).partial_cmp(&OrderedFloat(other.total_priority()))
    }
}

impl Ord for Branch {
    fn cmp(&self, other: &Self) -> Ordering {
        // Reverse order for min-heap behavior in BinaryHeap (which is a max-heap)
        OrderedFloat(other.total_priority()).cmp(&OrderedFloat(self.total_priority()))
    }
}

impl Branch {
    fn total_priority(&self) -> f64 {
        self.total_distance + self.heuristic_estimate
    }
}