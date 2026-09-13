# Quacksim

Quacksim is a Rust library for parallel, high-performance 2D simulations.

The library provides:

- A flat, row-major grid.
- Parallel cell and agent updates with Rayon.
- Immutable world views during parallel stages.
- Deterministic command collection order for all worker counts.
- Model-defined command conflict resolution.
- Reusable cell buffers between ticks.

Each tick has these stages:

1. Update global state.
2. Update cells in parallel.
3. Run agent behavior in parallel.
4. Resolve commands.
5. Apply commands in sequence.
6. Finalize mutable state.
7. Write output.

`TickContext` identifies the tick in progress. Parallel callbacks receive an
immutable `WorldView`. During cell and agent callbacks, the view reports the
last committed tick. The output callback receives the newly committed world.

Each agent writes intent through its `CommandSink`. Before conflict resolution,
the engine sorts commands by agent index and then by per-agent sequence. A
model must preserve deterministic behavior in its resolver if it needs fully
repeatable results. Resolved commands are applied in sequence.

Models implement `TickModel`. A model must define command application and
output. Global, cell, agent, and conflict-resolution methods have useful
defaults. The engine reuses cell storage between ticks, but model output and
command vectors can allocate according to model needs.

## Example: Wolf-Sheep-Grass
The wolf-sheep-grass example includes random movement, movement energy
costs, sheep grazing, wolf predation, reproduction, death, grass regrowth, and
species counts. Model randomness uses a configured seed and per-agent streams,
so parallel worker scheduling does not change results.

The example provides `generate_agents` for one-call random population creation.
Use `generate_agents_with_rng` with a seeded random generator when setup must be
repeatable. Both functions permit multiple agents at one position.

Run the wolf-sheep-grass example:

```sh
cargo run --example wolf_sheep_grass
```

Run all tests:

```sh
cargo test --all-targets --all-features
```

Record wolf-sheep-grass metrics as CSV:

```sh
cargo run --example wolf_sheep_grass -- \
  --metrics runs/run-001.csv --sample-every 50
```

Plot one or more metric files with Python:

```sh
python3 -m venv .venv
. .venv/bin/activate
python -m pip install -r analysis/requirements.txt
python analysis/plot.py runs/run-001.csv --output graphs/run-001.png
```

Python 3.10 or newer is required. See `analysis/README.md` for details.

The library provides the generic `TickRecorder` trait and `SampledRecorder`.
Recorders receive output only after a tick commits successfully. Model examples
define their own metric formats.
