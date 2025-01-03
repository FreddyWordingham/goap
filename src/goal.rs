use serde::Deserialize;

use crate::State;

#[derive(Clone, Debug, Deserialize)]
pub enum DiscontentmentKind {
    StayAbove,
    StayBelow,
    StayAt,
}

#[derive(Clone, Debug, Deserialize)]
pub struct Goal {
    property: String,
    target: i32, // Value to achieve
    scale: f32,  // Discontentment per delta from target
    weight: f32, // Overall goal weighting
    kind: DiscontentmentKind,
}

impl Goal {
    pub fn discontentment(&self, state: &State) -> f32 {
        let current_value = *state.properties.get(&self.property).unwrap_or(&0);

        let delta = match self.kind {
            DiscontentmentKind::StayAbove => (self.target - current_value).max(0),
            DiscontentmentKind::StayBelow => (current_value - self.target).max(0),
            DiscontentmentKind::StayAt => (self.target - current_value).abs(),
        };

        self.scale * self.weight * delta as f32
    }
}
