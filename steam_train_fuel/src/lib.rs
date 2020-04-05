use std::collections::HashMap;
use std::convert::TryInto;

pub use spec::*;
/// Enforce immutability of the specifications.
///
/// Types within this module have their private types shielded from mutability.
pub mod spec {
    /// Train specifications.
    #[derive(Copy, Clone, Debug, PartialEq)]
    pub struct TrainSpec {
        capacity: usize,
    }
    impl TrainSpec {
        pub fn new(capacity: usize) -> TrainSpec {
            TrainSpec { capacity }
        }
        pub fn capacity(&self) -> usize {
            self.capacity
        }
    }
    impl From<GoalSpec> for TrainSpec {
        fn from(goal: GoalSpec) -> Self {
            goal.train_spec
        }
    }

    /// Goal specifications.
    #[derive(Copy, Clone)]
    pub struct GoalSpec {
        train_spec: TrainSpec,
        destination: usize,
    }
    impl GoalSpec {
        /// Constructs new Goal using the specified capacity and destination.
        ///
        /// NOTE: The destination must be out-of-reach from the fuel capacity,
        /// otherwise the goal is trivially accomplished by a single `Travel(destination)`.
        ///
        /// ```should_panic
        /// use steam_train_fuel::GoalSpec;
        /// // forbids trivial goals
        /// GoalSpec::new(500, 499);
        /// ```
        pub fn new(capacity: usize, destination: usize) -> GoalSpec {
            if destination <= capacity {
                panic!(
                    "illegal trivial goal: destination ({}) is within capacity ({})",
                    destination, capacity
                );
            }
            GoalSpec {
                train_spec: TrainSpec::new(capacity),
                destination,
            }
        }
        pub fn capacity(&self) -> usize {
            self.train_spec.capacity()
        }
        pub fn destination(&self) -> usize {
            self.destination
        }
    }
}

pub type Error = &'static str;

/// Locomotive for transportation. Tracks the location, fuel, and fuel stashes.
#[derive(Debug, PartialEq, Clone)]
pub struct Train {
    spec: TrainSpec,
    //
    location: usize,
    fuel: usize,
    stashes: HashMap<usize, usize>,
}
impl Train {
    /// Creates a new train starting at the origin with full fuel tank.
    ///
    /// ```
    /// use steam_train_fuel::Train;
    /// let train = Train::new(500);
    /// ```
    pub fn new(capacity: usize) -> Self {
        Self::from(TrainSpec::new(capacity))
    }
    pub fn from_goal(goal: GoalSpec) -> Self {
        Self::from(goal.into())
    }
    fn from(spec: TrainSpec) -> Self {
        Train {
            spec,
            location: 0,
            fuel: spec.capacity(),
            stashes: HashMap::new(),
        }
    }

    /// Tavels the specified distance. If returned to depot (location = 0), the fuel is refilled to capacity.
    ///
    /// ```
    /// use steam_train_fuel::Train;
    /// // starting config
    /// let state0 = Train::new(500);
    /// assert_eq!(state0.fuel(), 500);
    /// assert_eq!(state0.location(), 0);
    ///
    /// // travel forward to 200 mile marker, using 200 fuel (300 remaining)
    /// let state200 = state0.travel(200).unwrap();
    /// assert_eq!(state200.fuel(), 300);
    /// assert_eq!(state200.location(), 200);
    ///
    /// // travel backward to 1 mile marker, using 199 fuel (101 remaining)
    /// let state1 = state200.travel(-199).unwrap();
    /// assert_eq!(state1.fuel(), 101);
    /// assert_eq!(state1.location(), 1);
    ///
    /// // travel backward to origin, using 1 fuel then immediately refueling
    /// let state_end = state1.travel(-1).unwrap();
    /// assert_eq!(state_end.fuel(), 500);
    /// assert_eq!(state_end.location(), 0);
    ///
    /// // travel behind origin is forbidden
    /// assert_eq!(state1.travel(-2), Err("moved beyond depot"));
    /// ```
    pub fn travel(&self, distance: isize) -> Result<Train, Error> {
        let Train {
            spec,
            location,
            fuel,
            stashes,
        } = self;
        let mut stashes = stashes.clone();
        let location = ((*location as isize) + distance)
            .try_into()
            .map_err(|_| "moved beyond depot")?;
        let fuel = ((*fuel as isize) - distance.abs())
            .try_into()
            .map_err(|_| "used more fuel than was in the tank")?;
        let fuel = if let Some(stashed) = self.stowed_at(location) {
            stashes.remove(&location);
            fuel + stashed
        } else if location == 0 {
            spec.capacity()
        } else {
            fuel
        };
        Ok(Train {
            spec: *spec,
            location,
            fuel,
            stashes,
        })
    }

