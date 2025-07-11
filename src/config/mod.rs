//! Configuration management for the reverse Game of Life solver

pub mod settings;

pub use settings::{
    Settings, SimulationConfig, SolverConfig, InputConfig, OutputConfig, EncodingConfig,
    BoundaryCondition, OutputFormat, CliOverrides, SolverBackend
};