use std::{
    cmp::Ordering,
    collections::{BinaryHeap, HashMap},
};

use crate::{Action, Model, State};

#[derive(Debug)]
pub struct Planner {
    max_depth: usize,
    actions: Vec<Action>,
}

impl Planner {
    pub fn new(max_depth: usize, actions: Vec<Action>) -> Self {
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
