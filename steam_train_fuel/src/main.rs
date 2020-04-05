use steam_train_fuel::{simulate, Command::*, GoalSpec};

fn main() {
    let goal = GoalSpec::new(500, 600);
    let cmds = vec![
        Travel(200),
        StowFuel(100),
        Travel(-200),
        Travel(200),
        Travel(400),
    ];
    println!("{}", simulate(goal, cmds.iter()).unwrap());
}
