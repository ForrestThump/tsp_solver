
use std::fs::File;
use std::env;
use std::io::Read;
use std::io;
use std::collections::{HashSet, BinaryHeap};
use std::time::Instant;
use std::time::Duration;
use rand::seq::SliceRandom;
use rand::thread_rng; 

use std::sync::{Arc, Mutex,};
use std::sync::atomic::{AtomicBool, Ordering as AtomicOrdering};

use rayon::prelude::*;

mod point;
use crate::point::Points;

mod solution;
use crate::solution::Solution;

mod distance_map;
use crate::distance_map::DistanceMap;

mod priority_queue_structs;
use crate::priority_queue_structs::Edge;
use crate::priority_queue_structs::DisjointSet;
use crate::priority_queue_structs::Branch;

mod random_tsp;
use crate::random_tsp::RandomTSPGenerator;

mod input_parsers;
mod query;

/* Round the number to avoid fp rounding errors. */
fn round(number: f64) -> f64 {
    (number * 100000000.0).round() / 100000000.0
}

/* Returns the total length of a given solution. */
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

/* Finds the greedy solution to TSP. */
fn get_greedy(map: &DistanceMap) -> Solution {
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

        // Parallel distance calculation
        let mut distances: Vec<(f64, u32)> = (0..map.point_count() as u32)
            .into_par_iter() // Using Rayon's parallel iterator
            .filter(|&index| !in_solution.contains(&index))
            .map(|index| (map.get_distance_from_points(&current_node, &index), index))
            .collect();

        distances.par_sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap_or(std::cmp::Ordering::Equal)); 

        let closest = distances[0].1;

        solution.push(closest);
        in_solution.insert(closest);
    }

    let distance: f64 = get_solution_length(map, &solution).0;

    Solution { route: solution, distance: distance }
}


/* Swap edges leaving v1 and v2 in the route in place. */
fn two_opt_swap(route: &mut Vec<u32>, v1: usize, v2: usize) {
    assert!(v1 < v2, "v1 must be less than v2");

    let mut temp_route = Vec::with_capacity(route.len());

    // 1. Take the route from the start to v1
    temp_route.extend_from_slice(&route[0..=v1]);

    // 2. Reverse the segment from v1+1 to v2 and add to the route
    let mut reversed_segment = route[v1+1..=v2].to_vec();
    reversed_segment.reverse();
    temp_route.extend_from_slice(&reversed_segment);

    // 3. Add the rest of the route from v2+1 to the end
    temp_route.extend_from_slice(&route[v2+1..]);

    // Copy the temp_route back to the original route
    route.clone_from(&temp_route);
}

/* Determines the incremental gain of swapping edges */
fn get_delta(map: &DistanceMap, solution: &Solution, i: &usize, j: &usize) -> f64 {
    let next_i = (i + 1) % solution.route.len();
    let next_j = (j + 1) % solution.route.len();

    let old_i_edge = map.get_distance_from_points(&solution.route[*i], &solution.route[next_i]);
    let old_j_edge = map.get_distance_from_points(&solution.route[*j], &solution.route[next_j]);
    let new_i_edge = map.get_distance_from_points(&solution.route[next_i], &solution.route[next_j]);
    let new_j_edge = map.get_distance_from_points(&solution.route[*i], &solution.route[*j]);

    let added = new_i_edge + new_j_edge - old_i_edge - old_j_edge;

    round(added)
}

