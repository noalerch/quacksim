use quacksim::{Grid, GridError};

#[test]
fn new_grid_has_requested_dimensions_and_initial_values() {
    let grid = Grid::new(2, 3, 7).unwrap();

    assert_eq!(grid.width(), 2);
    assert_eq!(grid.height(), 3);
    assert_eq!(grid.len(), 6);
    assert_eq!(grid.cells(), &[7; 6]);
}

#[test]
fn grid_accepts_an_empty_axis() {
    let grid = Grid::new(0, 4, 7).unwrap();

    assert_eq!(grid.width(), 0);
    assert_eq!(grid.height(), 4);
    assert!(grid.is_empty());
}

#[test]
fn grid_rejects_an_incorrect_cell_count() {
    let error = Grid::from_cells(2, 2, vec![0; 3]).unwrap_err();

    assert_eq!(
        error,
        GridError::CellCountMismatch {
            expected: 4,
            actual: 3,
        }
    );
}

#[test]
fn grid_rejects_dimension_overflow() {
    let error = Grid::new(usize::MAX, 2, 0).unwrap_err();

    assert_eq!(error, GridError::CapacityOverflow);
}

#[test]
fn grid_errors_have_clear_messages() {
    assert_eq!(
        GridError::CellCountMismatch {
            expected: 4,
            actual: 3,
        }
        .to_string(),
        "grid needs 4 cells but received 3 cells"
    );
    assert_eq!(
        GridError::CapacityOverflow.to_string(),
        "grid capacity exceeds usize"
    );
}
