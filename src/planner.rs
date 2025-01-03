use serde::Deserialize;

use std::{
    cmp::Ordering,
    collections::{BinaryHeap, HashMap},
};

use crate::{Action, Model, State};

#[derive(Debug, Clone, Copy, Deserialize)]
pub enum Algorithm {
    Traditional,
    Efficient,
    Hybrid,
}

#[derive(Debug, Clone, Copy, Deserialize)]
pub enum Solution {
    Fast,
    Best,
}

#[derive(Debug)]
pub struct Planner {
    algorithm: Algorithm,
    solution: Solution,
    max_depth: usize,
    actions: Vec<Action>,
}

impl Planner {
    /// Construct a new planner instance.
    pub fn new(
        algorithm: Algorithm,
        solution: Solution,
        max_depth: usize,
        actions: Vec<Action>,
    ) -> Self {
        Self {
            algorithm,
            solution,
            max_depth,
            actions,
        }
    }

    pub fn plan(&self, model: &Model) -> (f32, i32, Vec<Action>) {
        match (self.algorithm, self.solution) {
            (Algorithm::Traditional, Solution::Fast) => self.fast_total_plan(model),
            (Algorithm::Efficient, Solution::Fast) => self.fast_efficiency_plan(model),
            (Algorithm::Hybrid, Solution::Fast) => self.fast_hybrid_plan(model),
            (Algorithm::Traditional, Solution::Best) => {
                let mut memo = HashMap::new();
                self.best_total_plan(model, self.max_depth, &mut memo)
            }
            (Algorithm::Efficient, Solution::Best) => {
                let mut memo = HashMap::new();
                self.best_efficiency_plan(model, self.max_depth, &mut memo)
            }
            (Algorithm::Hybrid, Solution::Best) => {
                let mut memo = HashMap::new();
                self.best_hybrid_plan(model, self.max_depth, &mut memo)
            }
        }
    }

