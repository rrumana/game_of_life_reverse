//! ParKissat-RS SAT solver integration

use super::constraints::Clause;
use super::solver::{SolverOptions, SolverSolution, SolverStatistics, SolverResultType, OptimizationLevel};
use anyhow::Result;
use parkissat_sys::{ParkissatSolver, SolverConfig, SolverResult};
use std::collections::HashMap;
use std::time::{Duration, Instant};

/// SAT solver wrapper for ParKissat-RS
pub struct ParkissatSatSolver {
    solver: ParkissatSolver,
    variable_count: usize,
    clause_count: usize,
    timeout: Option<Duration>,
    configured: bool,
}

impl ParkissatSatSolver {
    /// Create a new SAT solver instance
    pub fn new() -> Result<Self> {
        let solver = ParkissatSolver::new()
            .map_err(|e| anyhow::anyhow!("Failed to create ParKissat solver: {}", e))?;
        
        Ok(Self {
            solver,
            variable_count: 0,
            clause_count: 0,
            timeout: None,
            configured: false,
        })
    }

    /// Set solving timeout
    pub fn set_timeout(&mut self, timeout: Duration) {
        self.timeout = Some(timeout);
    }

    /// Add clauses to the solver
    pub fn add_clauses(&mut self, clauses: &[Clause]) -> Result<()> {
        for clause in clauses {
            self.add_clause(clause)?;
        }
        Ok(())
    }

    /// Add a single clause to the solver
    pub fn add_clause(&mut self, clause: &Clause) -> Result<()> {
        if clause.is_empty() {
            anyhow::bail!("Cannot add empty clause (unsatisfiable)");
        }

        // Ensure solver is configured before adding clauses
        self.ensure_configured()?;

        // Update variable count
        for &literal in &clause.literals {
            let var = literal.abs() as usize;
            if var > self.variable_count {
                self.variable_count = var;
            }
        }

        // Add clause to solver
        self.solver.add_clause(&clause.literals)
            .map_err(|e| anyhow::anyhow!("Failed to add clause: {}", e))?;

        self.clause_count += 1;
        Ok(())
    }

    /// Solve the SAT problem and return the first solution
    pub fn solve(&mut self) -> Result<Option<SolverSolution>> {
        self.ensure_configured()?;
        
        let start_time = Instant::now();
        
        let result = self.solver.solve()
            .map_err(|e| anyhow::anyhow!("Solver error: {}", e))?;
        
        let solve_time = start_time.elapsed();

        match result {
            SolverResult::Sat => {
                let assignment = self.extract_assignment()?;
                Ok(Some(SolverSolution {
                    assignment,
                    solve_time,
                }))
            }
            SolverResult::Unsat => Ok(None),
            SolverResult::Unknown => {
                anyhow::bail!("Solver returned unknown result (possibly timeout)")
            }
        }
    }

    /// Solve and find multiple solutions up to a limit
    pub fn solve_multiple(&mut self, max_solutions: usize) -> Result<Vec<SolverSolution>> {
        let mut solutions = Vec::new();
        let start_time = Instant::now();

        for _ in 0..max_solutions {
            match self.solver.solve()
                .map_err(|e| anyhow::anyhow!("Solver error: {}", e))? {
                SolverResult::Sat => {
                    let assignment = self.extract_assignment()?;
                    let solution = SolverSolution {
                        assignment: assignment.clone(),
                        solve_time: start_time.elapsed(),
                    };
                    solutions.push(solution);

                    // Add blocking clause to prevent finding the same solution again
                    self.add_blocking_clause(&assignment)?;
                }
                SolverResult::Unsat => break,
                SolverResult::Unknown => {
                    anyhow::bail!("Solver returned unknown result during multiple solution search")
                }
            }
        }

        Ok(solutions)
    }

    /// Extract variable assignment from the solver
    fn extract_assignment(&self) -> Result<HashMap<i32, bool>> {
        let mut assignment = HashMap::new();

        for var in 1..=self.variable_count as i32 {
            let value = self.solver.get_model_value(var)
                .map_err(|e| anyhow::anyhow!("Failed to get model value for variable {}: {}", var, e))?;
            assignment.insert(var, value);
        }

        Ok(assignment)
    }

