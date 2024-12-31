use crate::{Action, Goal, State};

#[derive(Debug, Clone)]
pub struct Model {
    pub time: i32,
    pub state: State,
    pub goals: Vec<Goal>,
    pub applied: Vec<Action>,
}

impl Model {
    pub fn new(state: State, goals: Vec<Goal>) -> Self {
        Self {
            time: 0,
            state,
            goals,
            applied: vec![],
        }
    }

    pub fn apply(&self, action: &Action) -> Option<Self> {
        if let Some(next_state) = self.state.apply(action) {
            let mut new_applied = self.applied.clone();
            new_applied.push(action.clone());
            Some(Self {
                time: self.time + action.duration,
                state: next_state,
                goals: self.goals.clone(),
                applied: new_applied,
            })
        } else {
            None
        }
    }

    pub fn calculate_discontentment(&self) -> f32 {
        self.goals
            .iter()
            .map(|g| g.discontentment(&self.state))
            .sum()
    }
}
