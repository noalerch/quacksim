use crate::{Grid, Position};

/// Owns all state for one simulation.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct World<G, C, A> {
    tick: u64,
    max_agents_per_cell: usize,
    global: G,
    grid: Grid<C>,
    agents: Vec<A>,
}

impl<G, C, A> World<G, C, A> {
    /// Creates a world at tick zero.
    #[must_use]
    pub const fn new(max_agents_per_cell: usize, global: G, grid: Grid<C>, agents: Vec<A>) -> Self {
        Self::with_tick(0, max_agents_per_cell, global, grid, agents)
    }

    /// Creates a world at a specified tick.
    ///
    /// This constructor supports snapshot restoration and controlled tests.
    #[must_use]
    pub const fn with_tick(
        tick: u64,
        max_agents_per_cell: usize,
        global: G,
        grid: Grid<C>,
        agents: Vec<A>,
    ) -> Self {
        Self {
            tick,
            max_agents_per_cell,
            global,
            grid,
            agents,
        }
    }

    /// Returns the last committed tick.
    #[must_use]
    pub const fn tick(&self) -> u64 {
        self.tick
    }

    /// Returns the global state.
    #[must_use]
    pub const fn global(&self) -> &G {
        &self.global
    }

    /// Returns mutable global state for sequential setup or command commits.
    #[must_use]
    pub const fn global_mut(&mut self) -> &mut G {
        &mut self.global
    }

    /// Returns the cell grid.
    #[must_use]
    pub const fn grid(&self) -> &Grid<C> {
        &self.grid
    }

    /// Returns the mutable cell grid for sequential setup or command commits.
    #[must_use]
    pub const fn grid_mut(&mut self) -> &mut Grid<C> {
        &mut self.grid
    }

    /// Returns the maximum amount of agents per cell.
    ///
    /// A value of zero means there is no limit.
    #[must_use]
    pub const fn max_agents_per_cell(&self) -> usize {
        self.max_agents_per_cell
    }

    /// Returns all agents in stable index order.
    #[must_use]
    pub fn agents(&self) -> &[A] {
        &self.agents
    }

    /// Returns mutable agent storage for sequential setup or command commits.
    ///
    /// A model can add or remove agents during command application. Commands
    /// that use agent indexes must account for such structural changes.
    #[must_use]
    pub fn agents_mut(&mut self) -> &mut Vec<A> {
        &mut self.agents
    }

    /// Creates an immutable view of the current state.
    #[must_use]
    pub fn view(&self) -> WorldView<'_, G, C, A> {
        WorldView {
            tick: self.tick,
            max_agents_per_cell: self.max_agents_per_cell,
            global: &self.global,
            grid: &self.grid,
            agents: &self.agents,
        }
    }

    pub(crate) const fn set_tick(&mut self, tick: u64) {
        self.tick = tick;
    }

    pub(crate) fn global_stage_parts(&mut self) -> (&mut G, &Grid<C>, &[A]) {
        (&mut self.global, &self.grid, &self.agents)
    }
}

/// An immutable view of world state at one stage boundary.
///
/// Cell and agent callbacks receive a copy of this view. During those parallel
/// stages, [`Self::tick`] is the last committed tick and [`crate::TickContext`]
/// identifies the tick in progress.
#[derive(Debug)]
pub struct WorldView<'a, G, C, A> {
    tick: u64,
    max_agents_per_cell: usize,
    global: &'a G,
    grid: &'a Grid<C>,
    agents: &'a [A],
}

impl<G, C, A> Copy for WorldView<'_, G, C, A> {}

impl<G, C, A> Clone for WorldView<'_, G, C, A> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<'a, G, C, A> WorldView<'a, G, C, A> {
    /// Returns the last committed tick for this view.
    #[must_use]
    pub const fn tick(&self) -> u64 {
        self.tick
    }

    /// Returns the maximum amount of agents per cell for this view.
    ///
    /// A value of zero means there is no limit.
    #[must_use]
    pub const fn max_agents_per_cell(&self) -> usize {
        self.max_agents_per_cell
    }

    /// Returns the global state for this view.
    #[must_use]
    pub const fn global(&self) -> &'a G {
        self.global
    }

    /// Returns the grid for this view.
    #[must_use]
    pub const fn grid(&self) -> &'a Grid<C> {
        self.grid
    }

    /// Returns all agents for this view.
    #[must_use]
    pub const fn agents(&self) -> &'a [A] {
        self.agents
    }

    /// Returns the cell at `position` for this view.
    #[must_use]
    pub fn cell(&self, position: Position) -> Option<&'a C> {
        self.grid.get(position)
    }
}
