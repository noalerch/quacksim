use std::error::Error;
use std::fmt::{self, Display, Formatter};

use crate::{CommandEnvelope, CommandSink, Grid, Position, World, WorldView};

/// Identifies the tick that is in progress.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct TickContext {
    tick: u64,
}

impl TickContext {
    /// Creates context for `tick`.
    #[must_use]
    pub const fn new(tick: u64) -> Self {
        Self { tick }
    }

    /// Returns the tick that is in progress.
    #[must_use]
    pub const fn tick(self) -> u64 {
        self.tick
    }
}

/// Defines all model-specific stages in a simulation tick.
///
/// The engine calls the methods in declaration order. It runs cell and agent
/// calls in parallel. All other calls run in sequence.
pub trait TickModel: Sync {
    /// State shared by the complete world.
    type Global: Sync;
    /// State stored in each grid position.
    type Cell: Clone + Send + Sync;
    /// State stored for each agent.
    type Agent: Sync;
    /// An intent that an agent creates during parallel behavior.
    type Command: Send;
    /// Data produced after a committed tick.
    type Output;

    /// Updates global state before parallel work starts.
    ///
    /// The grid and agent slice contain the state from the previous tick.
    fn update_global(
        &self,
        _context: TickContext,
        _global: &mut Self::Global,
        _grid: &Grid<Self::Cell>,
        _agents: &[Self::Agent],
    ) {
    }

    /// Produces the next state for one cell.
    ///
    /// The default implementation clones the current cell. `world` contains
    /// the updated global state and the previous cell state.
    fn update_cell(
        &self,
        _context: TickContext,
        _position: Position,
        cell: &Self::Cell,
        _world: WorldView<'_, Self::Global, Self::Cell, Self::Agent>,
    ) -> Self::Cell {
        cell.clone()
    }

    /// Runs behavior for one agent and records its commands.
    ///
    /// `world` contains the updated global and cell state. Agent storage is
    /// unchanged until command application starts.
    fn run_agent(
        &self,
        _context: TickContext,
        _agent_index: usize,
        _agent: &Self::Agent,
        _world: WorldView<'_, Self::Global, Self::Cell, Self::Agent>,
        _commands: &mut CommandSink<'_, Self::Command>,
    ) {
    }

    /// Resolves command conflicts before state mutation.
    ///
    /// `commands` is in deterministic agent and sequence order. The default
    /// implementation keeps all commands in that order.
    fn resolve_commands(
        &self,
        _context: TickContext,
        _world: WorldView<'_, Self::Global, Self::Cell, Self::Agent>,
        commands: Vec<CommandEnvelope<Self::Command>>,
    ) -> Vec<CommandEnvelope<Self::Command>> {
        commands
    }

    /// Applies one resolved command to mutable world state.
    ///
    /// The engine calls this method in the order returned by
    /// [`Self::resolve_commands`].
    fn apply_command(
        &self,
        context: TickContext,
        world: &mut World<Self::Global, Self::Cell, Self::Agent>,
        command: CommandEnvelope<Self::Command>,
    );

    /// Finalizes mutable state after all resolved commands are applied.
    ///
    /// The default implementation makes no changes. Models can use this stage
    /// for cleanup that would invalidate command source indexes.
    fn finish_tick(
        &self,
        _context: TickContext,
        _world: &mut World<Self::Global, Self::Cell, Self::Agent>,
    ) {
    }

    /// Produces output after all state changes are committed.
    ///
    /// Both `context` and `world` identify the newly committed tick.
    fn write_output(
        &self,
        context: TickContext,
        world: WorldView<'_, Self::Global, Self::Cell, Self::Agent>,
    ) -> Self::Output;
}

/// Counts work completed during one tick.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct TickReport {
    tick: u64,
    generated_commands: usize,
    resolved_commands: usize,
}

impl TickReport {
    #[must_use]
    pub(crate) const fn new(
        tick: u64,
        generated_commands: usize,
        resolved_commands: usize,
    ) -> Self {
        Self {
            tick,
            generated_commands,
            resolved_commands,
        }
    }

    /// Returns the committed tick.
    #[must_use]
    pub const fn tick(self) -> u64 {
        self.tick
    }

    /// Returns the number of commands made by agent behavior.
    #[must_use]
    pub const fn generated_commands(self) -> usize {
        self.generated_commands
    }

    /// Returns the number of commands returned by conflict resolution.
    #[must_use]
    pub const fn resolved_commands(self) -> usize {
        self.resolved_commands
    }
}

/// Contains a tick report and model output.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TickOutcome<O> {
    report: TickReport,
    output: O,
}

impl<O> TickOutcome<O> {
    #[must_use]
    pub(crate) const fn new(report: TickReport, output: O) -> Self {
        Self { report, output }
    }

    /// Returns the tick report.
    #[must_use]
    pub const fn report(&self) -> TickReport {
        self.report
    }

    /// Returns the model output.
    #[must_use]
    pub const fn output(&self) -> &O {
        &self.output
    }

    /// Removes the outcome and returns the model output.
    #[must_use]
    pub fn into_output(self) -> O {
        self.output
    }

    /// Removes the outcome and returns its report and output.
    #[must_use]
    pub fn into_parts(self) -> (TickReport, O) {
        (self.report, self.output)
    }
}

/// An error that prevents a tick from starting.
#[non_exhaustive]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum TickError {
    /// The next tick would exceed [`u64::MAX`].
    TickOverflow,
}

impl Display for TickError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Self::TickOverflow => formatter.write_str("tick counter exceeds u64"),
        }
    }
}

impl Error for TickError {}
