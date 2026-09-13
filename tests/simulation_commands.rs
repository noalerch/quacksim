use quacksim::{
    CommandEnvelope, CommandSink, Grid, Simulation, TickContext, TickModel, World, WorldView,
};
use rayon::ThreadPoolBuilder;

#[derive(Clone, Copy)]
struct CommandModel;

impl TickModel for CommandModel {
    type Agent = u32;
    type Cell = ();
    type Command = u32;
    type Global = Vec<u32>;
    type Output = Vec<u32>;

    fn run_agent(
        &self,
        _context: TickContext,
        _agent_index: usize,
        agent: &Self::Agent,
        _world: WorldView<'_, Self::Global, Self::Cell, Self::Agent>,
        commands: &mut CommandSink<'_, Self::Command>,
    ) {
        commands.push(agent * 10);
        commands.push(agent * 10 + 1);
    }

    fn apply_command(
        &self,
        _context: TickContext,
        world: &mut World<Self::Global, Self::Cell, Self::Agent>,
        command: CommandEnvelope<Self::Command>,
    ) {
        world.global_mut().push(command.into_command());
    }

    fn write_output(
        &self,
        _context: TickContext,
        world: WorldView<'_, Self::Global, Self::Cell, Self::Agent>,
    ) -> Self::Output {
        world.global().clone()
    }
}

pub fn run_with_threads(thread_count: usize) -> Vec<u32> {
    let pool = ThreadPoolBuilder::new()
        .num_threads(thread_count)
        .build()
        .unwrap();

    pool.install(|| {
        let agents = (0..128).collect();
        let world = World::new(0, Vec::new(), Grid::new(1, 1, ()).unwrap(), agents);
        Simulation::new(CommandModel, world)
            .tick()
            .unwrap()
            .into_output()
    })
}

#[test]
fn command_order_is_independent_of_worker_count() {
    let serial = run_with_threads(1);
    let parallel = run_with_threads(4);

    assert_eq!(parallel, serial);
    assert_eq!(&parallel[..6], &[0, 1, 10, 11, 20, 21]);
}

#[derive(Clone, Copy)]
struct ConflictModel;

impl TickModel for ConflictModel {
    type Agent = u32;
    type Cell = ();
    type Command = u32;
    type Global = Vec<u32>;
    type Output = Vec<u32>;

    fn run_agent(
        &self,
        _context: TickContext,
        _agent_index: usize,
        agent: &Self::Agent,
        _world: WorldView<'_, Self::Global, Self::Cell, Self::Agent>,
        commands: &mut CommandSink<'_, Self::Command>,
    ) {
        commands.push(*agent);
    }

    fn resolve_commands(
        &self,
        _context: TickContext,
        _world: WorldView<'_, Self::Global, Self::Cell, Self::Agent>,
        commands: Vec<CommandEnvelope<Self::Command>>,
    ) -> Vec<CommandEnvelope<Self::Command>> {
        commands.into_iter().take(1).collect()
    }

    fn apply_command(
        &self,
        _context: TickContext,
        world: &mut World<Self::Global, Self::Cell, Self::Agent>,
        command: CommandEnvelope<Self::Command>,
    ) {
        world.global_mut().push(command.into_command());
    }

    fn write_output(
        &self,
        _context: TickContext,
        world: WorldView<'_, Self::Global, Self::Cell, Self::Agent>,
    ) -> Self::Output {
        world.global().clone()
    }
}

#[test]
fn resolver_can_remove_conflicting_commands_before_commit() {
    let world = World::new(0, Vec::new(), Grid::new(1, 1, ()).unwrap(), vec![3, 4]);
    let mut simulation = Simulation::new(ConflictModel, world);

    let outcome = simulation.tick().unwrap();

    assert_eq!(outcome.output(), &[3]);
    assert_eq!(outcome.report().generated_commands(), 2);
    assert_eq!(outcome.report().resolved_commands(), 1);
}

#[derive(Clone, Copy)]
struct MetadataModel;

impl TickModel for MetadataModel {
    type Agent = u32;
    type Cell = ();
    type Command = u32;
    type Global = Vec<(usize, usize, u32)>;
    type Output = Vec<(usize, usize, u32)>;

    fn run_agent(
        &self,
        _context: TickContext,
        _agent_index: usize,
        agent: &Self::Agent,
        _world: WorldView<'_, Self::Global, Self::Cell, Self::Agent>,
        commands: &mut CommandSink<'_, Self::Command>,
    ) {
        assert!(commands.is_empty());
        commands.push(*agent);
        commands.push(*agent + 1);
        assert_eq!(commands.len(), 2);
    }

    fn apply_command(
        &self,
        _context: TickContext,
        world: &mut World<Self::Global, Self::Cell, Self::Agent>,
        command: CommandEnvelope<Self::Command>,
    ) {
        let metadata = (
            command.agent_index(),
            command.sequence(),
            *command.command(),
        );
        world.global_mut().push(metadata);
    }

    fn write_output(
        &self,
        _context: TickContext,
        world: WorldView<'_, Self::Global, Self::Cell, Self::Agent>,
    ) -> Self::Output {
        world.global().clone()
    }
}

#[test]
fn command_envelopes_keep_source_and_sequence_metadata() {
    let world = World::new(0, Vec::new(), Grid::new(1, 1, ()).unwrap(), vec![10, 20]);
    let mut simulation = Simulation::new(MetadataModel, world);

    let outcome = simulation.tick().unwrap();

    assert_eq!(
        outcome.output(),
        &[(0, 0, 10), (0, 1, 11), (1, 0, 20), (1, 1, 21)]
    );
}
