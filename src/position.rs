/// A zero-based position in a two-dimensional grid.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct Position {
    /// The horizontal coordinate.
    pub x: usize,
    /// The vertical coordinate.
    pub y: usize,
}

impl Position {
    /// Creates a position from horizontal and vertical coordinates.
    #[must_use]
    pub const fn new(x: usize, y: usize) -> Self {
        Self { x, y }
    }
}
