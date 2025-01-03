use serde::Deserialize;

use crate::{Action, Goal, State};

#[derive(Clone, Debug, Deserialize)]
pub enum Solution {
    Fast,
    Best,
}

#[derive(Debug, Deserialize)]
pub struct Config {
    pub max_depth: usize,
    pub solution: Solution,
    pub state: State,
    pub goals: Vec<Goal>,
    pub actions: Vec<Action>,
}
