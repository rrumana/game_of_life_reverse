//! Game of Life core functionality

pub mod grid;
pub mod rules;
pub mod io;

pub use grid::Grid;
pub use rules::GameOfLifeRules;
pub use io::{load_grid_from_file, save_grid_to_file, create_example_grids};