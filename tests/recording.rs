use quacksim::{
    CommandEnvelope, Grid, SampledRecorder, Simulation, TickContext, TickModel, TickRecorder,
    World, WorldView, record_tick,
};

#[derive(Clone, Copy)]
struct Model;

impl TickModel for Model {
    type Agent = ();
    type Cell = ();
    type Command = ();
    type Global = ();
    type Output = u64;

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
        _world: WorldView<'_, Self::Global, Self::Cell, Self::Agent>,
    ) -> Self::Output {
        context.tick()
    }
}

#[derive(Debug, Default)]
struct Recorder {
    ticks: Vec<u64>,
}

impl TickRecorder<u64> for Recorder {
    type Error = ();

    fn record(&mut self, report: quacksim::TickReport, output: &u64) -> Result<(), Self::Error> {
        self.ticks.push(report.tick());
        assert_eq!(report.tick(), *output);
        Ok(())
    }
}

fn simulation() -> Simulation<Model> {
    let grid = Grid::new(0, 0, ()).unwrap();
    Simulation::new(Model, World::new(0, (), grid, vec![]))
}

#[test]
fn recorder_receives_successful_tick_output() {
    let mut simulation = simulation();
    let mut recorder = Recorder::default();

    let outcome = simulation.tick().unwrap();
    record_tick::<Model, _>(&mut recorder, &outcome).unwrap();

    assert_eq!(recorder.ticks, vec![1]);
}

#[test]
fn sampled_recorder_records_matching_ticks() {
    let mut simulation = simulation();
    let recorder = Recorder::default();
    let mut recorder = SampledRecorder::new(recorder, 2).unwrap();

    for _ in 0..5 {
        let outcome = simulation.tick().unwrap();
        record_tick::<Model, _>(&mut recorder, &outcome).unwrap();
    }

    assert_eq!(recorder.into_inner().ticks, vec![2, 4]);
}

#[test]
fn sampled_recorder_rejects_zero_interval() {
    assert_eq!(
        SampledRecorder::new(Recorder::default(), 0)
            .unwrap_err()
            .to_string(),
        "sampling interval must be greater than zero"
    );
}
