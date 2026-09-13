#![doc = include_str!("../README.md")]
#![forbid(unsafe_code)]
#![warn(missing_docs)]

mod command;
mod error;
mod grid;
mod position;
mod recording;
mod simulation;
mod tick;
mod world;

pub use command::{CommandEnvelope, CommandSink};
pub use error::GridError;
pub use grid::Grid;
pub use position::Position;
pub use recording::{SampledRecorder, SamplingError, TickRecorder, record_tick};
pub use simulation::{Simulation, SimulationParts};
pub use tick::{TickContext, TickError, TickModel, TickOutcome, TickReport};
pub use world::{World, WorldView};
