use quacksim::{CommandEnvelope, CommandSink, Position, TickContext, TickModel, World, WorldView};
use rand::rngs::SmallRng;
use rand::{RngExt, SeedableRng};

use crate::agent::{Agent, AgentCommand, FoodClaim, Species};
use crate::cell::{Cell, Grass};
use crate::stats::Statistics;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct SpeciesConfig {
    pub food_energy: u32,
    pub reproduction_probability: f64,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ModelConfig {
    pub seed: u64,
    pub sheep: SpeciesConfig,
    pub wolves: SpeciesConfig,
    pub movement_energy: u32,
    pub grass_regrowth_ticks: u32,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct WolfSheepGrass {
    config: ModelConfig,
}

impl WolfSheepGrass {
    #[must_use]
    pub const fn new(config: ModelConfig) -> Self {
        Self { config }
    }

    fn species_config(&self, species: Species) -> SpeciesConfig {
        match species {
            Species::Sheep => self.config.sheep,
            Species::Wolf => self.config.wolves,
        }
    }

    fn random_for(&self, tick: u64, agent_index: usize) -> SmallRng {
        let agent = u64::try_from(agent_index).unwrap_or(u64::MAX);
        let seed = self.config.seed
            ^ tick.wrapping_mul(0x9e37_79b9_7f4a_7c15)
            ^ agent.wrapping_mul(0xbf58_476d_1ce4_e5b9);
        SmallRng::seed_from_u64(seed)
    }

    fn destination<R: RngExt + ?Sized>(
        position: Position,
        width: usize,
        height: usize,
        random: &mut R,
    ) -> Position {
        let x = position.x % width;
        let y = position.y % height;
        match random.random_range(0..4) {
            0 => Position::new(if x + 1 == width { 0 } else { x + 1 }, y),
            1 => Position::new(if x == 0 { width - 1 } else { x - 1 }, y),
            2 => Position::new(x, if y + 1 == height { 0 } else { y + 1 }),
            _ => Position::new(x, if y == 0 { height - 1 } else { y - 1 }),
        }
    }

    fn claim_grass(
        world: WorldView<'_, (), Cell, Agent>,
        position: Position,
        claimed: &mut [bool],
    ) -> Option<FoodClaim> {
        let index = position.y * world.grid().width() + position.x;
        let available = world
            .grid()
            .get(position)
            .is_some_and(|cell| cell.grass == Grass::Grown);
        if available && !claimed[index] {
            claimed[index] = true;
            Some(FoodClaim::Grass { position })
        } else {
            None
        }
    }

    fn claim_sheep(
        &self,
        world: WorldView<'_, (), Cell, Agent>,
        position: Position,
        destinations: &[Position],
        claimed: &mut [bool],
    ) -> Option<FoodClaim> {
        let prey_index = world
            .agents()
            .iter()
            .enumerate()
            .position(|(index, agent)| {
                !claimed[index]
                    && agent.energy > self.config.movement_energy
                    && agent.species == Species::Sheep
                    && destinations[index] == position
            })?;
        claimed[prey_index] = true;
        Some(FoodClaim::Sheep {
            agent_index: prey_index,
        })
    }
}

impl TickModel for WolfSheepGrass {
    type Agent = Agent;
    type Cell = Cell;
    type Command = AgentCommand;
    type Global = ();
    type Output = Statistics;

    fn update_cell(
        &self,
        _context: TickContext,
        _position: Position,
        cell: &Self::Cell,
        _world: WorldView<'_, Self::Global, Self::Cell, Self::Agent>,
    ) -> Self::Cell {
        match cell.grass {
            Grass::Bare {
                ticks_until_grown: 0 | 1,
            } => Cell::grown(),
            Grass::Bare { ticks_until_grown } => Cell::bare(ticks_until_grown - 1),
            Grass::Grown => *cell,
        }
    }

    fn run_agent(
        &self,
        context: TickContext,
        agent_index: usize,
        agent: &Self::Agent,
        world: WorldView<'_, Self::Global, Self::Cell, Self::Agent>,
        commands: &mut CommandSink<'_, Self::Command>,
    ) {
        let width = world.grid().width();
        let height = world.grid().height();
        if width == 0 || height == 0 {
            return;
        }

        let mut random = self.random_for(context.tick(), agent_index);
        let destination = Self::destination(agent.position, width, height, &mut random);
        let reproduction_probability = self.species_config(agent.species).reproduction_probability;
        let reproduce = reproduction_probability >= 1.0
            || (reproduction_probability > 0.0
                && reproduction_probability < 1.0
                && random.random_bool(reproduction_probability));
        commands.push(AgentCommand::Act {
            destination,
            reproduce,
            food: None,
        });
    }

    fn resolve_commands(
        &self,
        _context: TickContext,
        world: WorldView<'_, Self::Global, Self::Cell, Self::Agent>,
        mut commands: Vec<CommandEnvelope<Self::Command>>,
    ) -> Vec<CommandEnvelope<Self::Command>> {
        let max_agents = world.max_agents_per_cell();
        let mut actual_destinations: Vec<_> =
            world.agents().iter().map(|agent| agent.position).collect();
        let mut occupancy = vec![0usize; world.grid().len()];

        for command in &mut commands {
            let agent_index = command.agent_index();
            let agent = &world.agents()[agent_index];
            let AgentCommand::Act { destination, .. } = command.command_mut();
            let dest_index = destination.y * world.grid().width() + destination.x;

            let allowed = if max_agents > 0 {
                if occupancy[dest_index] < max_agents {
                    occupancy[dest_index] += 1;
                    true
                } else {
                    false
                }
            } else {
                true
            };

            if allowed {
                actual_destinations[agent_index] = *destination;
            } else {
                *destination = agent.position;
                actual_destinations[agent_index] = agent.position;
                let stay_index = agent.position.y * world.grid().width() + agent.position.x;
                if max_agents > 0 {
                    occupancy[stay_index] += 1;
                }
            }
        }

        let mut grass_claimed = vec![false; world.grid().len()];
        let mut sheep_claimed = vec![false; world.agents().len()];
        for command in &mut commands {
            let agent_index = command.agent_index();
            let agent = &world.agents()[agent_index];
            let AgentCommand::Act {
                destination, food, ..
            } = command.command_mut();
            if agent.energy <= self.config.movement_energy {
                continue;
            }

            *food = match agent.species {
                Species::Sheep => Self::claim_grass(world, *destination, &mut grass_claimed),
                Species::Wolf => self.claim_sheep(
                    world,
                    *destination,
                    &actual_destinations,
                    &mut sheep_claimed,
                ),
            };
        }

        commands
    }

    fn apply_command(
        &self,
        _context: TickContext,
        world: &mut World<Self::Global, Self::Cell, Self::Agent>,
        command: CommandEnvelope<Self::Command>,
    ) {
        let agent_index = command.agent_index();
        let AgentCommand::Act {
            destination,
            reproduce,
            food,
        } = command.into_command();
        let fed = food.is_some();

        {
            let Some(agent) = world.agents_mut().get_mut(agent_index) else {
                return;
            };
            agent.position = destination;
            agent.energy = agent.energy.saturating_sub(self.config.movement_energy);
            if agent.energy == 0 {
                return;
            }
        }

        match food {
            Some(FoodClaim::Grass { position }) => {
                if let Some(cell) = world.grid_mut().get_mut(position) {
                    cell.grass = Grass::Bare {
                        ticks_until_grown: self.config.grass_regrowth_ticks,
                    };
                }
                if let Some(agent) = world.agents_mut().get_mut(agent_index) {
                    agent.energy = agent.energy.saturating_add(self.config.sheep.food_energy);
                }
            }
            Some(FoodClaim::Sheep {
                agent_index: prey_index,
            }) => {
                if let Some(prey) = world.agents_mut().get_mut(prey_index) {
                    prey.energy = 0;
                }
                if let Some(agent) = world.agents_mut().get_mut(agent_index) {
                    agent.energy = agent.energy.saturating_add(self.config.wolves.food_energy);
                }
            }
            None => {}
        }

        if let Some(agent) = world.agents_mut().get_mut(agent_index) {
            agent.reproduce = reproduce && fed;
        }
    }

    fn finish_tick(
        &self,
        _context: TickContext,
        world: &mut World<Self::Global, Self::Cell, Self::Agent>,
    ) {
        world.agents_mut().retain(|agent| agent.energy > 0);
        let parent_count = world.agents().len();
        for parent_index in 0..parent_count {
            let offspring = {
                let parent = &mut world.agents_mut()[parent_index];
                let reproduce = std::mem::take(&mut parent.reproduce);
                if reproduce && parent.energy > 1 {
                    let offspring_energy = parent.energy / 2;
                    parent.energy -= offspring_energy;
                    Some(Agent::new(
                        parent.species,
                        parent.position,
                        offspring_energy,
                    ))
                } else {
                    None
                }
            };
            if let Some(offspring) = offspring {
                world.agents_mut().push(offspring);
            }
        }
    }

    fn write_output(
        &self,
        _context: TickContext,
        world: WorldView<'_, Self::Global, Self::Cell, Self::Agent>,
    ) -> Self::Output {
        let mut statistics =
            world
                .agents()
                .iter()
                .fold(Statistics::default(), |mut statistics, agent| {
                    match agent.species {
                        Species::Sheep => statistics.sheep += 1,
                        Species::Wolf => statistics.wolves += 1,
                    }
                    statistics
                });
        statistics.grown_grass = world
            .grid()
            .cells()
            .iter()
            .filter(|cell| cell.grass == Grass::Grown)
            .count();
        statistics
    }
}
