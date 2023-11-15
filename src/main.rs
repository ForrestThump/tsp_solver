use serde::{Deserialize, Serialize};
use std::fs::File;
use std::env;
use std::io::Read;

#[derive(Serialize, Deserialize, Debug, Clone)]
struct Point {
    x: f64,
    y: f64,
}

#[derive(Serialize, Deserialize, Debug)]
struct Points {
    points: Vec<Point>,
}

#[derive(Clone)]
struct DistanceMap {
    distances: Vec<f64>,
    _ids: Vec<u32>,
}

impl DistanceMap {
    fn new(points: &PointsNumbered) -> DistanceMap {
        let size = (points.points.len()*(points.points.len()-1))/2;

        let mut map = DistanceMap{distances: vec![-1.0; size], _ids: (0..points.points.len() as u32).collect(),};

        for (i, point1) in points.points.iter().enumerate() {
            for (j, point2) in points.points.iter().enumerate().skip(i + 1) {
                let i_u32 = i as u32;
                let j_u32 = j as u32;
                map.distances[DistanceMap::get_index(&i_u32, &j_u32)] = DistanceMap::get_distance(point1, point2);
            }
        }
        

        map
    }
    
    fn len(&self) -> usize {
        self._ids.len()
    }

    fn get_distance_from_points(&self, point1: &u32, point2: &u32) ->f64 {
        if *point1 == *point2 { return 0 as f64; }
        
        self.distances[DistanceMap::get_index(point1, point2)]
    }

    fn get_index(point1: &u32, point2: &u32) -> usize{
        assert_ne!(point1, point2, "Cannot index same point. Distance is 0.");

        let (lower, higher) = if point1 < point2 { (point1, point2) } else { (point2, point1) };

        if *lower == 0 as u32 { return *higher as usize; }
    
        // Gets a unique index in the array from two points.
        // Indexes the lower half of the matrix triangle..excluding diagonals.

        //This should really never overflow since our index shouldn't every exceed 4 billion.
        println!("Lower is {} and Higher is {}", lower, higher);
        
        let val:u32 = ((lower - 1)* lower)/2 + higher;
    
        val as usize
    }

    fn get_distance(point1: &PointNumbered, point2: &PointNumbered) -> f64 {
        (f64::powf(point1.point.x-point2.point.x,2.0) + f64::powf(point1.point.y-point2.point.y,2.0)).sqrt()
    }    
}

struct PointNumbered {
    point: Point,
    _id: u32,
}

struct PointsNumbered {
    points: Vec<PointNumbered>,
}

impl PointsNumbered {
    fn new() -> PointsNumbered {
        PointsNumbered { points: Vec::new() }
    }

    fn push(&mut self, point_numbered: PointNumbered) {
        self.points.push(point_numbered);
    }
}

fn get_solution_length(map: &DistanceMap, solution: &Vec<u32>) -> (f64, bool){
    let mut length: f64 = 0.0;

    let is_complete: bool = solution.len() == map.len();

    // Add the total distance from point to point
    for (current, next) in solution.iter().zip(solution.iter().skip(1)) {

        length += map.get_distance_from_points(current, next);
    }

    if is_complete {
        // Add the distance back to the beginning.
        length += map.get_distance_from_points(solution.last().unwrap(),
        solution.first().unwrap());
    }

    (length, is_complete)
}

fn greedy_recurse(map: &DistanceMap, depth: u8, solution: &mut Vec<u32>) -> f64{
    let mut current_node: &u32 = &0;

    // Get current node, or set it to zero if it's the first node in the solution.
    if solution.len() > 0 {
        current_node = &(solution.last().unwrap());
    } else {
        // Arbitrarily start at node zero.
        solution.push(0);
    }

    // Make a list of all values that are not contained in the solution.
    let mut not_in_solution: Vec<u32> = vec![];
    for i in 0..map.len() {
        if !solution.contains(&(i as u32)){
            not_in_solution.push(i as u32);
        }
    }

    // Make a vector to hold the depth closest nodes.
    let mut closest: Vec<u32> = vec![];

    for i in 0..depth {
        // Grab the first three values
        closest.push(i as u32);
    }

    // Check all points to get the closest ones.

    for i in depth..map.len() as u8 {
        for point in closest.iter_mut(){
            if map.get_distance_from_points(&current_node, point) <
                map.get_distance_from_points(&current_node, &(i as u32)) {
                *point = i as u32;                   
            }
        }
    }


    let mut cheapest: f64 = f64::MAX;
    let mut best_push: &u32 = &u32::MAX;

    /* Add each of the closest nodes and run the recursive function on them again.
    *  Keeping track of which one leads to the shortest route.*/
    for i in closest.iter() {
        solution.push(*i);
        let cost:f64 = greedy_recurse(map, depth-1, solution);
        
        if cheapest < cost {
            cheapest = cost;
            best_push = i;
        }

        solution.pop();
    }

    /* Push the value that was determined to lead to the 
    *  shortest route (looking depth ahead) */
    solution.push(*best_push);

    /* Return the cost of your route so far */
    return get_solution_length(map, solution).0;
}

fn get_greedy(map: &DistanceMap, mut depth: u8) -> Vec<u32> {

    /* Correct the depth if it's greater than the number of nodes or less than 1.
    * Note: If depth is equal to the number of nodes, then this 
    * is a brute force algorithm with O(n!) complexity.*/
    if depth > map.len() as u8 { depth = map.len() as u8; }
    else if depth == 0 as u8 { depth += 1; }

    let mut solution: Vec<u32> = vec![];

    loop {
        greedy_recurse(&map, depth, &mut solution);

        let (_length, is_complete) = get_solution_length(&map, &map._ids);

        /* Keep looping until you have a solution. It will be optimized. */
        if is_complete { return solution; }
    }
}


fn get_file_name() -> String {
    let args: Vec<_> = env::args().collect();

    let mut filename: String = String::from("points.json");

    if args.len() > 1 {
        filename = args[1].clone();

        if filename.ends_with(".txt") {
            filename = filename.replace(".txt", ".json");
        } else if !filename.ends_with(".json") {
            filename.push_str(".json");
        }
    }

    filename
}

fn parse_file() -> PointsNumbered {
    let filename = get_file_name();

    let path = filename.as_str();
    let mut file = File::open(path).expect("File not found");

    let mut data = String::new();
    file.read_to_string(&mut data).expect("Failed to read file");

    let points: Points = serde_json::from_str(&data).expect("Error while deserializing");

    let mut points_numbered = PointsNumbered::new();

    for (i, point) in points.points.iter().enumerate() {
        points_numbered.push(PointNumbered { point: point.clone(), _id: i as u32})
    }

    points_numbered
}

fn main() {
    // Get points from the json file.
    let points: PointsNumbered = parse_file();

    let map = DistanceMap::new(&points);

    let depth: u8 = 1;

    //let point_ids: Vec<u32> = (0..points.points.len() as u32).collect();

    let _greedy_solution: Vec<u32> = get_greedy(&map, depth);

    // TODO Find greedy

    // TODO Potentially find 2-OPT from greedy

    // TODO Branch and bound

    // Return optimal solution as a JSON file.
}
