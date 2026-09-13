#[path = "../examples/wolf_sheep_grass/agent.rs"]
mod agent;
#[path = "../examples/wolf_sheep_grass/cell.rs"]
mod cell;
#[path = "../examples/wolf_sheep_grass/model.rs"]
mod model;
#[path = "../examples/wolf_sheep_grass/stats.rs"]
mod stats;

use agent::{Agent, Species, generate_agents_with_rng};
use cell::Cell;
use model::WolfSheepGrass;
use quacksim::{Grid, Position, Simulation, World};
use rand::SeedableRng;
use rand::rngs::SmallRng;

#[test]
fn grass_regrows_after_the_configured_countdown() {
    let grid = Grid::new(1, 1, Cell::bare(2)).unwrap();
    let world = World::new(0, (), grid, vec![]);
    let mut simulation = Simulation::new(WolfSheepGrass::new(config()), world);

    simulation.tick().unwrap();
    assert_eq!(simulation.world().grid().cells(), &[Cell::bare(1)]);

    simulation.tick().unwrap();
    assert_eq!(simulation.world().grid().cells(), &[Cell::grown()]);
}

#[test]
fn zero_countdown_grass_grows_on_the_next_cell_update() {
    let grid = Grid::new(1, 1, Cell::bare(0)).unwrap();
    let world = World::new(0, (), grid, vec![]);
    let mut simulation = Simulation::new(WolfSheepGrass::new(config()), world);

    simulation.tick().unwrap();

    assert_eq!(simulation.world().grid().cells(), &[Cell::grown()]);
}

#[test]
fn first_agent_wins_a_destination_conflict() {
    let grid = Grid::new(2, 1, Cell::default()).unwrap();
    let agents = vec![
        Agent::new(Species::Sheep, Position::new(0, 0), 10),
        Agent::new(Species::Sheep, Position::new(0, 0), 10),
    ];
    let world = World::new(1, (), grid, agents);
    let mut simulation = Simulation::new(WolfSheepGrass::new(config()), world);

    let outcome = simulation.tick().unwrap();
    let pos0 = simulation.world().agents()[0].position;
    let pos1 = simulation.world().agents()[1].position;

    assert_eq!(outcome.report().generated_commands(), 2);
    assert_eq!(outcome.report().resolved_commands(), 2);
    assert!(
        (pos0 == Position::new(1, 0) && pos1 == Position::new(0, 0))
            || (pos0 == Position::new(0, 0) && pos1 == Position::new(1, 0))
    );
}

#[test]
fn output_counts_each_species() {
    let grid = Grid::new(1, 1, Cell::default()).unwrap();
    let agents = vec![
        Agent::new(Species::Sheep, Position::new(0, 0), 10),
        Agent::new(Species::Sheep, Position::new(0, 0), 10),
        Agent::new(Species::Wolf, Position::new(0, 0), 20),
    ];
    let world = World::new(0, (), grid, agents);
    let mut simulation = Simulation::new(WolfSheepGrass::new(config()), world);

    let outcome = simulation.tick().unwrap();

    assert_eq!(outcome.output().sheep, 1);
    assert_eq!(outcome.output().wolves, 1);
    assert_eq!(outcome.output().grown_grass, 0);
}

#[test]
fn reproduction_requires_food() {
    let grid = Grid::new(1, 1, Cell::grown()).unwrap();
    let agents = vec![Agent::new(Species::Wolf, Position::new(0, 0), 20)];
    let world = World::new(0, (), grid, agents);
    let mut simulation = Simulation::new(
        WolfSheepGrass::new(model::ModelConfig {
            wolves: model::SpeciesConfig {
                reproduction_probability: 1.0,
                ..config().wolves
            },
            ..config()
        }),
        world,
    );

    simulation.tick().unwrap();

    assert_eq!(simulation.world().agents().len(), 1);
}

#[test]
fn balanced_population_does_not_collapse_immediately() {
    let grid = Grid::new(64, 64, Cell::grown()).unwrap();
    let mut random = SmallRng::seed_from_u64(42);
    let mut agents =
        generate_agents_with_rng(300, Species::Sheep, 12, 64, 64, &mut random).unwrap();
    agents.extend(generate_agents_with_rng(40, Species::Wolf, 24, 64, 64, &mut random).unwrap());
    let world = World::new(0, (), grid, agents);
    let mut simulation = Simulation::new(
        WolfSheepGrass::new(model::ModelConfig {
            seed: 7,
            sheep: model::SpeciesConfig {
                food_energy: 8,
                reproduction_probability: 0.02,
            },
            wolves: model::SpeciesConfig {
                food_energy: 20,
                reproduction_probability: 0.01,
            },
            movement_energy: 1,
            grass_regrowth_ticks: 8,
        }),
        world,
    );

    for _ in 0..1_000 {
        simulation.tick().unwrap();
    }

    assert!(
        simulation
            .world()
            .agents()
            .iter()
            .any(|agent| agent.species == Species::Sheep)
    );
    assert!(
        simulation
            .world()
            .agents()
            .iter()
            .any(|agent| agent.species == Species::Wolf)
    );
}

fn config() -> model::ModelConfig {
    model::ModelConfig {
        seed: 42,
        sheep: model::SpeciesConfig {
            food_energy: 4,
            reproduction_probability: 0.0,
        },
        wolves: model::SpeciesConfig {
            food_energy: 20,
            reproduction_probability: 0.0,
        },
        movement_energy: 1,
        grass_regrowth_ticks: 3,
    }
}