/* Local search algorithm swaps edges to find local minima solution from
*  existing passed in solution. */
fn get_two_opt(map: &DistanceMap, solution_input: Solution) -> Solution {
    let solution = Arc::new(Mutex::new(solution_input));
    let improved = Arc::new(AtomicBool::new(false));

    loop {
        let local_improved = improved.clone();
        let local_solution = solution.clone();

        // Extract route length outside of the parallel loop
        let route_len = {
            let sol = local_solution.lock().unwrap();
            sol.route.len()
        };

        // Parallel iteration
        (0..route_len).into_par_iter().for_each(|i| {
            for j in i + 1..route_len {
                let length_delta: f64 = {
                    let sol = local_solution.lock().unwrap();
                    get_delta(map, &sol, &i, &j)
                };

                if length_delta < 0.0 {
                    let mut sol = local_solution.lock().unwrap();
                    two_opt_swap(&mut sol.route, i, j);
                    sol.distance += length_delta;
                    local_improved.store(true, AtomicOrdering::Relaxed);
                    return; // Exit current iteration
                }
            }
        });

        if !improved.load(AtomicOrdering::Relaxed) {
            break;
        } else {
            improved.store(false, AtomicOrdering::Relaxed); // Reset for next iteration
        }
    }

    let mut return_solution: Solution = Arc::try_unwrap(solution).unwrap().into_inner().unwrap();

    /* Recalculate local minima route distance to correct float error. This would not work if the float
    error did not have a consistent tendency to round down.*/
    return_solution.distance = get_solution_length(&map, &return_solution.route).0;
    return_solution
}

/* Consider implementing and analyzing 3-opt. */

/* Recursive branch and bound search function */
fn branch_and_bound_recurse(solution: &mut Solution, 
    map: &DistanceMap, 
    bssf: &Arc<Mutex<f64>>, 
    unvisited: & mut HashSet<u32>, 
    best_solution: &Arc<Mutex<Solution>>,) {

    /* If it is a complete solution */
    if &solution.len() == &map.len() {
        let mut bssf_guard = bssf.lock().unwrap();
        if solution.distance < *bssf_guard {
            let additional_distance = map.get_distance_from_points(&solution.route.last().unwrap(), 
                                                            &solution.route.first().unwrap());

            if solution.distance + additional_distance < *bssf_guard {
                *bssf_guard = solution.distance + additional_distance;
                let mut best_solution_guard = best_solution.lock().unwrap();
                *best_solution_guard = solution.clone();
                best_solution_guard.distance = *bssf_guard;
                drop(bssf_guard);
                drop(best_solution_guard);
            }
        }
    } else {

        // Grab bssf and drop lock quickly to avoid clashing
        let bssf_guard = bssf.lock().unwrap();
        let stale_bssf = *bssf_guard;
        drop(bssf_guard);

        /* This could be a point for micro-optimization, but it's kind of a pain. */
        let temp_nodes: Vec<u32> = unvisited.iter().cloned().collect();

        for node in temp_nodes {
            let additional_distance = map.get_distance_from_points(&node, solution.route.last().unwrap());

            if solution.distance + additional_distance < stale_bssf {
                solution.distance += additional_distance;
                solution.route.push(node);
                unvisited.remove(&node);
                branch_and_bound_recurse(solution, map, bssf, unvisited, best_solution);
                unvisited.insert(node);
                solution.route.pop();
                solution.distance -= additional_distance;
            }
        }
    }
}

/* Parallelize the branch and bound search */
fn parallel_branch_and_bound(map: &DistanceMap, bssf_input: f64, best_solution: &Solution) -> Solution {
    /* Init mutex objects */
    let bssf = Arc::new(Mutex::new(f64::from(bssf_input)));
    let best_solution_arc = Arc::new(Mutex::new(best_solution.clone()));

    // Assuming the start node is 0 and branching out to different nodes
    let start_node = 0;
    let unvisited: Vec<u32> = (1..map.len() as u32).collect();  // Starting from 1 as 0 is the start node

    /* Parallel loop */
    unvisited.par_iter().for_each(|&node| {
        let bssf_clone = Arc::clone(&bssf);
        let best_solution_clone = Arc::clone(&best_solution_arc);
        let mut solution = Solution { route: vec![start_node, node], distance: map.get_distance_from_points(&start_node, &node) };
        let mut unvisited_thread: HashSet<u32> = unvisited.iter().cloned().filter(|&n| n != node).collect();

        /* Recursive call will return the optimal solution in the best_solution_arc memory location */
        branch_and_bound_recurse(&mut solution, map, &bssf_clone, &mut unvisited_thread, &best_solution_clone);
    });

    let solution_clone = best_solution_arc.lock().unwrap().clone();
    solution_clone
}

