use serde::{Deserialize, Serialize};

use std::fs::File;
use std::env;
use std::io::Read;
use std::collections::HashMap;
use std::collections::HashSet;
use std::time::Instant;

use rayon::prelude::*;
use dashmap::DashMap;

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
    map: DashMap<(u32, u32), f64>,
    num_points: u32,
}

impl DistanceMap {
    fn new(points: &Points) -> DistanceMap {
        let num_points = points.points.len() as u32;
        let map = DashMap::with_capacity(num_points as usize * (num_points as usize - 1) / 2);

        points.points.par_iter().enumerate().for_each(|(i, point1)| {
            for (j, point2) in points.points.iter().enumerate().skip(i + 1) {
                let distance = DistanceMap::get_distance(point1, point2);
                map.insert((i as u32, j as u32), distance);
            }
        });

        DistanceMap { map, num_points }
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
    
        self.map.get(&(*smaller, *larger)).map_or(0.0, |v| *v)
    }

    fn get_distance(point1: &Point, point2: &Point) -> f64 {

        /* Emperical testing with a random, even distribution of points shows that NOT taking the square root
        * yields a 1% slower distance and greedy computation time compared to taking the square root. Evidently, 
        * the square root is not that expensive, and the increased variable size of not taking the square root is 
        * comparatively more expensive.
        *******************************/
        f64::powf(point1.x-point2.x,2.0) + f64::powf(point1.y-point2.y,2.0).sqrt()
    }    
}

#[derive(Clone)]
struct Solution {
    route: Vec<u32>,
    distance: f64,
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

fn two_opt_swap(route: &Vec<u32>, v1: usize, v2: usize) -> Vec<u32> {
    let mut new_route = Vec::with_capacity(route.len());

    // 1. Add route[0] to route[v1] in order
    new_route.extend_from_slice(&route[0..=v1]);

    // 2. Add route[v1+1] to route[v2] in reverse order
    for i in (v1+1..=v2).rev() {
        new_route.push(route[i]);
    }

    // 3. Add route[v2+1] to route[end] in order
    if v2 < route.len() - 1 {
        new_route.extend_from_slice(&route[v2+1..]);
    }

    new_route
}

fn get_delta(map: &DistanceMap, solution: &Solution, i: &usize, j: &usize) -> f64{
    - map.get_distance_from_points(&solution.route[*i], 
        &solution.route[*i + 1]) - map.get_distance_from_points(&solution.route[*j], 
            &solution.route[*j + 1]) + map.get_distance_from_points(&solution.route[*i + 1], 
                &solution.route[*j + 1]) +
                map.get_distance_from_points(&solution.route[*i], &solution.route[*j])
}

fn get_two_opt(map: &DistanceMap, mut solution: Solution) -> Solution {

    loop {
        let mut improved = false;
        'outer_for: for i in 0..solution.route.len() {
            for j in i + 1..solution.route.len() {
                let length_delta: f64 = get_delta(map, &solution, &i, &j);

                if length_delta < 0 as f64 {
                    solution.route = two_opt_swap(&solution.route, i, j);
                    solution.distance -= length_delta;
                    improved = true;
                    break 'outer_for;
                }
            }
        }
        if !improved { break; }
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

    let mut duration = start.elapsed();

    println!("Distances are mapped: +{:?}", duration);

    let greedy_solution: Solution = get_greedy(&map);

    duration = start.elapsed() - duration;

    println!("Greedy solution is found: +{:?}", duration);

    let _two_opt_solution: Solution = get_two_opt(&map, greedy_solution.clone());

    duration = start.elapsed() - duration;

    println!("Two-opt is found: +{:?}", duration);

    println!("Total time: {:?}", start.elapsed());
    println!("Mapped {} points.", map.len());



    // TODO Find 2-OPT from greedy

    // TODO Branch and bound

    // Return optimal solution as a JSON file.
}