use serde::Serialize;

#[derive(Clone, Debug, Serialize)]
pub struct Solution {
    pub route: Vec<u32>,
    pub distance: f64,
}

impl Solution { 
    pub fn len(&self) -> usize {
        self.route.len()
    }

    pub fn new() -> Solution {
        Solution {route: Vec::new(), distance: 0.0 }
    }
}