use serde::{Deserialize, Serialize};
use std::fs::File;
use std::env;
use std::io::Read;
use std::collections::HashMap;
use std::collections::HashSet;

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
        (f64::powf(point1.x-point2.x,2.0) + f64::powf(point1.y-point2.y,2.0)).sqrt()
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

fn greedy_recurse(map: &DistanceMap, depth: u8, solution: &mut Vec<u32>, in_solution: &mut HashSet<u32>) -> f64{
    let mut current_node: &u32 = &0;

    // Get current node, or set it to zero if it's the first node in the solution.
    if solution.len() > 0 {
        current_node = &(solution.last().unwrap());
    } else {
        // Arbitrarily start at node zero.
        solution.push(0);
        in_solution.insert(0 as u32);
        //in_solution[0] = true;
    }

    let mut distances: Vec<(f64, u32)> = (0..map.point_count() as u32)
        .filter(|&index| !in_solution.contains(&index))
        .map(|index| {
            let distance = map.get_distance_from_points(current_node, &index);
            (distance, index)
        })
        .collect();



    // Sort by the distance using a custom comparison function
    distances.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap_or(std::cmp::Ordering::Equal));

    // Take the first 'depth' elements
    let closest: Vec<u32> = distances.iter().take(depth as usize).map(|&(_, point)| point).collect();

    let mut cheapest: f64 = f64::MAX;
    let mut best_push: &u32 = &u32::MAX;

    /* Add each of the closest nodes and run the recursive function on them again.
    *  Keeping track of which one leads to the shortest route.*/
    for i in closest.iter() {
        solution.push(*i);
        in_solution.insert(*i as u32);
        //in_solution[*i as usize] = true;
        let cost:f64 = greedy_recurse(map, depth-1, solution, in_solution);
        
        if cost < cheapest {
            cheapest = cost;
            best_push = i;
        }

        solution.pop();
        in_solution.remove(&(*i as u32));
        //in_solution[*i as usize] = false;
    }

    /* Push the value that was determined to lead to the 
    *  shortest route (looking depth ahead) */
    solution.push(*best_push);
    in_solution.insert(*best_push as u32);
    //in_solution[*best_push as usize] = true;

    /* Return the cost of your route so far */
    return get_solution_length(map, solution).0;
}

#[inline(never)]
fn get_greedy(map: &DistanceMap, mut depth: u8) -> Vec<u32> {

    /* Correct the depth if it's greater than the number of nodes or less than 1.
    * Note: If depth is equal to the number of nodes, then this 
    * is a brute force algorithm with O(n!) complexity.*/
    if depth > map.point_count() as u8 { depth = map.point_count() as u8; }
    else if depth == 0 as u8 { depth = 1; }

    let mut solution: Vec<u32> = vec![];

    //let mut in_solution = vec![false; map.point_count()];

    let mut in_solution = HashSet::new();

    loop {
        /* Greedy recurse will add a node to the solution with
        *  each iteration. Iterations will terminate when the
        *  solution is complete. */
        greedy_recurse(&map, depth, &mut solution, &mut in_solution);

        let is_complete = solution.len() == map.point_count();

        /* Keep looping until you have a solution. */
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

    let map = DistanceMap::new(&points);

    let depth = 1;

    let _greedy_solution: Vec<u32> = get_greedy(&map, depth);

    let _x = 1;

    // if false{ 
    //     for i in 1..5 {
    //         let i: u8 = i as u8;
    //         let _greedy_solution: Vec<u32> = get_greedy(&map, i);
    //         let cost = get_solution_length(&map, &_greedy_solution);
    //         println!("Depth of {} gave us a cost of {:.2} for {} points.", i, cost.0, points.points.len());
    //     }
    // }

    //let _greedy_solution: Vec<u32> = get_greedy(&map, depth);

    // TODO Potentially find 2-OPT from greedy

    // TODO Branch and bound

    // Return optimal solution as a JSON file.
}
