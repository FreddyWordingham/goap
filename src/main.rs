use serde::Deserialize;
use std::cmp::Ordering;
use std::collections::{BinaryHeap, HashMap};
use std::fs;
use std::hash::{Hash, Hasher};

// A helper struct to hold search nodes for A*.
#[derive(Clone)]
struct AStarNode {
    // The cost so far (g-cost) – in this context, the “discontentment” so far.
    cost_so_far: f32,
    // The estimated total cost (f = g + h).
    estimated_total: f32,
    // Time spent for this path.
    time: i32,
    // The current model (state, etc.).
    model: Model,
    // Actions taken to reach this state.
    plan: Vec<Action>,
}

// We need an ordering so the BinaryHeap picks the smallest estimated_total first.
impl PartialEq for AStarNode {
    fn eq(&self, other: &Self) -> bool {
        self.estimated_total == other.estimated_total
    }
}
impl Eq for AStarNode {}
impl Ord for AStarNode {
    fn cmp(&self, other: &Self) -> Ordering {
        // Flip ordering to make the smallest f-cost the "greatest" priority in the heap
        other
            .estimated_total
            .partial_cmp(&self.estimated_total)
            .unwrap_or(Ordering::Equal)
    }
}
impl PartialOrd for AStarNode {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

#[derive(Debug, Deserialize)]
struct Config {
    state: State,
    goals: Vec<Goal>,
    actions: Vec<Action>,
}

#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
struct Action {
    label: String,
    duration: i32,
    deltas: HashMap<String, i32>,
}

#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
struct State {
    properties: HashMap<String, i32>,
}

impl State {
    // Try applying an action and return a new State if valid
    fn apply(&self, action: &Action) -> Option<Self> {
        let mut new_props = self.properties.clone();
        for (key, delta) in &action.deltas {
            let old_val = *new_props.get(key).unwrap_or(&0);
            let new_val = old_val + delta;
            if new_val < 0 {
                return None;
            }
            new_props.insert(key.clone(), new_val);
        }
        Some(Self {
            properties: new_props,
        })
    }
}

impl Hash for State {
    fn hash<H: Hasher>(&self, state: &mut H) {
        // BTreeMap iterates in sorted key order, so this is deterministic.
        for (key, value) in &self.properties {
            key.hash(state);
            value.hash(state);
        }
    }
}

#[derive(Clone, Debug, Deserialize)]
struct Goal {
    weight: f32,
    property: String,
    target: i32,
}

impl Goal {
    // Example: discontentment is how far below target the property is
    fn discontentment(&self, state: &State) -> f32 {
        let current_value = *state.properties.get(&self.property).unwrap_or(&0);
        if current_value < self.target {
            ((self.target - current_value).max(0) as f32 / self.target as f32) * self.weight
        } else {
            0.0
        }
    }
}

#[derive(Debug, Clone)]
struct Model {
    time: i32,
    state: State,
    goals: Vec<Goal>,
    applied: Vec<Action>,
}

impl Model {
    fn new(state: State, goals: Vec<Goal>) -> Self {
        Self {
            time: 0,
            state,
            goals,
            applied: vec![],
        }
    }

    fn apply(&self, action: &Action) -> Option<Self> {
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

    fn calculate_discontentment(&self) -> f32 {
        self.goals
            .iter()
            .map(|g| g.discontentment(&self.state))
            .sum()
    }
}

#[derive(Debug)]
struct Planner {
    max_depth: usize,
    actions: Vec<Action>,
}

impl Planner {
    fn new(max_depth: usize, actions: Vec<Action>) -> Self {
        Self { max_depth, actions }
    }

    fn find_best_plan(
        &self,
        model: &Model,
        depth: usize,
        memo: &mut HashMap<(State, usize), (f32, i32, Vec<Action>)>,
    ) -> (f32, i32, Vec<Action>) {
        let key = (model.state.clone(), depth);
        if let Some(result) = memo.get(&key) {
            return result.clone();
        }

        if depth == 0 {
            let score = model.calculate_discontentment();
            let result = (score, 0, vec![]);
            memo.insert(key, result.clone());
            return result;
        }

        let current_score = model.calculate_discontentment();
        let mut best_score = current_score;
        let mut best_time = 0;
        let mut best_plan = vec![];

        for action in &self.actions {
            if let Some(next_model) = model.apply(action) {
                let (sub_score, sub_time, mut sub_plan) =
                    self.find_best_plan(&next_model, depth - 1, memo);
                // Tie-break by time if scores are the same
                if sub_score < best_score
                    || (sub_score == best_score && sub_time + action.duration < best_time)
                {
                    best_score = sub_score;
                    best_time = sub_time + action.duration;
                    sub_plan.insert(0, action.clone());
                    best_plan = sub_plan;
                }
            }
        }
        let result = (best_score, best_time, best_plan);
        memo.insert(key, result.clone());
        result
    }

