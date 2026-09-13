mod agent;
mod cell;
mod model;
mod recording;
mod stats;

use std::fs::File;
use std::io::BufWriter;
use std::path::PathBuf;

use agent::{Species, generate_agents_with_rng};
use cell::Cell;
use model::{ModelConfig, SpeciesConfig, WolfSheepGrass};
use quacksim::{Grid, SampledRecorder, Simulation, World, record_tick};
use rand::SeedableRng;
use rand::rngs::SmallRng;

use recording::CsvRecorder;

struct Options {
    metrics_path: Option<PathBuf>,
    sample_every: u64,
}

fn parse_options() -> Result<Options, Box<dyn std::error::Error>> {
    let mut arguments = std::env::args().skip(1);
    let mut metrics_path = None;
    let mut sample_every = 1;

    while let Some(argument) = arguments.next() {
        match argument.as_str() {
            "--metrics" => {
                let path = arguments.next().ok_or("--metrics requires a path")?;
                metrics_path = Some(PathBuf::from(path));
            }
            "--sample-every" => {
                let interval = arguments.next().ok_or("--sample-every requires a number")?;
                sample_every = interval.parse()?;
                if sample_every == 0 {
                    return Err("--sample-every must be greater than zero".into());
                }
            }
            _ => return Err(format!("unknown argument: {argument}").into()),
        }
    }

    Ok(Options {
        metrics_path,
        sample_every,
    })
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let options = parse_options()?;
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
    let mut recorder = options
        .metrics_path
        .map(File::create)
        .transpose()?
        .map(|file| CsvRecorder::new(BufWriter::new(file)))
        .transpose()?
        .take()
        .map(|recorder| SampledRecorder::new(recorder, options.sample_every))
        .transpose()?;

    for i in 0..10000 {
        let outcome = simulation.tick()?;
        if let Some(recorder) = recorder.as_mut() {
            record_tick::<WolfSheepGrass, _>(recorder, &outcome)?;
        }
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

    if let Some(recorder) = recorder {
        recorder.into_inner().finish()?;
    }

    Ok(())
}
