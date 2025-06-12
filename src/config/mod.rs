//! Configuration management for the reverse Game of Life solver

pub mod settings;

pub use settings::{
    Settings, BoundaryCondition, OptimizationLevel,
    OutputFormat, CliOverrides
};