fn calculate_heuristic_estimate(map: &DistanceMap, branch: &Branch) -> f64 {
    let unvisited: HashSet<u32> = (0..map.point_count() as u32)
        .collect::<HashSet<_>>()
        .difference(&branch.route.iter().cloned().collect())
        .cloned()
        .collect();

    // Example: Using the sum of the shortest distances from each unvisited node
    unvisited.iter()
        .map(|&node| {
            unvisited.iter()
                .filter(|&&other_node| node != other_node)
                .map(|&other_node| map.get_distance_from_points(&node, &other_node))
                .min_by(|a, b| a.partial_cmp(b).unwrap())
                .unwrap_or(0.0)
        })
        .sum()
}

fn get_heuristic_kruskals(map: &DistanceMap) -> f64 {
    // Collect edges
    let mut edges = Vec::new();
    for key in map.map.iter() {
        edges.push(Edge {
            node1: key.key().0,
            node2: key.key().1,
            distance: *key.value(),
        });
    }

    // Sort edges by distance
    edges.par_sort_unstable_by(|a, b| a.distance.partial_cmp(&b.distance).unwrap());

    // Create MST using disjoint set
    let mut ds = DisjointSet::new(map.point_count());
    let mut mst_weight = 0.0;
    for edge in edges {
        let x = ds.find(edge.node1);
        let y = ds.find(edge.node2);

        if x != y {
            mst_weight += edge.distance;
            ds.union(x, y);
        }
    }

    mst_weight
}

fn parallel_priority_queue_bnb(map: &DistanceMap, bssf_input: &mut f64,) -> Solution {
    let mut best_solution = Solution::new();
    let bssf = Arc::new(Mutex::new(f64::from(*bssf_input)));

    let mut queue = BinaryHeap::new();

    let initial_estimate: f64 = get_heuristic_kruskals(&map);

    queue.push( Branch {route: vec![0], total_distance: 0.0, heuristic_estimate: initial_estimate, } );

    while let Some(branch) = queue.pop() {
        if branch.route.len() == map.point_count() {
            let mut bssf_guard = bssf.lock().unwrap();
            let solution_distance = get_solution_length(map, &branch.route).0;
            if solution_distance < *bssf_guard {
                best_solution.route = branch.route.clone();
                best_solution.distance = solution_distance;
                *bssf_guard = solution_distance;
            }
        } else {
            // Generate new branches in parallel
            let new_branches: Vec<Branch> = (0..map.point_count() as u32)
                .into_par_iter()
                .filter(|&node| !branch.route.contains(&node))
                .map(|node| {
                    let mut new_route = branch.route.clone();
                    new_route.push(node);
                    let new_total_distance = get_solution_length(&map, &new_route).0;
                    let new_heuristic_estimate = calculate_heuristic_estimate(&map, &branch);
                    Branch {
                        route: new_route,
                        total_distance: new_total_distance,
                        heuristic_estimate: new_heuristic_estimate,
                    }
                })
                .filter(|new_branch| new_branch.total_distance + new_branch.heuristic_estimate < *bssf.lock().unwrap())
                .collect();
    
            // Sequentially insert new branches into the queue
            for new_branch in new_branches {
                queue.push(new_branch);
            }
        }
    }

    best_solution
}

/* While this can return the optimal solution, the O(n!) worst-case time complexity makes
   it a pretty bad idea to use if you are mapping more than ~16 points.

   16 points runs for about 5 minutes. Not sure for 17 points.
   
   Additionally, the optimal solution actually typically returns the 2-opt local minima
   solution. So the low chance of getting a better solution usually isn't worth the time.*/
fn get_optimal(map: &DistanceMap, bssf: &mut f64, best_solution: &Solution) -> Solution {
    let using_queue = false;

    if using_queue {
        parallel_priority_queue_bnb(map, bssf)
    } else {

        /* This one runs faster. 
        *  Possibly try to improve this one with heuristic driling.*/
        parallel_branch_and_bound(map, *bssf, best_solution)
    }
    
    /* Possibly replace branch and bound with the dynamic TSP algorithm in the future? */
}

fn parse_file(filename: &String) -> Points {
    let path = filename.as_str();
    let mut file = File::open(path).expect("File not found");

    let mut data = String::new();
    file.read_to_string(&mut data).expect("Failed to read file");

    let points: Points = serde_json::from_str(&data).expect("Error while deserializing");

    points
}

