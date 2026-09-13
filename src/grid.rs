use crate::error::GridError;
use crate::position::Position;

/// A dense two-dimensional grid with row-major storage.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Grid<T> {
    width: usize,
    height: usize,
    cells: Vec<T>,
}

impl<T> Grid<T> {
    /// Creates a grid from existing row-major cell storage.
    ///
    /// A zero width or height is valid when `cells` is empty.
    ///
    /// # Errors
    ///
    /// Returns [`GridError::CapacityOverflow`] when the dimension product
    /// exceeds [`usize::MAX`]. Returns [`GridError::CellCountMismatch`] when
    /// `cells` does not contain exactly `width * height` values.
    pub fn from_cells(width: usize, height: usize, cells: Vec<T>) -> Result<Self, GridError> {
        let expected = grid_capacity(width, height)?;
        let actual = cells.len();
        if actual != expected {
            return Err(GridError::CellCountMismatch { expected, actual });
        }

        Ok(Self {
            width,
            height,
            cells,
        })
    }

    /// Returns the number of columns.
    #[must_use]
    pub const fn width(&self) -> usize {
        self.width
    }

    /// Returns the number of rows.
    #[must_use]
    pub const fn height(&self) -> usize {
        self.height
    }

    /// Returns the number of cells.
    #[must_use]
    pub fn len(&self) -> usize {
        self.cells.len()
    }

    /// Returns `true` when the grid contains no cells.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.cells.is_empty()
    }

    /// Returns all cells in row-major order.
    #[must_use]
    pub fn cells(&self) -> &[T] {
        &self.cells
    }

    /// Returns all cells in mutable row-major order.
    #[must_use]
    pub fn cells_mut(&mut self) -> &mut [T] {
        &mut self.cells
    }

    /// Returns the cell at `position`.
    ///
    /// Returns `None` when either coordinate is outside the grid.
    #[must_use]
    pub fn get(&self, position: Position) -> Option<&T> {
        self.index(position).map(|index| &self.cells[index])
    }

    /// Returns the mutable cell at `position`.
    ///
    /// Returns `None` when either coordinate is outside the grid.
    #[must_use]
    pub fn get_mut(&mut self, position: Position) -> Option<&mut T> {
        self.index(position).map(|index| &mut self.cells[index])
    }

    pub(crate) fn swap_cells(&mut self, cells: &mut Vec<T>) {
        std::mem::swap(&mut self.cells, cells);
    }

    fn index(&self, position: Position) -> Option<usize> {
        (position.x < self.width && position.y < self.height)
            .then(|| position.y * self.width + position.x)
    }
}

impl<T: Clone> Grid<T> {
    /// Creates a grid and clones `cell` into each position.
    ///
    /// A zero width or height creates an empty grid.
    ///
    /// # Errors
    ///
    /// Returns [`GridError::CapacityOverflow`] when the dimension product
    /// exceeds [`usize::MAX`].
    pub fn new(width: usize, height: usize, cell: T) -> Result<Self, GridError> {
        let capacity = grid_capacity(width, height)?;
        Self::from_cells(width, height, vec![cell; capacity])
    }
}

fn grid_capacity(width: usize, height: usize) -> Result<usize, GridError> {
    width.checked_mul(height).ok_or(GridError::CapacityOverflow)
}
