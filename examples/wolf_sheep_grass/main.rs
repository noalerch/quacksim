mod agent;
mod cell;
mod model;
mod stats;

use agent::{Species, generate_agents_with_rng};
use cell::Cell;
use model::{ModelConfig, SpeciesConfig, WolfSheepGrass};
use quacksim::{Grid, Simulation, World};
use rand::SeedableRng;
use rand::rngs::SmallRng;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let grid = Grid::new(64, 64, Cell::grown())?;
    let mut random = SmallRng::seed_from_u64(42);
    let mut agents = generate_agents_with_rng(300, Species::Sheep, 12, 64, 64, &mut random)?;
    agents.extend(generate_agents_with_rng(
        40,
        Species::Wolf,
        24,
        64,
        64,
        &mut random,
    )?);
    let world = World::new(0, (), grid, agents);
    let model = WolfSheepGrass::new(ModelConfig {
        seed: 7,
        sheep: SpeciesConfig {
            food_energy: 8,
            reproduction_probability: 0.02,
        },
        wolves: SpeciesConfig {
            food_energy: 20,
            reproduction_probability: 0.01,
        },
        movement_energy: 1,
        grass_regrowth_ticks: 8,
    });
    let mut simulation = Simulation::new(model, world);

    for i in 0..10000 {
        let outcome = simulation.tick()?;
        if i % 50 == 0 {
            println!(
                "tick {}: {} sheep, {} wolves, {} grown grass",
                outcome.report().tick(),
                outcome.output().sheep,
                outcome.output().wolves,
                outcome.output().grown_grass,
            );
        }
    }

    Ok(())
}
