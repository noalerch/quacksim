use std::sync::atomic::{AtomicUsize, Ordering};

use quacksim::{
    CommandEnvelope, Grid, Simulation, TickContext, TickError, TickModel, World, WorldView,
};

#[derive(Default)]
struct EmptyModel {
    global: AtomicUsize,
    cells: AtomicUsize,
    agents: AtomicUsize,
    resolver: AtomicUsize,
    output: AtomicUsize,
}

impl TickModel for EmptyModel {
    type Agent = ();
    type Cell = ();
    type Command = ();
    type Global = usize;
    type Output = usize;

    fn update_global(
        &self,
        _context: TickContext,
        global: &mut Self::Global,
        _grid: &Grid<Self::Cell>,
        _agents: &[Self::Agent],
    ) {
        self.global.fetch_add(1, Ordering::Relaxed);
        *global += 1;
    }

    fn update_cell(
        &self,
        _context: TickContext,
        _position: quacksim::Position,
        _cell: &Self::Cell,
        _world: WorldView<'_, Self::Global, Self::Cell, Self::Agent>,
    ) -> Self::Cell {
        self.cells.fetch_add(1, Ordering::Relaxed);
    }

    fn run_agent(
        &self,
        _context: TickContext,
        _agent_index: usize,
        _agent: &Self::Agent,
        _world: WorldView<'_, Self::Global, Self::Cell, Self::Agent>,
        _commands: &mut quacksim::CommandSink<'_, Self::Command>,
    ) {
        self.agents.fetch_add(1, Ordering::Relaxed);
    }

    fn resolve_commands(
        &self,
        _context: TickContext,
        _world: WorldView<'_, Self::Global, Self::Cell, Self::Agent>,
        commands: Vec<CommandEnvelope<Self::Command>>,
    ) -> Vec<CommandEnvelope<Self::Command>> {
        self.resolver.fetch_add(1, Ordering::Relaxed);
        commands
    }

    fn apply_command(
        &self,
        _context: TickContext,
        _world: &mut World<Self::Global, Self::Cell, Self::Agent>,
        _command: CommandEnvelope<Self::Command>,
    ) {
        unreachable!("the empty model does not create commands");
    }

    fn write_output(
        &self,
        _context: TickContext,
        world: WorldView<'_, Self::Global, Self::Cell, Self::Agent>,
    ) -> Self::Output {
        self.output.fetch_add(1, Ordering::Relaxed);
        *world.global()
    }
}

#[test]
fn empty_world_runs_non_parallel_stages_once() {
    let model = EmptyModel::default();
    let world = World::new(0, 0, Grid::new(0, 0, ()).unwrap(), vec![]);
    let mut simulation = Simulation::new(model, world);

    let outcome = simulation.tick().unwrap();

    assert_eq!(*outcome.output(), 1);
    assert_eq!(simulation.model().global.load(Ordering::Relaxed), 1);
    assert_eq!(simulation.model().cells.load(Ordering::Relaxed), 0);
    assert_eq!(simulation.model().agents.load(Ordering::Relaxed), 0);
    assert_eq!(simulation.model().resolver.load(Ordering::Relaxed), 1);
    assert_eq!(simulation.model().output.load(Ordering::Relaxed), 1);
}

#[test]
fn tick_overflow_fails_before_any_stage_changes_state() {
    let model = EmptyModel::default();
    let world = World::with_tick(u64::MAX, 0, 7, Grid::new(0, 0, ()).unwrap(), vec![]);
    let mut simulation = Simulation::new(model, world);

    let error = simulation.tick().unwrap_err();

    assert_eq!(error, TickError::TickOverflow);
    assert_eq!(simulation.world().tick(), u64::MAX);
    assert_eq!(*simulation.world().global(), 7);
    assert_eq!(simulation.model().global.load(Ordering::Relaxed), 0);
}
