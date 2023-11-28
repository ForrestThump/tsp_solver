use serde::{Deserialize, Serialize};
use rand::Rng;

use std::fs::File;
use std::io;
use std::io::Write;
use std::path::Path;

const MAX_POINTS:u32 = 1000000000;

#[derive(Serialize, Deserialize)]
struct Point {
    x: f64,
    y: f64,
    id: u32,
}

#[derive(Serialize, Deserialize)]
struct Points {
    points: Vec<Point>,
}

pub struct RandomTSPGenerator {
    max_x_coord: f64,
    max_y_coord: f64,
    count: u32,
}

impl RandomTSPGenerator {
    pub fn generate(&self, count: &u32, filename: &str) {

        let mut points = Points{ 
            points: Vec::new(),
        };
        
        for i in 0..*count {
            points.points.push( Point {
                x: self.round(rand::thread_rng().gen_range(0.0..self.max_x_coord)),
                y: self.round(rand::thread_rng().gen_range(0.0..self.max_y_coord)),
                id: i, 
            });
        }

        let json_string = serde_json::to_string_pretty(&points).expect("Error converting to JSON");
        RandomTSPGenerator::write_to_file(json_string, filename.to_string());

    }
    
    pub fn new(max_x: f64, max_y: f64) -> RandomTSPGenerator {
        RandomTSPGenerator{ max_x_coord: max_x, max_y_coord: max_y, count: 10, }
    }

    pub fn get_points_count(&self) -> u32 {
        let mut input = String::new();
    
        loop {
            input.clear();
    
            println!("How many points do you need?");
            
            if let Err(_) = io::stdin().read_line(&mut input) {
                println!("Failed to read line.");
                continue;
            }
    
            match input.trim().parse::<u32>() {
                Ok(num) if num > 0 && num < MAX_POINTS => {
                    return num;
                },
                _ =>{
                    println!("Invalid input. Try again.");
                }
            }
        }
    }

    pub fn write_to_file(json_string: String, file_name: String) {
        let path = Path::new(file_name.as_str());
        let display = path.display();
    
        let mut file = match File::create(&path){
            Err(why) => panic!("Couldn't create {}: {}", display, why),
            Ok(file) => file,
        };
    
        match file.write_all(json_string.as_bytes()) {
            Err(why) => panic!("Couldn't write to {}: {}", display, why),
            Ok(_) => println!("Successfully wrote to {}", display),
        }
    }

    pub fn round(&self, number: f64) -> f64 {
        (number * 100.0).round() / 100.0
    }
}