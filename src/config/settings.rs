//! Configuration settings for the reverse Game of Life solver

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Settings {
    pub simulation: SimulationConfig,
    pub solver: SolverConfig,
    pub input: InputConfig,
    pub output: OutputConfig,
    pub encoding: EncodingConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SimulationConfig {
    pub generations: usize,
    pub boundary_condition: BoundaryCondition,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum BoundaryCondition {
    Dead,
    Wrap,
    Mirror,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SolverConfig {
    pub max_solutions: usize,
    pub timeout_seconds: u64,
    pub optimization_level: OptimizationLevel,
    pub backend: SolverBackend,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum SolverBackend {
    Cadical,
    Parkissat,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum OptimizationLevel {
    Fast,
    Balanced,
    Thorough,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InputConfig {
    pub target_state_file: PathBuf,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OutputConfig {
    pub format: OutputFormat,
    pub save_intermediate: bool,
    pub output_directory: PathBuf,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum OutputFormat {
    Text,
    Json,
    Visual,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EncodingConfig {
    pub symmetry_breaking: bool,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            simulation: SimulationConfig {
                generations: 5,
                boundary_condition: BoundaryCondition::Dead,
            },
            solver: SolverConfig {
                max_solutions: 10,
                timeout_seconds: 300,
                optimization_level: OptimizationLevel::Balanced,
                backend: SolverBackend::Cadical,
            },
            input: InputConfig {
                target_state_file: PathBuf::from("input/target_states/example.txt"),
            },
            output: OutputConfig {
                format: OutputFormat::Text,
                save_intermediate: false,
                output_directory: PathBuf::from("output/solutions"),
            },
            encoding: EncodingConfig {
                symmetry_breaking: false,
            },
        }
    }
}

impl Settings {
    /// Load settings from a YAML file
    pub fn from_file(path: &PathBuf) -> Result<Self> {
        let content = std::fs::read_to_string(path)
            .with_context(|| format!("Failed to read config file: {}", path.display()))?;
        
        let settings: Settings = serde_yaml::from_str(&content)
            .with_context(|| format!("Failed to parse config file: {}", path.display()))?;
        
        settings.validate()?;
        Ok(settings)
    }

    /// Save settings to a YAML file
    pub fn to_file(&self, path: &PathBuf) -> Result<()> {
        let content = serde_yaml::to_string(self)
            .context("Failed to serialize settings")?;
        
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)
                .with_context(|| format!("Failed to create directory: {}", parent.display()))?;
        }
        
        std::fs::write(path, content)
            .with_context(|| format!("Failed to write config file: {}", path.display()))?;
        
        Ok(())
    }

    /// Validate the settings
    pub fn validate(&self) -> Result<()> {
        if self.simulation.generations == 0 {
            anyhow::bail!("Number of generations must be positive");
        }
        
        if self.solver.max_solutions == 0 {
            anyhow::bail!("Maximum solutions must be positive");
        }
        
        if !self.input.target_state_file.exists() {
            anyhow::bail!("Target state file does not exist: {}", self.input.target_state_file.display());
        }
        
        Ok(())
    }

    /// Merge settings with command line overrides
    pub fn merge_with_cli(&mut self, cli_overrides: &CliOverrides) {
        if let Some(generations) = cli_overrides.generations {
            self.simulation.generations = generations;
        }
        if let Some(max_solutions) = cli_overrides.max_solutions {
            self.solver.max_solutions = max_solutions;
        }
        if let Some(ref target_file) = cli_overrides.target_file {
            self.input.target_state_file = target_file.clone();
        }
        if let Some(ref output_dir) = cli_overrides.output_dir {
            self.output.output_directory = output_dir.clone();
        }
    }
}

/// Command line overrides for settings
#[derive(Debug, Default)]
pub struct CliOverrides {
    pub generations: Option<usize>,
    pub max_solutions: Option<usize>,
    pub target_file: Option<PathBuf>,
    pub output_dir: Option<PathBuf>,
}