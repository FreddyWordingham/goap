use serde::Deserialize;

use crate::State;

#[derive(Clone, Debug, Deserialize)]
pub enum DiscontentmentKind {
    StayAbove, // Original: Keep value above the target
    StayBelow, // New: Keep value below the target
}

#[derive(Clone, Debug, Deserialize)]
pub struct Goal {
    weight: f32,
    property: String,
    target: i32,
    kind: DiscontentmentKind,
}

impl Goal {
    pub fn discontentment(&self, state: &State) -> f32 {
        let current_value = *state.properties.get(&self.property).unwrap_or(&0);

        match self.kind {
            DiscontentmentKind::StayAbove => {
                if current_value < self.target {
                    ((self.target - current_value).max(0) as f32 / self.target as f32) * self.weight
                } else {
                    0.0
                }
            }
            DiscontentmentKind::StayBelow => {
                if current_value > self.target {
                    ((current_value - self.target).max(0) as f32 / self.target as f32) * self.weight
                } else {
                    0.0
                }
            }
        }
    }
}