    /// Stows the specified amount of fuel at the current position.
    ///
    /// ```
    /// use steam_train_fuel::Train;
    /// let state = Train::new(500);
    /// let state = state.travel(150).unwrap();
    /// assert_eq!(state.fuel(), 500-150);
    /// // stow 50 units of fuel
    /// assert_eq!(state.stowed_at(150), None);
    /// let state = state.stow_fuel(50).unwrap();
    /// assert_eq!(state.fuel(), 500-150-50);
    /// assert_eq!(state.stowed_at(150), Some(50));
    ///
    /// // pickup the stowed fuel
    /// assert_eq!(state.fuel(), 300);
    /// let state = state.travel(-100).unwrap();
    /// assert_eq!(state.fuel(), 200);
    /// assert_eq!(state.stowed_at(150), Some(50));
    /// let state = state.travel(100).unwrap();
    /// assert_eq!(state.fuel(), 150);
    /// assert_eq!(state.stowed_at(150), None);
    ///
    /// // cannot stow at the depot (origin)
    /// let state = Train::new(500);
    /// assert_eq!(state.stow_fuel(1), Err("cannot stow fuel at the depot"));
    /// ```
    pub fn stow_fuel(&self, amount: usize) -> Result<Train, Error> {
        let Train {
            spec,
            location,
            fuel,
            stashes,
        } = self;
        if *location == 0 {
            return Err("cannot stow fuel at the depot");
        }
        // subtract from fuel tank
        let fuel = (*fuel - amount)
            .try_into()
            .map_err(|_| "stowed more fuel than was in the tank")?;
        // add to stash
        let mut stashes = stashes.clone();
        let stash_amount = stashes.entry(*location).or_insert(0);
        *stash_amount += amount;
        Ok(Train {
            spec: *spec,
            location: *location,
            fuel,
            stashes,
        })
    }
    pub fn update(&self, command: Command) -> Result<Train, Error> {
        match command {
            Command::Travel(distance) => {
                self.travel(distance)
            }
            Command::StowFuel(amount) => {
                self.stow_fuel(amount)
            }
        }
    }

    pub fn fuel(&self) -> usize {
        self.fuel
    }
    pub fn location(&self) -> usize {
        self.location
    }
    pub fn stowed_at(&self, location: usize) -> Option<usize> {
        self.stashes.get(&location).copied()
    }
    pub fn meets_goal(&self, goal: &GoalSpec) -> bool {
        self.location == goal.destination()
    }
}
use std::fmt;
impl fmt::Display for Train {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        const STEP: usize = 25;
        for x in (0..).step_by(STEP) {
            let symbol = if x % 100 == 0 { "|" } else { "=" };
            write!(f, "{} ", symbol)?;
            if x + STEP > self.location {
                break;
            }
        }
        writeln!(f)?;
        for (location, stashed) in &self.stashes {
            writeln!(
                f,
                "{1:0$}^[{2} stash @{3}]",
                (*location / STEP) * 2,
                " ",
                stashed,
                location
            )?;
        }
        write!(f, "@ {}, fuel {}", self.location, self.fuel)
    }
}

/// Elementary commands for a train
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Command {
    /// Travel the specified distance +forward, or -backward
    Travel(isize),
    /// Stow the specified amount of fuel at the current location
    StowFuel(usize),
}
/// Agent deciding how to command the train to achieve a goal.
pub trait Strategy {
    fn decide(&mut self, state: &Train, goal: &GoalSpec) -> Option<Command>;
}

/// Summarizes the result of a `simlation` run.
pub struct SimulationSummary {
    goal: GoalSpec,
    final_state: Train,
    commands: Vec<Command>,
}
impl SimulationSummary {
    pub fn new(goal: GoalSpec, final_state: Train, commands: Vec<Command>) -> Self {
        Self {
            goal,
            final_state,
            commands,
        }
    }
    pub fn goal(&self) -> &GoalSpec {
        &self.goal
    }
    pub fn final_state(&self) -> &Train {
        &self.final_state
    }
    pub fn commands(&self) -> &[Command] {
        &self.commands
    }
    pub fn fuel_used(&self) -> usize {
        self.commands
            .iter()
            .map(|c| match c {
                Command::Travel(distance) => distance.abs() as usize,
                _ => 0,
            })
            .sum()
    }
}
impl fmt::Display for SimulationSummary {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "fuel_used = {}, steps = {}", self.fuel_used(), self.commands().len())
    }
}

/// Simulates a train following the specified Strategy's commands.
///
/// ```
/// use steam_train_fuel::{GoalSpec, Command::{self, *}, simulate};
/// let goal = GoalSpec::new(500, 600);
/// let cmds = vec![
///     Travel(200),
///     StowFuel(100),
///     Travel(-200),
///     Travel(200),
///     Travel(400)
/// ];
/// let result = simulate(goal, cmds.iter()).unwrap();
/// assert_eq!(result.fuel_used(), 1_000);
/// assert_eq!(result.commands(), &cmds[..]);
/// ```
pub fn simulate<S: Strategy>(goal: GoalSpec, mut strategy: S) -> Result<SimulationSummary, Error> {
    let mut state = Train::from_goal(goal);
    let mut commands = Vec::new();
    for _ in 0..20 {
        println!("{}", state);
        if state.meets_goal(&goal) {
            let summary = SimulationSummary::new(goal, state, commands);
            return Ok(summary);
        }
        match strategy.decide(&state, &goal) {
            Some(command) => {
                commands.push(command);
                state = state.update(command)?;
            }
            None => {
                return Err("strategy returned None");
            }
        }
    }
    Err("simulation max iteration couter reached")
}

impl<'a, T> Strategy for T
where
    T: Iterator<Item = &'a Command>,
{
    fn decide(&mut self, _state: &Train, _goal: &GoalSpec) -> Option<Command> {
        self.next().copied()
    }
}
