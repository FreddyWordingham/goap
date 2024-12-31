use serde::Deserialize;

use crate::{Action, Goal, State};

#[derive(Debug, Deserialize)]
pub struct Config {
    pub max_depth: usize,
    pub solve: String,
    pub state: State,
    pub goals: Vec<Goal>,
    pub actions: Vec<Action>,
}
