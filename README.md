# G.O.A.P. AI

Goal Orientated Action Planning AI is a library for creating AI agents which plan their actions based on a set of overall goals.
It is suitable for use in games and simulations where the AI needs to respond dynamically to a changing environment.

## Installation

You can install the library into your project with:

```toml
[dependencies]
goap = "0.2.0"
```

Alternatively, you can clone the repository and set your project to use the local copy:

```shell
git clone https://github.com/FreddyWoringham/goap.git goap
cd goap
```

Then build the `goap` binary with:

```shell
cargo build --release
```

After which you can run the tool:

```shell
cargo run --release config.yml
```

## Configuration

A GOAP agent is configured with a set of `Goal`s it is trying to achieve, and a set of `Action`s it can perform.
At each step, the agent will select which `Action` to perform given the current `State` of the environment, and how unfulfilled the `Goal`s are that it is trying to achieve.

### State

A `State` object is a list of key-value pairs which represent a snapshot of the environment:

```yaml
state:
  energy: 50
  health: 20
```

### Goals

`Goals` are essentially target values of the `State` which the agent is trying to achieve:

```yaml
goals:
  health:
    target: 100
    kind: Minimize time
    weight: 1
  health:
    target: 100
    kind: GreaterThanOrEqualTo
    weight: 4
  energy:
    target: 100
    kind: GreaterThanOrEqualTo
    weight: 1
```

When planning its actions, an agent will try and minimize "discontentment": the total difference between the current `State` and the `Goal`s it is trying to achieve.

$discontentment = \sum_{i=1}^{n} weight_i \times state_i - goal_i$

> Note: This is representative of `GreaterThanOrEqualTo` goals, but different kinds of goals will use different formulae when calculating their discontentment with the current state.

### Actions

`Actions` are the things an agent can do to change the `State` of the environment in order to achieve its `Goal`s (minimize discontentment).

```yaml
actions:
  eat:
    energy: 0
  sleep:
    health: 0
```

### Plan

The agent will plan a sequence of `Action`s to achieve its `Goal`s.
Planning the goal is the expensive part of the process, so you can choose from a few different algorithms to use:

- Best: Uses an exhaustive depth-first search to find the best plan.
- Fast: Uses the A\* algorithm to find a plan quickly, but it may not be the best possible plan.

Additionally, you can set the maximum number of steps the agent will take to plan a sequence of actions.

```yaml
plan:
  solution: Best
  max_steps: 100
```

You can then generate a plan of action:

```text

```
