use std::collections::HashMap;

use crate::{Action, Goal, State};

#[derive(Debug, Clone)]
pub struct Model {
    pub time: i32,
    pub state: State,
    pub goals: HashMap<String, Goal>,
    pub action_history: Vec<Action>,
}

impl Model {
    /// Create a new model with the given state and goals.
    pub fn new(state: State, goals: HashMap<String, Goal>) -> Self {
        Self {
            time: 0,
            state,
            goals,
            action_history: vec![],
        }
    }

    pub fn apply(&self, action: &Action) -> Option<Self> {
        if let Some(next_state) = self.state.apply(action) {
            let mut updated_action_history = self.action_history.clone();
            updated_action_history.push(action.clone());
            Some(Self {
                time: self.time + action.duration,
                state: next_state,
                goals: self.goals.clone(),
                action_history: updated_action_history,
            })
        } else {
            None
        }
    }

    pub fn calculate_discontentment(&self) -> f32 {
        let mut total_discontentment = 0.0;
        for (name, goal) in self.goals.iter() {
            let current_value = *self.state.get(name).unwrap_or(&0);
            let discontentment = goal.discontentment(current_value);
            total_discontentment += discontentment;
        }
        total_discontentment
    }
}