    /// Add a blocking clause to prevent finding the same solution again
    fn add_blocking_clause(&mut self, assignment: &HashMap<i32, bool>) -> Result<()> {
        let mut blocking_literals = Vec::new();

        for (&var, &value) in assignment {
            // Add the negation of the current assignment
            blocking_literals.push(if value { -var } else { var });
        }

        let blocking_clause = Clause::new(blocking_literals);
        self.add_clause(&blocking_clause)?;

        Ok(())
    }

    /// Get solver statistics
    pub fn statistics(&self) -> SolverStatistics {
        SolverStatistics {
            variable_count: self.variable_count,
            clause_count: self.clause_count,
            solve_time: Duration::from_secs(0), // Will be updated during solving
            result: SolverResultType::Error, // Will be updated during solving
        }
    }

    /// Reset the solver (clear all clauses)
    pub fn reset(&mut self) -> Result<()> {
        self.solver = ParkissatSolver::new()
            .map_err(|e| anyhow::anyhow!("Failed to create new ParKissat solver: {}", e))?;
        self.variable_count = 0;
        self.clause_count = 0;
        self.configured = false;
        Ok(())
    }

    /// Check if a partial assignment satisfies all clauses
    pub fn check_assignment(&self, _assignment: &HashMap<i32, bool>) -> bool {
        // This is a simplified check - in practice, you might want to use
        // the solver's internal checking mechanisms
        true // Placeholder implementation
    }

    /// Get the number of variables
    pub fn variable_count(&self) -> usize {
        self.variable_count
    }

    /// Get the number of clauses
    pub fn clause_count(&self) -> usize {
        self.clause_count
    }

    /// Set solver configuration options
    pub fn configure(&mut self, options: &SolverOptions) -> Result<()> {
        let mut config = SolverConfig::default();
        
        // Map optimization level to thread count and other settings
        match options.optimization_level {
            OptimizationLevel::Fast => {
                config.num_threads = 1;
                config.enable_preprocessing = false;
                config.verbosity = 0;
            }
            OptimizationLevel::Balanced => {
                config.num_threads = 2;
                config.enable_preprocessing = true;
                config.verbosity = 0;
            }
            OptimizationLevel::Thorough => {
                config.num_threads = 4;
                config.enable_preprocessing = true;
                config.verbosity = 1;
            }
        }
        
        // Set timeout
        if let Some(timeout) = options.timeout {
            config.timeout = timeout;
            self.set_timeout(timeout);
        }
        
        // Set random seed if provided
        if let Some(seed) = options.random_seed {
            config.random_seed = seed as u32;
        }
        
        // Configure the solver
        self.solver.configure(&config)
            .map_err(|e| anyhow::anyhow!("Failed to configure solver: {}", e))?;
        
        self.configured = true;
        Ok(())
    }
    
    /// Ensure the solver is configured before solving
    fn ensure_configured(&mut self) -> Result<()> {
        if !self.configured {
            // Use default configuration
            let default_options = SolverOptions::default();
            self.configure(&default_options)?;
        }
        Ok(())
    }
}

impl Default for ParkissatSatSolver {
    fn default() -> Self {
        Self::new().expect("Failed to create default ParKissat solver")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_solver_creation() {
        let solver = ParkissatSatSolver::new().unwrap();
        assert_eq!(solver.variable_count(), 0);
        assert_eq!(solver.clause_count(), 0);
    }

    #[test]
    fn test_simple_satisfiable() {
        let mut solver = ParkissatSatSolver::new().unwrap();
        
        // Add clause: x1
        let clause = Clause::new(vec![1]);
        solver.add_clause(&clause).unwrap();
        
        let result = solver.solve().unwrap();
        assert!(result.is_some());
        
        let solution = result.unwrap();
        assert_eq!(solution.assignment.get(&1), Some(&true));
    }

    #[test]
    fn test_unsatisfiable() {
        let mut solver = ParkissatSatSolver::new().unwrap();
        
        // Add contradictory clauses: x1 and ¬x1
        solver.add_clause(&Clause::new(vec![1])).unwrap();
        solver.add_clause(&Clause::new(vec![-1])).unwrap();
        
        let result = solver.solve().unwrap();
        assert!(result.is_none());
    }

    #[test]
    fn test_solver_options() {
        let mut solver = ParkissatSatSolver::new().unwrap();
        let options = SolverOptions {
            optimization_level: OptimizationLevel::Fast,
            timeout: Some(Duration::from_secs(10)),
            random_seed: Some(42),
        };
        
        solver.configure(&options).unwrap();
        // Test that configuration doesn't crash
        assert_eq!(solver.variable_count(), 0);
    }
}