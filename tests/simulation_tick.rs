use std::sync::Mutex;

use quacksim::{
    CommandEnvelope, CommandSink, Grid, Position, Simulation, TickContext, TickModel, World,
    WorldView,
};

#[derive(Default)]
struct OrderedModel {
    stages: Mutex<Vec<&'static str>>,
}

impl TickModel for OrderedModel {
    type Agent = u32;
    type Cell = u32;
    type Command = u32;
    type Global = Vec<u32>;
    type Output = Vec<&'static str>;

    fn update_global(
        &self,
        _context: TickContext,
        global: &mut Self::Global,
        _grid: &Grid<Self::Cell>,
        _agents: &[Self::Agent],
    ) {
        self.stages.lock().unwrap().push("global");
        global.push(1);
    }

    fn update_cell(
        &self,
        _context: TickContext,
        _position: Position,
        cell: &Self::Cell,
        world: WorldView<'_, Self::Global, Self::Cell, Self::Agent>,
    ) -> Self::Cell {
        self.stages.lock().unwrap().push("cell");
        assert_eq!(world.global(), &[1]);
        cell + 1
    }

    fn run_agent(
        &self,
        _context: TickContext,
        _agent_index: usize,
        agent: &Self::Agent,
        world: WorldView<'_, Self::Global, Self::Cell, Self::Agent>,
        commands: &mut CommandSink<'_, Self::Command>,
    ) {
        self.stages.lock().unwrap().push("agent");
        assert_eq!(world.grid().cells(), &[1]);
        commands.push(*agent);
    }

    fn resolve_commands(
        &self,
        _context: TickContext,
        _world: WorldView<'_, Self::Global, Self::Cell, Self::Agent>,
        commands: Vec<CommandEnvelope<Self::Command>>,
    ) -> Vec<CommandEnvelope<Self::Command>> {
        self.stages.lock().unwrap().push("resolve");
        commands
    }

    fn apply_command(
        &self,
        _context: TickContext,
        world: &mut World<Self::Global, Self::Cell, Self::Agent>,
        command: CommandEnvelope<Self::Command>,
    ) {
        self.stages.lock().unwrap().push("apply");
        world.global_mut().push(command.into_command());
    }

    fn write_output(
        &self,
        _context: TickContext,
        _world: WorldView<'_, Self::Global, Self::Cell, Self::Agent>,
    ) -> Self::Output {
        self.stages.lock().unwrap().push("output");
        self.stages.lock().unwrap().clone()
    }
}

#[test]
fn tick_runs_each_stage_in_order() {
    let world = World::new(0, Vec::new(), Grid::new(1, 1, 0).unwrap(), vec![7]);
    let mut simulation = Simulation::new(OrderedModel::default(), world);

    let outcome = simulation.tick().unwrap();

    assert_eq!(
        outcome.output(),
        &["global", "cell", "agent", "resolve", "apply", "output"]
    );
    assert_eq!(outcome.report().generated_commands(), 1);
    assert_eq!(outcome.report().resolved_commands(), 1);
    assert_eq!(simulation.world().tick(), 1);
    assert_eq!(simulation.world().global(), &[1, 7]);
}

#[derive(Clone, Copy)]
struct SnapshotModel;

impl TickModel for SnapshotModel {
    type Agent = ();
    type Cell = u32;
    type Command = ();
    type Global = ();
    type Output = Vec<u32>;

    fn update_cell(
        &self,
        _context: TickContext,
        position: Position,
        cell: &Self::Cell,
        world: WorldView<'_, Self::Global, Self::Cell, Self::Agent>,
    ) -> Self::Cell {
        assert_eq!(world.grid().cells(), &[1, 2, 3]);
        cell + u32::try_from(position.x).unwrap() + 1
    }

    fn apply_command(
        &self,
        _context: TickContext,
        _world: &mut World<Self::Global, Self::Cell, Self::Agent>,
        _command: CommandEnvelope<Self::Command>,
    ) {
        unreachable!("the snapshot model does not create commands");
    }

    fn write_output(
        &self,
        _context: TickContext,
        world: WorldView<'_, Self::Global, Self::Cell, Self::Agent>,
    ) -> Self::Output {
        world.grid().cells().to_vec()
    }
}

#[test]
fn parallel_cell_updates_read_one_immutable_snapshot() {
    let world = World::new(
        0,
        (),
        Grid::from_cells(3, 1, vec![1, 2, 3]).unwrap(),
        vec![],
    );
    let mut simulation = Simulation::new(SnapshotModel, world);

    let outcome = simulation.tick().unwrap();

    assert_eq!(outcome.output(), &[2, 4, 6]);
}

#[derive(Clone, Copy)]
struct IncrementModel;

impl TickModel for IncrementModel {
    type Agent = ();
    type Cell = u32;
    type Command = ();
    type Global = ();
    type Output = Vec<u32>;

    fn update_cell(
        &self,
        _context: TickContext,
        _position: Position,
        cell: &Self::Cell,
        _world: WorldView<'_, Self::Global, Self::Cell, Self::Agent>,
    ) -> Self::Cell {
        cell + 1
    }

    fn apply_command(
        &self,
        _context: TickContext,
        _world: &mut World<Self::Global, Self::Cell, Self::Agent>,
        _command: CommandEnvelope<Self::Command>,
    ) {
        unreachable!("the increment model does not create commands");
    }

    fn write_output(
        &self,
        _context: TickContext,
        world: WorldView<'_, Self::Global, Self::Cell, Self::Agent>,
    ) -> Self::Output {
        world.grid().cells().to_vec()
    }
}

#[test]
fn repeated_ticks_commit_the_previous_next_state() {
    let world = World::new(0, (), Grid::new(1, 1, 0).unwrap(), vec![]);
    let mut simulation = Simulation::new(IncrementModel, world);

    assert_eq!(simulation.tick().unwrap().output(), &[1]);
    assert_eq!(simulation.tick().unwrap().output(), &[2]);
    assert_eq!(simulation.world().tick(), 2);
}
