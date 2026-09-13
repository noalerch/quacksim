use std::error::Error;
use std::fmt::{self, Display, Formatter};

use quacksim::Position;
use rand::RngExt;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Species {
    Sheep,
    Wolf,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Agent {
    pub species: Species,
    pub position: Position,
    pub energy: u32,
    pub(crate) reproduce: bool,
}

impl Agent {
    #[must_use]
    pub const fn new(species: Species, position: Position, energy: u32) -> Self {
        Self {
            species,
            position,
            energy,
            reproduce: false,
        }
    }
}

#[allow(dead_code)]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AgentGenerationError {
    EmptyGrid,
}

impl Display for AgentGenerationError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Self::EmptyGrid => formatter.write_str("cannot place agents in an empty grid"),
        }
    }
}

impl Error for AgentGenerationError {}

#[allow(dead_code)]
pub fn generate_agents(
    count: usize,
    species: Species,
    energy: u32,
    width: usize,
    height: usize,
) -> Result<Vec<Agent>, AgentGenerationError> {
    generate_agents_with_rng(count, species, energy, width, height, &mut rand::rng())
}

#[allow(dead_code)]
pub fn generate_agents_with_rng<R: RngExt + ?Sized>(
    count: usize,
    species: Species,
    energy: u32,
    width: usize,
    height: usize,
    random: &mut R,
) -> Result<Vec<Agent>, AgentGenerationError> {
    if count > 0 && (width == 0 || height == 0) {
        return Err(AgentGenerationError::EmptyGrid);
    }

    Ok((0..count)
        .map(|_| {
            let position = Position::new(
                random.random_range(0..width),
                random.random_range(0..height),
            );
            Agent::new(species, position, energy)
        })
        .collect())
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum FoodClaim {
    Grass { position: Position },
    Sheep { agent_index: usize },
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AgentCommand {
    Act {
        destination: Position,
        reproduce: bool,
        food: Option<FoodClaim>,
    },
}
