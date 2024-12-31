use std::{fs, time::Instant};

use goap_ai::{Config, Model, Planner};

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
        println!("Best plan in: {:?}", duration);
        pl
    } else {
        let start = Instant::now();
        let pl = planner.quick_plan(&model);
        let duration = start.elapsed();
        println!("Quick plan in: {:?}", duration);
        pl
    };
    println!();
    println!("Steps               : {}", plan.len());
    println!("Est. Discontentment : {score}");
    println!("Est. Time           : {time}");
    println!("----------------------------");
    // for (key, val) in &model.state.properties {
    //     println!("{}: {}", key, val);
    // }
    // println!();

    for action in plan.iter() {
        if let Some(next_model) = model.apply(action) {
            println!("{}", action.label);
            model = next_model;
        }
    }
}