    /// A* fast plan (traditional) focusing on lowering discontentment quickly.
    pub fn fast_total_plan(&self, start_model: &Model) -> (f32, i32, Vec<Action>) {
        // Heuristic: how much discontentment remains?
        fn heuristic(model: &Model) -> f32 {
            model.calculate_discontentment()
        }

        let mut visited: HashMap<State, f32> = HashMap::new();
        let mut frontier = BinaryHeap::new();

        // Initialize
        let start_discontent = start_model.calculate_discontentment();
        let start_h = heuristic(start_model);
        frontier.push(AStarNode {
            cost_so_far: start_discontent,
            estimated_total: start_discontent + start_h,
            time: 0,
            model: start_model.clone(),
            plan: vec![],
        });

        // A* loop
        while let Some(node) = frontier.pop() {
            if let Some(&best_known) = visited.get(&node.model.state) {
                if node.cost_so_far > best_known {
                    continue;
                }
            }
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
            visited.insert(node.model.state.clone(), node.cost_so_far);

            // Expand actions
            for action in &self.actions {
                if let Some(next_model) = node.model.apply(action) {
                    let new_g = node.cost_so_far + next_model.calculate_discontentment();
                    let new_time = node.time + action.duration;
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
        (start_discontent, 0, vec![])
    }

    /// A* plan optimizing efficiency (discontentment reduction per time).
    pub fn fast_efficiency_plan(&self, start_model: &Model) -> (f32, i32, Vec<Action>) {
        // For efficiency, we'll invert "efficiency" into a cost.
        // Higher efficiency => lower cost => A* prioritizes those paths.
        fn efficiency_heuristic(model: &Model) -> f32 {
            // Could still be the raw discontentment as a guiding heuristic.
            model.calculate_discontentment()
        }

        let mut visited: HashMap<State, f32> = HashMap::new();
        let mut frontier = BinaryHeap::new();

        let start_discontent = start_model.calculate_discontentment();
        let start_h = efficiency_heuristic(start_model);
        frontier.push(AStarNode {
            cost_so_far: 0.0, // We'll accumulate "inefficiency" as cost
            estimated_total: start_h,
            time: 0,
            model: start_model.clone(),
            plan: vec![],
        });

        while let Some(node) = frontier.pop() {
            if let Some(&best_known) = visited.get(&node.model.state) {
                if node.cost_so_far > best_known {
                    continue;
                }
            }
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
            visited.insert(node.model.state.clone(), node.cost_so_far);

            // Expand actions
            for action in &self.actions {
                if let Some(next_model) = node.model.apply(action) {
                    let discontent_delta = node.model.calculate_discontentment()
                        - next_model.calculate_discontentment();
                    let efficiency = discontent_delta / action.duration.max(1) as f32;
                    // Accumulate cost as the inverse of efficiency
                    let new_cost = node.cost_so_far + 1.0 / (efficiency + 1e-6);
                    let new_time = node.time + action.duration;

                    if !visited.contains_key(&next_model.state)
                        || new_cost < visited[&next_model.state]
                    {
                        let mut new_plan = node.plan.clone();
                        new_plan.push(action.clone());
                        let new_h = efficiency_heuristic(&next_model);
                        frontier.push(AStarNode {
                            cost_so_far: new_cost,
                            estimated_total: new_cost + new_h,
                            time: new_time,
                            model: next_model,
                            plan: new_plan,
                        });
                    }
                }
            }
        }
        (start_discontent, 0, vec![])
    }

    /// A* plan mixing efficiency and raw discontentment (hybrid).
    pub fn fast_hybrid_plan(&self, start_model: &Model) -> (f32, i32, Vec<Action>) {
        fn hybrid_heuristic(model: &Model) -> f32 {
            model.calculate_discontentment()
        }

        let mut visited: HashMap<State, f32> = HashMap::new();
        let mut frontier = BinaryHeap::new();

        let start_discontent = start_model.calculate_discontentment();
        let start_h = hybrid_heuristic(start_model);
        frontier.push(AStarNode {
            cost_so_far: 0.0,
            estimated_total: start_h,
            time: 0,
            model: start_model.clone(),
            plan: vec![],
        });

        while let Some(node) = frontier.pop() {
            if let Some(&best_known) = visited.get(&node.model.state) {
                if node.cost_so_far > best_known {
                    continue;
                }
            }
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
            visited.insert(node.model.state.clone(), node.cost_so_far);

            for action in &self.actions {
                if let Some(next_model) = node.model.apply(action) {
                    let discontent_delta = node.model.calculate_discontentment()
                        - next_model.calculate_discontentment();
                    let efficiency = discontent_delta / action.duration.max(1) as f32;

                    // Decide if we prioritize efficiency or raw discontentment
                    let use_efficiency = depth_so_far > 2 && efficiency > 0.1;
                    let metric = if use_efficiency {
                        1.0 / (efficiency + 1e-6)
                    } else {
                        next_model.calculate_discontentment()
                    };

                    let new_cost = node.cost_so_far + metric;
                    let new_time = node.time + action.duration;

                    if !visited.contains_key(&next_model.state)
                        || new_cost < visited[&next_model.state]
                    {
                        let mut new_plan = node.plan.clone();
                        new_plan.push(action.clone());
                        let new_h = hybrid_heuristic(&next_model);
                        frontier.push(AStarNode {
                            cost_so_far: new_cost,
                            estimated_total: new_cost + new_h,
                            time: new_time,
                            model: next_model,
                            plan: new_plan,
                        });
                    }
                }
            }
        }
        (start_discontent, 0, vec![])
    }

    /// Exhaustive best plan (traditional), using memoized search.
    fn best_total_plan(
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
            let res = (score, 0, vec![]);
            memo.insert(key, res.clone());
            return res;
        }

        let current_score = model.calculate_discontentment();
        let mut best_score = current_score;
        let mut best_time = 0;
        let mut best_plan = vec![];

        for action in &self.actions {
            if let Some(next_model) = model.apply(action) {
                let (sub_score, sub_time, mut sub_plan) =
                    self.best_total_plan(&next_model, depth - 1, memo);

                // Prioritize lower discontentment, then shorter time
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
        let res = (best_score, best_time, best_plan);
        memo.insert(key, res.clone());
        res
    }

    /// Exhaustive best plan focusing on efficiency.
    fn best_efficiency_plan(
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
            let res = (score, 0, vec![]);
            memo.insert(key, res.clone());
            return res;
        }

        let current_score = model.calculate_discontentment();
        let mut best_eff = f32::MIN;
        let mut best_time = 0;
        let mut best_plan = vec![];

        for action in &self.actions {
            if let Some(next_model) = model.apply(action) {
                let discontent_delta = current_score - next_model.calculate_discontentment();
                let efficiency = discontent_delta / action.duration.max(1) as f32;

                let (sub_score, sub_time, mut sub_plan) =
                    self.best_efficiency_plan(&next_model, depth - 1, memo);
                let sub_discontent_delta = current_score - sub_score;
                let _sub_eff = sub_discontent_delta / (sub_time.max(1) as f32);

                // Pick the path that yields better efficiency
                if efficiency > best_eff
                    || (efficiency == best_eff && sub_time + action.duration < best_time)
                {
                    best_eff = efficiency;
                    best_time = sub_time + action.duration;
                    sub_plan.insert(0, action.clone());
                    best_plan = sub_plan;
                }
            }
        }
        // Reconstruct a "score" from the final efficiency.
        // Alternatively, you might calculate final discontent instead.
        let final_discontent = current_score - best_eff;
        let res = (final_discontent, best_time, best_plan);
        memo.insert(key, res.clone());
        res
    }

    /// Exhaustive best plan using a hybrid strategy.
    fn best_hybrid_plan(
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
            let res = (score, 0, vec![]);
            memo.insert(key, res.clone());
            return res;
        }

        let current_score = model.calculate_discontentment();
        let mut best_metric = f32::MAX;
        let mut best_time = 0;
        let mut best_plan = vec![];

        for action in &self.actions {
            if let Some(next_model) = model.apply(action) {
                let discontent_delta = current_score - next_model.calculate_discontentment();
                let efficiency = discontent_delta / action.duration.max(1) as f32;

                // Decide whether to use raw discontent or efficiency
                let use_efficiency = depth > 2 && efficiency > 0.1;
                let metric = if use_efficiency {
                    // Lower is better, so invert efficiency
                    1.0 / (efficiency + 1e-6)
                } else {
                    // Minimizing discontent
                    next_model.calculate_discontentment()
                };

                let (_sub_score, sub_time, mut sub_plan) =
                    self.best_hybrid_plan(&next_model, depth - 1, memo);

                // Compare metric to decide best path
                if metric < best_metric
                    || (metric == best_metric && sub_time + action.duration < best_time)
                {
                    best_metric = metric;
                    best_time = sub_time + action.duration;
                    sub_plan.insert(0, action.clone());
                    best_plan = sub_plan;
                }
            }
        }
        // Construct final discontent.
        let final_discontent = current_score - (1.0 / best_metric);
        let res = (final_discontent, best_time, best_plan);
        memo.insert(key, res.clone());
        res
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
