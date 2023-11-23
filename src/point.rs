use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Point {
    pub x: f64,
    pub y: f64,
    pub id: u32,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Points {
    pub points: Vec<Point>,
}