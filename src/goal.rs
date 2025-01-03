use serde::Deserialize;

#[allow(clippy::enum_variant_names)]
#[derive(Clone, Debug, Deserialize)]
pub enum DiscontentmentKind {
    GreaterThanOrEqualTo,
    LessThanOrEqualTo,
    EqualTo,
}

#[derive(Clone, Debug, Deserialize)]
pub struct Goal {
    weight: f32, // Weight to apply per difference from the target
    target: i32, // Value to achieve
    pub kind: DiscontentmentKind,
}

impl Goal {
    pub fn discontentment(&self, current_value: i32) -> f32 {
        let delta = match self.kind {
            DiscontentmentKind::GreaterThanOrEqualTo => (self.target - current_value).max(0),
            DiscontentmentKind::LessThanOrEqualTo => (current_value - self.target).max(0),
            DiscontentmentKind::EqualTo => (self.target - current_value).abs(),
        };

        self.weight * delta as f32
    }
}
