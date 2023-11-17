use serde::{Deserialize, Serialize};

use std::fs::File;
use std::env;
use std::io::Read;
use std::collections::HashMap;
use std::collections::HashSet;
use std::time::Instant;

#[derive(Serialize, Deserialize, Debug, Clone)]
struct Point {
    x: f64,
    y: f64,
    id: u32,
}

#[derive(Serialize, Deserialize, Debug)]
struct Points {
    points: Vec<Point>,
}

#[derive(Clone)]
struct DistanceMap {
    map: HashMap<(u32, u32), f64>,
    num_points: u32,
}

impl DistanceMap {
    fn new(points: &Points) -> DistanceMap {
        //let size = (points.points.len()*(points.points.len()-1))/2;

        let mut map = DistanceMap{ map: HashMap::new(), num_points: points.points.len() as u32 };

        for (i, point1) in points.points.iter().enumerate() {
            for (j, point2) in points.points.iter().enumerate().skip(i + 1) {
                let i_u32 = i as u32;
                let j_u32 = j as u32;
                map.map.insert((i_u32, j_u32), DistanceMap::get_distance(point1, point2));
            }
        }

        map
    }
    
    fn point_count(&self) -> usize {
        self.num_points as usize
    }

    fn len(&self) -> usize {
        self.point_count()
    }

    fn get_distance_from_points(&self, point1: &u32, point2: &u32) ->f64 {
        if *point1 == *point2 { return 0.0; }

        let (smaller, larger) = if point1 < point2 {
            (point1, point2)
        } else {
            (point2, point1)
        };
    
        *self.map.get(&(*smaller, *larger)).unwrap_or(&0.0)
    }

    fn get_distance(point1: &Point, point2: &Point) -> f64 {
        //(f64::powf(point1.x-point2.x,2.0) + f64::powf(point1.y-point2.y,2.0)).sqrt()
        (f64::powf(point1.x-point2.x,2.0) + f64::powf(point1.y-point2.y,2.0))
    }    
}

fn get_solution_length(map: &DistanceMap, solution: &Vec<u32>) -> (f64, bool){
    let mut length: f64 = 0.0;

    let is_complete: bool = solution.len() == map.point_count();

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

#[inline(never)]
fn get_greedy(map: &DistanceMap) -> Vec<u32> {
    let mut solution: Vec<u32> = vec![];
    let mut in_solution = HashSet::new();

    let mut current_node: u32 = 0;

    while solution.len() < map.point_count() {

        // Get current node, or set it to zero if it's the first node in the solution.
        if solution.len() > 0 {
            current_node = *solution.last().unwrap();
        } else {
            // Arbitrarily start at node zero.
            solution.push(0);
            in_solution.insert(0);
        }

        let mut distances: Vec<(f64, u32)> = (0..map.point_count() as u32)
            .filter(|&index| !in_solution.contains(&index))
            .map(|index| (map.get_distance_from_points(&current_node, &index), index))
            .collect();

        // Sort by the distance using a custom comparison function
        distances.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap_or(std::cmp::Ordering::Equal));

        let closest = distances[0].1;

        solution.push(closest);
        in_solution.insert(closest);
    }
    solution
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

fn parse_file() -> Points {
    let filename = get_file_name();

    let path = filename.as_str();
    let mut file = File::open(path).expect("File not found");

    let mut data = String::new();
    file.read_to_string(&mut data).expect("Failed to read file");

    let points: Points = serde_json::from_str(&data).expect("Error while deserializing");

    points
}

fn main() {
    // Get points from the json file.
    let points: Points = parse_file();

    let start = Instant::now();

    let map = DistanceMap::new(&points);

    let duration = start.elapsed();

    println!("Now doing no square root.");

    println!("Distances are mapped: {:?}", duration);

    let _greedy_solution: Vec<u32> = get_greedy(&map);

    println!("Solution is found: {:?}", start.elapsed() - duration);


    println!("Total time: {:?}", start.elapsed());
    println!("And we mapped {} points.", map.len());

    // TODO Potentially find 2-OPT from greedy

    // TODO Branch and bound

    // Return optimal solution as a JSON file.
}
