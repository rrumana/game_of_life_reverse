//! SAT solving components for reverse Game of Life

pub mod variables;
pub mod constraints;
pub mod encoder;
pub mod solver;
pub mod parkissat_solver;
pub mod solver_factory;

pub use variables::VariableManager;
pub use constraints::ConstraintGenerator;
pub use encoder::SatEncoder;
pub use solver::{SatSolver, SolverOptions, SolverSolution, SolverStatistics, SolverResultType, OptimizationLevel};
pub use parkissat_solver::ParkissatSatSolver;
pub use solver_factory::UnifiedSatSolver;