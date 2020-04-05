use steam_train_fuel::{simulate, Command::*, GoalSpec, Train};

fn main() {
    println!("Hello, world!");
    let train = Train::new(500);
    println!("{}", train);
    let train = train.travel(200).unwrap();
    println!("{}", train);
    let train = train.stow_fuel(100).unwrap();
    println!("{}", train);
    let train = train.travel(-200).unwrap();
    println!("{}", train);
    let train = train.travel(200).unwrap();
    println!("{}", train);
    let train = train.travel(400).unwrap();
    println!("{}", train);
    // concise simulation, by Commands and the simulate function!
    println!("-----------------");
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
