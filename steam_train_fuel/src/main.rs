use steam_train_fuel::{simulate, Command::*, GoalSpec, SimulationSummary};

#[allow(unused)]
fn test_600() -> SimulationSummary {
    let goal = GoalSpec::new(500, 600);
    let cmds = vec![
        Travel(200),
        StowFuel(100),
        Travel(-200),
        Travel(200),
        Travel(400),
    ];
    simulate(goal, cmds.iter())
}

#[allow(unused)]
fn test_800() -> SimulationSummary {
    let goal = GoalSpec::new(500, 800);
    let cmds = vec![
        Travel(200),
        StowFuel(100),
        Travel(-200),
        Travel(200),
        Travel(400),
    ];
    simulate(goal, cmds.iter())
}

fn main() {
    let mut result = test_800();
    println!("{}", result);
    for state in result.states() {
        println!("{}", state);
    }
}
