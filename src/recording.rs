use std::error::Error;
use std::fmt::{self, Display, Formatter};

use crate::{TickModel, TickReport};

/// Receives output from successfully committed simulation ticks.
pub trait TickRecorder<O> {
    /// The error returned when recording fails.
    type Error;

    /// Records one committed tick and its model output.
    fn record(&mut self, report: TickReport, output: &O) -> Result<(), Self::Error>;
}

/// Records only ticks whose number is divisible by a fixed interval.
#[derive(Debug)]
pub struct SampledRecorder<R> {
    recorder: R,
    interval: u64,
}

impl<R> SampledRecorder<R> {
    /// Creates a recorder that records every `interval` ticks.
    ///
    /// # Errors
    ///
    /// Returns [`SamplingError::ZeroInterval`] when `interval` is zero.
    pub fn new(recorder: R, interval: u64) -> Result<Self, SamplingError> {
        if interval == 0 {
            return Err(SamplingError::ZeroInterval);
        }

        Ok(Self { recorder, interval })
    }

    /// Returns the sampling interval.
    #[must_use]
    pub const fn interval(&self) -> u64 {
        self.interval
    }

    /// Returns a shared reference to the wrapped recorder.
    #[must_use]
    pub const fn recorder(&self) -> &R {
        &self.recorder
    }

    /// Returns a mutable reference to the wrapped recorder.
    #[must_use]
    pub const fn recorder_mut(&mut self) -> &mut R {
        &mut self.recorder
    }

    /// Removes and returns the wrapped recorder.
    #[must_use]
    pub fn into_inner(self) -> R {
        self.recorder
    }
}

impl<O, R> TickRecorder<O> for SampledRecorder<R>
where
    R: TickRecorder<O>,
{
    type Error = R::Error;

    fn record(&mut self, report: TickReport, output: &O) -> Result<(), Self::Error> {
        if report.tick() % self.interval == 0 {
            self.recorder.record(report, output)?;
        }
        Ok(())
    }
}

/// An error in sampling configuration.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SamplingError {
    /// A sampling interval must be greater than zero.
    ZeroInterval,
}

impl Display for SamplingError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Self::ZeroInterval => {
                formatter.write_str("sampling interval must be greater than zero")
            }
        }
    }
}

impl Error for SamplingError {}

/// Records the output of each successful tick.
///
/// This helper keeps recording outside [`crate::Simulation`] while providing a
/// convenient bound for model-specific runners.
pub fn record_tick<M, R>(
    recorder: &mut R,
    outcome: &crate::TickOutcome<M::Output>,
) -> Result<(), R::Error>
where
    M: TickModel,
    R: TickRecorder<M::Output>,
{
    recorder.record(outcome.report(), outcome.output())
}