    #[allow(dead_code)]
    pub fn best_plan(&self, model: &Model) -> (f32, i32, Vec<Action>) {
        let mut memo = HashMap::new();
        self.find_best_plan(model, self.max_depth, &mut memo)
    }

    /// A* style planner
    #[allow(dead_code)]
    pub fn quick_plan(&self, start_model: &Model) -> (f32, i32, Vec<Action>) {
        // A function to estimate "remaining discontentment" as a heuristic.
        // For example, sum how far each property is from its target, scaled by goal weight.
        fn heuristic(model: &Model) -> f32 {
            model
                .goals
                .iter()
                .map(|g| g.discontentment(&model.state))
                .sum()
        }

        let mut visited: HashMap<State, f32> = HashMap::new();
        let mut frontier = BinaryHeap::new();

        let start_discontent = start_model.calculate_discontentment();
        let start_heuristic = heuristic(start_model);
        frontier.push(AStarNode {
            cost_so_far: start_discontent,
            estimated_total: start_discontent + start_heuristic,
            time: 0,
            model: start_model.clone(),
            plan: vec![],
        });

        while let Some(node) = frontier.pop() {
            // If we've already found a better route to this state, skip
            if let Some(&best_known) = visited.get(&node.model.state) {
                if node.cost_so_far > best_known {
                    continue;
                }
            }

            // If discontentment is 0 (or some small threshold) or we reached max_depth, we stop
            let depth_so_far = node.plan.len();
            if node.model.calculate_discontentment() < f32::EPSILON
                || depth_so_far >= self.max_depth
            {
                return (
                    node.model.calculate_discontentment(),
                    node.time,
                    node.plan.clone(),
                );
            }

            // Mark best cost found so far for this state
            visited.insert(node.model.state.clone(), node.cost_so_far);

            // Expand possible actions
            for action in &self.actions {
                if let Some(next_model) = node.model.apply(action) {
                    let new_g = node.cost_so_far + next_model.calculate_discontentment();
                    let new_time = node.time + action.duration;
                    // If we haven't visited or found a cheaper path, push to frontier
                    if !visited.contains_key(&next_model.state)
                        || new_g < visited[&next_model.state]
                    {
                        let mut new_plan = node.plan.clone();
                        new_plan.push(action.clone());
                        let new_h = heuristic(&next_model);
                        frontier.push(AStarNode {
                            cost_so_far: new_g,
                            estimated_total: new_g + new_h,
                            time: new_time,
                            model: next_model,
                            plan: new_plan,
                        });
                    }
                }
            }
        }
        // If we exhaust the frontier, return the start as a fallback
        (start_discontent, 0, vec![])
    }
}

fn main() {
    let config_file = fs::read_to_string("config.yml").expect("Failed to read config file");
    let config: Config = serde_yaml::from_str(&config_file).expect("Failed to parse YAML");

    for goal in &config.goals {
        println!("{:?}", goal);
    }
    for action in &config.actions {
        println!("{:?}", action);
    }

    println!("Init State:");
    for (key, val) in &config.state.properties {
        println!("{}: {}", key, val);
    }

    // Build the model and planner
    let mut model = Model::new(config.state, config.goals);
    let planner = Planner::new(14, config.actions);

    // Plan
    let (score, time, plan) = planner.quick_plan(&model);
    println!();
    println!("Steps               : {}", plan.len());
    println!("Est. Discontentment : {score}");
    println!("Est. Time           : {time}");
    for (key, val) in &model.state.properties {
        println!("{}: {}", key, val);
    }
    println!();

    for (i, act) in plan.iter().enumerate() {
        if let Some(next_model) = model.apply(act) {
            model = next_model;
            println!(
                "Step {}) [{}:{}] {}",
                i + 1,
                model.calculate_discontentment(),
                model.time,
                act.label
            );
        } else {
            println!(
                "Step {}) [{}:{}] {} (invalid)",
                i + 1,
                model.calculate_discontentment(),
                model.time,
                act.label
            );
            break;
        }

        for (key, val) in &model.state.properties {
            println!("{}: {}", key, val);
        }
        println!();
    }
}
