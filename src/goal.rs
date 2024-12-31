use serde::Deserialize;

use crate::State;

#[derive(Clone, Debug, Deserialize)]
pub struct Goal {
    weight: f32,
    property: String,
    target: i32,
}

impl Goal {
    // Example: discontentment is how far below target the property is
    pub fn discontentment(&self, state: &State) -> f32 {
        let current_value = *state.properties.get(&self.property).unwrap_or(&0);
        if current_value < self.target {
            ((self.target - current_value).max(0) as f32 / self.target as f32) * self.weight
        } else {
            0.0
        }
    }
}
