use rayon::prelude::*;

use crate::{
    CommandEnvelope, CommandSink, Position, TickContext, TickError, TickModel, TickOutcome,
    TickReport, World,
};

/// The model and world returned when a [`Simulation`] is removed.
pub type SimulationParts<M> = (
    M,
    World<<M as TickModel>::Global, <M as TickModel>::Cell, <M as TickModel>::Agent>,
);

/// runs a TickModel against owned world state
#[must_use = "a simulation does no work until tick is called"]
pub struct Simulation<M>
where
    M: TickModel,
{
    model: M,
    world: World<M::Global, M::Cell, M::Agent>,
    next_cells: Vec<M::Cell>,
}

impl<M> Simulation<M>
where
    M: TickModel,
{
    /// Creates a simulation from a model and its initial world.
    pub fn new(model: M, world: World<M::Global, M::Cell, M::Agent>) -> Self {
        let next_cells = Vec::with_capacity(world.grid().len());
        Self {
            model,
            world,
            next_cells,
        }
    }

    /// Returns the model.
    #[must_use]
    pub const fn model(&self) -> &M {
        &self.model
    }

    /// Returns the world.
    #[must_use]
    pub const fn world(&self) -> &World<M::Global, M::Cell, M::Agent> {
        &self.world
    }

    /// Runs and commits one complete tick.
    ///
    /// # Errors
    ///
    /// Returns [`TickError::TickOverflow`] before any stage runs when the tick
    /// counter is [`u64::MAX`].
    pub fn tick(&mut self) -> Result<TickOutcome<M::Output>, TickError> {
        let next_tick = self
            .world
            .tick()
            .checked_add(1)
            .ok_or(TickError::TickOverflow)?;
        let context = TickContext::new(next_tick);

        self.update_global(context);
        self.update_cells(context);
        let commands = self.run_agents(context);
        let generated_commands = commands.len();
        let resolved_commands = self.resolve_commands(context, commands);
        let resolved_command_count = resolved_commands.len();
        self.apply_commands(context, resolved_commands);
        self.model.finish_tick(context, &mut self.world);
        self.world.set_tick(next_tick);

        let output = self.model.write_output(context, self.world.view());
        let report = TickReport::new(next_tick, generated_commands, resolved_command_count);
        Ok(TickOutcome::new(report, output))
    }

    /// Removes the simulation and returns its model and world.
    #[must_use]
    pub fn into_parts(self) -> SimulationParts<M> {
        (self.model, self.world)
    }

    fn update_global(&mut self, context: TickContext) {
        let (global, grid, agents) = self.world.global_stage_parts();
        self.model.update_global(context, global, grid, agents);
    }

    fn update_cells(&mut self, context: TickContext) {
        let model = &self.model;
        let world = self.world.view();
        let width = world.grid().width();

        world
            .grid()
            .cells()
            .par_iter()
            .enumerate()
            .map(|(index, cell)| {
                let position = Position::new(index % width, index / width);
                model.update_cell(context, position, cell, world)
            })
            .collect_into_vec(&mut self.next_cells);

        self.world.grid_mut().swap_cells(&mut self.next_cells);
    }

    fn run_agents(&self, context: TickContext) -> Vec<CommandEnvelope<M::Command>> {
        let model = &self.model;
        let world = self.world.view();
        let mut commands = world
            .agents()
            .par_iter()
            .enumerate()
            .fold(Vec::new, |mut commands, (agent_index, agent)| {
                let mut sink = CommandSink::new(agent_index, &mut commands);
                model.run_agent(context, agent_index, agent, world, &mut sink);
                commands
            })
            .reduce(Vec::new, |mut left, mut right| {
                left.append(&mut right);
                left
            });

        commands.sort_unstable_by_key(|command| (command.agent_index(), command.sequence()));
        commands
    }

    fn resolve_commands(
        &self,
        context: TickContext,
        commands: Vec<CommandEnvelope<M::Command>>,
    ) -> Vec<CommandEnvelope<M::Command>> {
        self.model
            .resolve_commands(context, self.world.view(), commands)
    }

    fn apply_commands(&mut self, context: TickContext, commands: Vec<CommandEnvelope<M::Command>>) {
        for command in commands {
            self.model.apply_command(context, &mut self.world, command);
        }
    }
}
