use std::{
    collections::HashMap,
    hash::{Hash, Hasher},
};

use serde::Deserialize;

use crate::Action;

#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
pub struct State(HashMap<String, i32>);

impl State {
    pub fn keys(&self) -> impl Iterator<Item = &String> {
        self.0.keys()
    }

    pub fn get(&self, key: &str) -> Option<&i32> {
        self.0.get(key)
    }

    fn insert(&mut self, key: String, value: i32) {
        self.0.insert(key, value);
    }

    // Try applying an action and return a new State if valid
    pub fn apply(&self, action: &Action) -> Option<Self> {
        let mut new_props = self.clone();
        for (key, delta) in &action.deltas {
            let old_val = *new_props.get(key).unwrap_or(&0);
            let new_val = old_val + delta;
            if new_val < 0 {
                return None;
            }
            new_props.insert(key.clone(), new_val);
        }
        Some(new_props)
    }
}

impl Hash for State {
    fn hash<H: Hasher>(&self, state: &mut H) {
        // BTreeMap iterates in sorted key order, so this is deterministic.
        for (key, value) in &self.0 {
            key.hash(state);
            value.hash(state);
        }
    }
}
