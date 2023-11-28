use crate::query;
use std::fs::File;
use std::io;

fn print_local_options() {
    println!("Options for 'solve_local':");
    print!("<file>               ");
    println!("Path to the .json file containing the points to solve");
    print!("<runtime>            ");
    println!("Desired runtime in seconds");
    print!("                     ");
    println!("Example: ./tsp solve_local points10.json 60");
    println!("");
}

fn print_optimal_options() {
    println!("Options for 'solve_optimal':");
    print!("<file>               ");
    println!("Path to the .json file containing the points to solve");
    print!("                     ");
    println!("Example: ./tsp solve_optimal points10.json");
    println!("");
}

fn print_generate_options() {
    println!("Options for 'generate':");
    print!("<number>             ");
    println!("Number of points to generate");
    print!("                     ");
    println!("Example: ./tsp generate 100");
    println!("");
}

fn print_usage() {
    println!("Usage: ./tsp <command> [options]");
    println!("");
    println!("Commands:");
    print!("generate             ");
    println!("Generate data points");
    print!("solve_optimal        ");
    println!("Find the optimal solution");
    print!("solve_local          ");
    println!("Find a local minima solution.");
    println!("");
}

fn print_general_options() {
    println!("General Options:");
    print!("-h, --help, h, help  ");
    println!("Show this help message and exit.");
}

pub fn print_help() {
    print_usage();
    print_generate_options();
    print_optimal_options();
    print_local_options();
    print_general_options();
}





fn determine_local_or_optimal() -> query::Usage {
    let mut input: String = String::new();

    loop {
        input.clear();

        println!("Would you like the local or optimal solution?");

        if let Err(_) = io::stdin().read_line(&mut input) {
            println!("Failed to read line.");
            continue;
        }

        let words: Vec<&str> = input.trim().split_whitespace().collect();

        if words.len() == 0 {
            println!("Invalid input!");
            continue;
        }

        match words[0].to_lowercase().as_str() {
            "optimal" | "solve_optimal" => return query::Usage::SolveOptimal,
            "local" | "solve_local" => return query::Usage::SolveLocal,
            "solve" => if words[1].to_lowercase().as_str() == "optimal" { 
                return query::Usage::SolveOptimal
            } else if words[1].to_lowercase().as_str() == "local" { 
                return query::Usage::SolveLocal 
            }
            _ => { }
        }

        println!("Invalid input! Please enter \"local\" or \"optimal\".");
        continue;
    }
}

fn query_usage() -> query::Usage {
    let mut input = String::new();

    loop {
        input.clear();

        println!("Please indicate your program usage. (\"generate_problem\", \"solve_optimal\", \"solve_local\")");
        
        if let Err(_) = io::stdin().read_line(&mut input) {
            println!("Failed to read line.");
            continue;
        }

        let words: Vec<&str> = input.trim().split_whitespace().collect();

        if words.len() == 0 {
            println!("Invalid input!");
            continue;
        }

        match words[0].to_lowercase().as_str() {
            "generate" | "generate_problem" => return query::Usage::Generate,
            "solve_optimal" | "optimal" => return query::Usage::SolveOptimal,
            "solve_local" | "local" => return query::Usage::SolveLocal,
            "solve" => if words[1].to_lowercase().as_str() == "optimal" { 
                return query::Usage::SolveOptimal
            } else if words[1].to_lowercase().as_str() == "local" { 
                return query::Usage::SolveLocal 
            } else {
                return determine_local_or_optimal();
            }
            _ => println!("Invalid input!"),
        }
    }
}

pub fn get_usage(args: &Vec<String>,) -> query::Usage {
    if args.len() == 1 {
        return query_usage();
    } else {
        match args[1].to_lowercase().as_str() {
            "generate" | "generate_problem" => return query::Usage::Generate,
            "solve_optimal" | "optimal" => return query::Usage::SolveOptimal,
            "solve_local" | "local" => return query::Usage::SolveLocal,
            _ => { 
                println!("Invalid usage parameter."); 
                return query_usage(); }
        }
    }
}

pub fn parse_generate(query: &mut query::UserQuery, args: &Vec<String>,) {
    let mut query_ok: bool = true;

    if args.len() > 2 as usize {
        match args[2].trim().parse::<u32>() {
            Ok(num) if num > 0 && num < query.max_points => {
                query.points = num;
            },
            _ =>{
                query_ok = false;
            }
        }
    } else {
        query_ok = false;
    }

    if !query_ok {
        let mut input = String::new();
        let mut attempts = 0;
        loop {
            input.clear();
            println!("How many points do you need?");
            
            if let Err(_) = io::stdin().read_line(&mut input) {
                println!("Failed to read line.");
                continue;
            }
    
            match input.trim().parse::<u32>() {
                Ok(num) if num > 0 && num < query.max_points => {
                    query.points = num;
                    break;
                },
                _ =>{
                    println!("Invalid input. Try again.");
                    attempts += 1;
                    if attempts >= 5 {
                        println!("Max attempts reached. Exiting.");
                        std::process::exit(1);
                    }
                }
            }
        }
    }
}

fn check_file(file_name: &mut String, ok: &mut bool) {
    if File::open(&file_name).is_ok() {
        *ok = true;
        return;
    }

    let mut file_path = file_name.to_string();
    file_path.push_str(".json");            

    if File::open(&file_path).is_ok(){
        *file_name = String::from(file_path);
        *ok = true;
        return;
    }
}

fn get_file_name(query: &mut query::UserQuery, args: &Vec<String>) {
    let mut ok: bool = false;
    
    if args.len() > 2 {
        let mut file_path = args[2].clone();
        check_file(&mut file_path, &mut ok);
        if ok {
            query.filename = file_path;
            return;
        }
    }

    let mut input = String::new();
    loop {
        input.clear();

        println!("Please input your points filename:");
        if let Err(_) = io::stdin().read_line(&mut input) {
            println!("Failed to read line.");
            continue;
        }

        let mut file_path = input.trim().to_string();

        check_file(&mut file_path, &mut ok);

        if ok {
            break;
        } else {
            println!("File not found.");
        }
    };
}

fn get_execution_time(query: &mut query::UserQuery, args: &Vec<String>) {
    let mut seconds: u32 = 0 as u32;
    
    if args.len() > 3 {
        match args[3].parse::<u32>() {
            Ok(number) => {
                seconds = number; // Assign the parsed value to points_count
            },
            Err(_) => {},
        }
    }

    if seconds == 0 {
        let mut input = String::new();

        loop {
            input.clear();
            println!("How many seconds would you like the search to run for?");

            if let Err(_) = io::stdin().read_line(&mut input) {
                println!("Failed to read line.");
                continue;
            }

            input = input.trim().to_string();
            match input.parse::<u32>() {
                Ok(number) => {
                    query.time = number;
                    break;
                },
                Err(_) => { println!("Error: Please input some number of seconds.")},
            }
        }
    } else {
        query.time = seconds;
    }

    
}

pub fn parse_solve(query: &mut query::UserQuery, args: &Vec<String>) {
    get_file_name(query, args);

    if query.usage == query::Usage::SolveLocal {
        get_execution_time(query, args);
    }
}