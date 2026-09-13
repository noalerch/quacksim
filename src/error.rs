use std::error::Error;
use std::fmt::{self, Display, Formatter};

/// An error that prevents grid construction.
#[non_exhaustive]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum GridError {
    /// The product of the grid dimensions exceeds [`usize::MAX`].
    CapacityOverflow,
    /// The cell count does not equal the product of the dimensions.
    CellCountMismatch {
        /// The required cell count.
        expected: usize,
        /// The supplied cell count.
        actual: usize,
    },
}

impl Display for GridError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Self::CapacityOverflow => formatter.write_str("grid capacity exceeds usize"),
            Self::CellCountMismatch { expected, actual } => write!(
                formatter,
                "grid needs {expected} cells but received {actual} cells"
            ),
        }
    }
}

impl Error for GridError {}
