use std::sync::Mutex;

use quacksim::{
    CommandEnvelope, CommandSink, Grid, Position, Simulation, TickContext, TickModel, World,
    WorldView,
};

#[derive(Default)]
struct TimingModel {
    observations: Mutex<Vec<(&'static str, u64, u64)>>,
}

impl TickModel for TimingModel {
    type Agent = ();
    type Cell = ();
    type Command = ();
    type Global = ();
    type Output = (u64, u64);

    fn update_cell(
        &self,
        context: TickContext,
        _position: Position,
        _cell: &Self::Cell,
        world: WorldView<'_, Self::Global, Self::Cell, Self::Agent>,
    ) -> Self::Cell {
        self.observations
            .lock()
            .unwrap()
            .push(("cell", context.tick(), world.tick()));
    }

    fn run_agent(
        &self,
        context: TickContext,
        _agent_index: usize,
        _agent: &Self::Agent,
        world: WorldView<'_, Self::Global, Self::Cell, Self::Agent>,
        _commands: &mut CommandSink<'_, Self::Command>,
    ) {
        self.observations
            .lock()
            .unwrap()
            .push(("agent", context.tick(), world.tick()));
    }

    fn apply_command(
        &self,
        _context: TickContext,
        _world: &mut World<Self::Global, Self::Cell, Self::Agent>,
        _command: CommandEnvelope<Self::Command>,
    ) {
    }

    fn write_output(
        &self,
        context: TickContext,
        world: WorldView<'_, Self::Global, Self::Cell, Self::Agent>,
    ) -> Self::Output {
        (context.tick(), world.tick())
    }
}

#[test]
fn stage_views_use_the_previous_tick_until_commit() {
    let world = World::new(0, (), Grid::new(1, 1, ()).unwrap(), vec![()]);
    let mut simulation = Simulation::new(TimingModel::default(), world);

    let outcome = simulation.tick().unwrap();

    assert_eq!(
        simulation.model().observations.lock().unwrap().as_slice(),
        &[("cell", 1, 0), ("agent", 1, 0)]
    );
    assert_eq!(*outcome.output(), (1, 1));
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct PositionModel;

impl TickModel for PositionModel {
    type Agent = ();
    type Cell = usize;
    type Command = ();
    type Global = ();
    type Output = Vec<usize>;

    fn update_cell(
        &self,
        _context: TickContext,
        position: Position,
        _cell: &Self::Cell,
        _world: WorldView<'_, Self::Global, Self::Cell, Self::Agent>,
    ) -> Self::Cell {
        position.y * 10 + position.x
    }

    fn apply_command(
        &self,
        _context: TickContext,
        _world: &mut World<Self::Global, Self::Cell, Self::Agent>,
        _command: CommandEnvelope<Self::Command>,
    ) {
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
fn parallel_cell_stage_maps_two_dimensional_positions() {
    let world = World::new(0, (), Grid::new(2, 2, 0).unwrap(), vec![]);
    let mut simulation = Simulation::new(PositionModel, world);

    let outcome = simulation.tick().unwrap();

    assert_eq!(outcome.output(), &[0, 1, 10, 11]);
}

#[test]
fn simulation_returns_owned_model_and_world() {
    let world = World::new(0, (), Grid::new(1, 1, 0).unwrap(), vec![]);
    let simulation = Simulation::new(PositionModel, world);

    let (model, world) = simulation.into_parts();

    assert_eq!(model, PositionModel);
    assert_eq!(world.grid().cells(), &[0]);
}
