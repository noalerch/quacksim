use quacksim::{Grid, Position};

#[test]
fn grid_uses_row_major_storage() {
    let grid = Grid::from_cells(3, 2, vec![0, 1, 2, 3, 4, 5]).unwrap();

    assert_eq!(grid.get(Position::new(0, 0)), Some(&0));
    assert_eq!(grid.get(Position::new(2, 0)), Some(&2));
    assert_eq!(grid.get(Position::new(0, 1)), Some(&3));
    assert_eq!(grid.get(Position::new(2, 1)), Some(&5));
}

#[test]
fn grid_rejects_coordinates_outside_each_axis() {
    let grid = Grid::new(2, 2, 0).unwrap();

    assert_eq!(grid.get(Position::new(2, 0)), None);
    assert_eq!(grid.get(Position::new(0, 2)), None);
    assert_eq!(grid.get(Position::new(usize::MAX, 0)), None);
}

#[test]
fn empty_grid_has_no_valid_position() {
    let grid = Grid::new(0, 4, 0).unwrap();

    assert_eq!(grid.get(Position::new(0, 0)), None);
}

#[test]
fn mutable_access_changes_only_the_selected_cell() {
    let mut grid = Grid::new(2, 2, 0).unwrap();
    let position = Position::new(1, 0);

    *grid.get_mut(position).unwrap() = 1;

    assert_eq!(grid.cells(), &[0, 1, 0, 0]);
}

#[test]
fn mutable_cell_slice_keeps_grid_storage_contiguous() {
    let mut grid = Grid::new(3, 1, 0).unwrap();

    grid.cells_mut().copy_from_slice(&[1, 2, 3]);

    assert_eq!(grid.cells(), &[1, 2, 3]);
}
