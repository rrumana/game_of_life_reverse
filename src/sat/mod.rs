//! SAT solving components for reverse Game of Life

pub mod variables;
pub mod constraints;
pub mod encoder;
pub mod solver;

pub use variables::VariableManager;
pub use constraints::ConstraintGenerator;
pub use encoder::SatEncoder;
pub use solver::{SatSolver, SolverOptions, SolverSolution};