/* Return a random TSP solution for testing purposes. */
fn get_random_solution(map: &DistanceMap) -> Solution {
    let mut vec: Vec<u32> = (0..map.len() as u32).collect();
    vec.shuffle(&mut thread_rng());

    Solution { route: vec.clone(), distance: get_solution_length(&map, &vec).0 }
}

/* Print out the provided solution. */
fn print_solution(solution: &Solution) {
    print!("Solution is: ");

    for i in solution.route.iter() {
        print!("{}, ", i);
    }

    println!("");
    println!("Distance: {}", solution.distance);
    println!("");
}

fn generate_points(query: query::UserQuery) {
    let generator = RandomTSPGenerator::new(1000 as f64, 1000 as f64);

    let count: u32 = query.points;

    let filename: String = format!("points{}.json", count);

    generator.generate(&count, &filename);
}



fn export_solution(solution: &Solution, filename: String) {
    let json_string = serde_json::to_string_pretty(solution).expect("Error converting to JSON");
    RandomTSPGenerator::write_to_file(json_string, filename);
}

fn solve_tsp(filename: &String, usage: &query::Usage, time: u64) -> Solution {
    let start = Instant::now();

    let map = get_map_from_file(filename);

    let greedy_solution: Solution = get_greedy(&map);

    let mut best_solution: Solution = get_two_opt(&map, greedy_solution.clone());

    let mut bssf: f64 = best_solution.distance;

    if *usage == query::Usage::SolveLocal {
        let max_duration: Duration = Duration::new(time, 0);

        while start.elapsed() < max_duration {
            let random_solution: Solution = get_random_solution(&map);
            let new_solution: Solution = get_two_opt(&map, random_solution);

            if round(new_solution.distance) != round(get_solution_length(&map, &new_solution.route).0) {
                println!("get_two_opt is returning solutions with incorrect distances.")
            }
            
            if new_solution.distance < bssf {
                bssf = new_solution.distance;
                best_solution = new_solution.clone();
            }
        }
    } else {
        assert_eq!(*usage, query::Usage::SolveOptimal);
        best_solution = get_optimal(&map, &mut bssf, &best_solution.clone());
    }

    best_solution
}

fn process_query(mut query: query::UserQuery) {
    if query.usage == query::Usage::Generate {
        generate_points(query);
    } else {
        let best_solution = solve_tsp(&query.filename, &query.usage, query.time.clone() as u64);
        
        let solution_type = if query.usage == query::Usage::SolveLocal { "_local" } else {"_optimal"};

        if query.filename.ends_with(".json"){
            query.filename = query.filename.replace(".json", solution_type);
        } else {
            query.filename.push_str(solution_type);
        }

        query.filename.push_str("_solution.json");

        export_solution(&best_solution, query.filename.clone());
    }
}

fn get_map_from_file(filename: &String) -> DistanceMap {
    let points: Points = parse_file(filename);

    DistanceMap::new(&points)
}

fn run_test() {
    let file_string: &String = &"points1000.json".to_string();

    let time: Instant = Instant::now();

    println!("Finding local solution...");

    let solution = solve_tsp(file_string, &query::Usage::SolveLocal, 0 as u64);

    println!("Local solution found!");
    
    let map = get_map_from_file(file_string);

    assert_eq!(solution.distance, get_solution_length(&map, &solution.route).0);

    println!("Ran for {} seconds.", time.elapsed().as_secs() as f64);
}


fn main() {

    if true {
        run_test();
        return;
    }

    let args: Vec<_> = env::args().collect();

    if args.len() < 2 {
        input_parsers::print_help();
        return;
    } else {
        match args[1].to_ascii_lowercase().as_str() {
            "h" | "-h" | "--h" | "help" | "-help" | "--help" => {
                input_parsers::print_help();
                return;
            }
            _ => {}
        };
    }

    let mut query = query::UserQuery::new();

    query.usage = input_parsers::get_usage(&args);

    if query.usage == query::Usage::Generate {
        input_parsers::parse_generate(&mut query, &args);
    } else {
        input_parsers::parse_solve(&mut query, &args);
    }

    process_query(query);
}