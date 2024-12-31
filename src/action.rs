use std::collections::HashMap;

use serde::Deserialize;

#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
pub struct Action {
    pub label: String,
    pub duration: i32,
    pub deltas: HashMap<String, i32>,
}