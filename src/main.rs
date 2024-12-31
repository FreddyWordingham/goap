use std::{cmp::Ordering, fs, time::Instant};

use colored::*;

use goap_ai::{Config, Model, Planner, State};

fn print_state_headers(state: &State) {
    let mut names: Vec<_> = state.properties.keys().collect();
    names.sort();

    let longest_name_length = names.iter().map(|name| name.len()).max().unwrap_or(0); // Default to 0 if there are no keys
    for i in 0..longest_name_length {
        for name in &names {
            let delay = longest_name_length - name.len();
            if i < delay {
                print!("           ");
            } else if let Some(c) = name.chars().nth(i - delay) {
                print!("     {}     ", c);
            } else {
                print!("           ");
            }
        }
        println!();
    }
}

fn print_state_changes(old_state: &State, state: &State) {
    let mut names: Vec<_> = state.properties.keys().collect();
    names.sort(); // Sort the keys alphabetically

    for name in names {
        if let Some(old_value) = old_state.properties.get(name) {
            if let Some(value) = state.properties.get(name) {
                print!("{:>5} ", value);

                let delta = value - old_value;
                match delta.cmp(&0) {
                    Ordering::Greater => print!("{:5}", format!("{:+}", delta).blue()),
                    Ordering::Less => print!("{:5}", format!("{:+}", delta).red()),
                    Ordering::Equal => print!("     "),
                }
            }
        }
    }
}

fn main() {
    let config_file = fs::read_to_string("config.yml").expect("Failed to read config file");
    let config: Config = serde_yaml::from_str(&config_file).expect("Failed to parse YAML");

    // Build the model and planner
    let mut model = Model::new(config.state, config.goals);
    let planner = Planner::new(config.max_depth, config.actions);

    // Plan
    let (score, time, plan) = if config.solve.to_lowercase() == "best" {
        let start = Instant::now();
        let pl = planner.best_plan(&model);
        let duration = start.elapsed();
        println!("Best plan in        : {:?}", duration);
        pl
    } else {
        let start = Instant::now();
        let pl = planner.quick_plan(&model);
        let duration = start.elapsed();
        println!("Quick plan in       : {:?}", duration);
        pl
    };
    println!("Steps               : {}", plan.len());
    println!("Est. Discontentment : {score}");
    println!("Est. Time           : {time}");
    println!("----------------------------");

    print_state_headers(&model.state);
    print_state_changes(&model.state, &model.state);
    println!("[init]");
    for action in plan.iter() {
        if let Some(next_model) = model.apply(action) {
            print_state_changes(&model.state, &next_model.state);
            println!("{}", action.label);
            model = next_model;
        }
    }
}
