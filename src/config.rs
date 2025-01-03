use std::collections::HashMap;

use serde::Deserialize;

use crate::{Action, Algorithm, Goal, Solution, State};

#[derive(Debug, Deserialize)]
pub struct Config {
    pub algorithm: Algorithm,
    pub solution: Solution,
    pub max_depth: usize,
    pub state: State,
    pub goals: HashMap<String, Goal>,
    pub actions: HashMap<String, Action>,
}
