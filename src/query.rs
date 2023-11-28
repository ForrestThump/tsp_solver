#[derive(PartialEq, Debug)]
pub enum Usage {
    Generate,
    SolveOptimal,
    SolveLocal,
}

pub struct UserQuery {
    pub usage:Usage,
    pub points:u32,
    pub filename:String,
    pub time:u32,
    pub max_points:u32,
}

impl UserQuery {
    pub fn new() -> UserQuery {
        UserQuery {usage: Usage::SolveLocal, points: 0, filename: String::from("points.json"), time: 60 as u32, max_points: 1000000000 }
    }
}
