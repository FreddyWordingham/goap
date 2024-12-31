use std::fs;
// use std::{fs, time::Instant};

use goap_ai::{Config, Model, Planner, State};

fn print_state_headers(state: &State) {
    let mut names: Vec<_> = state.properties.keys().collect();
    names.sort();

    let longest_name_length = names.iter().map(|name| name.len()).max().unwrap_or(0); // Default to 0 if there are no keys
    for i in 0..longest_name_length {
        for name in &names {
            let delay = longest_name_length - name.len();
            if i < delay {
                print!("     ");
            } else {
                if let Some(c) = name.chars().nth(i - delay) {
                    print!("  {}  ", c);
                } else {
                    print!("     ");
                }
            }
        }
        println!();
    }
}

fn print_state_values(state: &State) {
    let mut names: Vec<_> = state.properties.keys().collect();
    names.sort(); // Sort the keys alphabetically

    for name in names {
        if let Some(value) = state.properties.get(name) {
            print!("{:^5}", value);
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
    let (_score, _time, plan) = if config.solve.to_lowercase() == "best" {
        // let start = Instant::now();
        let pl = planner.best_plan(&model);
        // let duration = start.elapsed();
        // println!("Best plan in: {:?}", duration);
        pl
    } else {
        // let start = Instant::now();
        let pl = planner.quick_plan(&model);
        // let duration = start.elapsed();
        // println!("Quick plan in: {:?}", duration);
        pl
    };
    // println!();
    // println!("Steps               : {}", plan.len());
    // println!("Est. Discontentment : {score}");
    // println!("Est. Time           : {time}");
    // println!("----------------------------");
    // for (key, val) in &model.state.properties {
    //     println!("{}: {}", key, val);
    // }
    // println!();

    print_state_headers(&model.state);
    println!("-----------------------------------");
    for action in plan.iter() {
        if let Some(next_model) = model.apply(action) {
            print_state_values(&model.state);
            println!("{}", action.label);
            model = next_model;
        }
    }
}
