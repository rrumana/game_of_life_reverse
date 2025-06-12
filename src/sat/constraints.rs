//! Constraint generation for Game of Life SAT encoding

use super::VariableManager;
use crate::config::BoundaryCondition;
use crate::game_of_life::{Grid, GameOfLifeRules};
use anyhow::Result;

/// Represents a SAT clause (disjunction of literals)
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Clause {
    pub literals: Vec<i32>, // Positive for variable, negative for negation
}

impl Clause {
    /// Create a new clause from literals
    pub fn new(literals: Vec<i32>) -> Self {
        Self { literals }
    }

    /// Create a unit clause (single literal)
    pub fn unit(literal: i32) -> Self {
        Self { literals: vec![literal] }
    }

    /// Create a binary clause (two literals)
    pub fn binary(lit1: i32, lit2: i32) -> Self {
        Self { literals: vec![lit1, lit2] }
    }

    /// Check if clause is empty (unsatisfiable)
    pub fn is_empty(&self) -> bool {
        self.literals.is_empty()
    }

    /// Check if clause is unit
    pub fn is_unit(&self) -> bool {
        self.literals.len() == 1
    }
}

/// Generates SAT constraints for the reverse Game of Life problem
pub struct ConstraintGenerator {
    variable_manager: VariableManager,
    width: usize,
    height: usize,
    time_steps: usize,
    boundary_condition: BoundaryCondition,
}

impl ConstraintGenerator {
    /// Create a new constraint generator
    pub fn new(
        width: usize,
        height: usize,
        time_steps: usize,
        boundary_condition: BoundaryCondition,
    ) -> Self {
        let variable_manager = VariableManager::new(width, height, time_steps, false);
        
        Self {
            variable_manager,
            width,
            height,
            time_steps,
            boundary_condition,
        }
    }

    /// Generate all constraints for the reverse Game of Life problem
    pub fn generate_all_constraints(&mut self, target_grid: &Grid) -> Result<Vec<Clause>> {
        let mut clauses = Vec::new();

        // 1. Target state constraints (final time step must match target)
        clauses.extend(self.generate_target_constraints(target_grid)?);

        // 2. Game of Life transition constraints for each time step
        for t in 0..self.time_steps - 1 {
            clauses.extend(self.generate_transition_constraints(t)?);
        }

        Ok(clauses)
    }

    /// Generate constraints that fix the final state to match the target
    fn generate_target_constraints(&mut self, target_grid: &Grid) -> Result<Vec<Clause>> {
        let mut clauses = Vec::new();
        let final_time = self.time_steps - 1;

        if target_grid.width != self.width || target_grid.height != self.height {
            anyhow::bail!("Target grid dimensions ({}, {}) don't match problem dimensions ({}, {})",
                         target_grid.width, target_grid.height, self.width, self.height);
        }

        for y in 0..self.height {
            for x in 0..self.width {
                let cell_var = self.variable_manager.cell_variable(x, y, final_time)?;
                let target_alive = target_grid.get(y, x);


                if target_alive {
                    // Cell must be alive
                    clauses.push(Clause::unit(cell_var));
                } else {
                    // Cell must be dead
                    clauses.push(Clause::unit(-cell_var));
                }
            }
        }

        Ok(clauses)
    }

    /// Generate Game of Life transition constraints between time steps
    fn generate_transition_constraints(&mut self, t: usize) -> Result<Vec<Clause>> {
        let mut clauses = Vec::new();

        for y in 0..self.height {
            for x in 0..self.width {
                clauses.extend(self.generate_cell_transition_constraints(x, y, t)?);
            }
        }

        Ok(clauses)
    }

    /// Generate transition constraints for a specific cell
    fn generate_cell_transition_constraints(&mut self, x: usize, y: usize, t: usize) -> Result<Vec<Clause>> {
        let mut clauses = Vec::new();

        let current_cell = self.variable_manager.cell_variable(x, y, t)?;
        let next_cell = self.variable_manager.cell_variable(x, y, t + 1)?;

        clauses.extend(self.generate_direct_transition_constraints(x, y, t, current_cell, next_cell)?);

        Ok(clauses)
    }


    /// Generate transition constraints without auxiliary variables (direct encoding)
    fn generate_direct_transition_constraints(
        &mut self,
        x: usize,
        y: usize,
        t: usize,
        current_cell: i32,
        next_cell: i32,
    ) -> Result<Vec<Clause>> {
        let mut clauses = Vec::new();

        // Get neighbor variables
        let neighbor_vars = self.get_neighbor_variables(x, y, t)?;

        // Generate constraints for each possible neighbor count
        for k in 0..=8 {
            // Generate all combinations of k neighbors being alive
            let neighbor_combinations = self.generate_neighbor_combinations(&neighbor_vars, k);

            for combination in neighbor_combinations {
                if GameOfLifeRules::should_be_alive(true, k) {
                    // If current cell is alive and exactly k neighbors are alive, next cell should be alive
                    let mut clause = vec![-current_cell, next_cell];
                    clause.extend(combination.iter().map(|&(var, alive)| if alive { -var } else { var }));
                    clauses.push(Clause::new(clause));
                } else {
                    // If current cell is alive and exactly k neighbors are alive, next cell should be dead
                    let mut clause = vec![-current_cell, -next_cell];
                    clause.extend(combination.iter().map(|&(var, alive)| if alive { -var } else { var }));
                    clauses.push(Clause::new(clause));
                }

                if GameOfLifeRules::should_be_alive(false, k) {
                    // If current cell is dead and exactly k neighbors are alive, next cell should be alive
                    let mut clause = vec![current_cell, next_cell];
                    clause.extend(combination.iter().map(|&(var, alive)| if alive { -var } else { var }));
                    clauses.push(Clause::new(clause));
                } else {
                    // If current cell is dead and exactly k neighbors are alive, next cell should be dead
                    let mut clause = vec![current_cell, -next_cell];
                    clause.extend(combination.iter().map(|&(var, alive)| if alive { -var } else { var }));
                    clauses.push(Clause::new(clause));
                }
            }
        }

        Ok(clauses)
    }


    /// Get neighbor variables for a cell, handling boundary conditions
    fn get_neighbor_variables(&mut self, x: usize, y: usize, t: usize) -> Result<Vec<i32>> {
        let mut neighbors = Vec::new();

        for dy in -1..=1 {
            for dx in -1..=1 {
                if dx == 0 && dy == 0 {
                    continue; // Skip the cell itself
                }

                let nx = x as isize + dx;
                let ny = y as isize + dy;

                if let Some(neighbor_var) = self.get_neighbor_variable_with_boundary(nx, ny, t)? {
                    neighbors.push(neighbor_var);
                }
            }
        }

        Ok(neighbors)
    }

    /// Get neighbor variable handling boundary conditions
    fn get_neighbor_variable_with_boundary(&mut self, x: isize, y: isize, t: usize) -> Result<Option<i32>> {
        match self.boundary_condition {
            BoundaryCondition::Dead => {
                if x >= 0 && x < self.width as isize && y >= 0 && y < self.height as isize {
                    Ok(Some(self.variable_manager.cell_variable(x as usize, y as usize, t)?))
                } else {
                    Ok(None) // Out of bounds cells are always dead (no variable needed)
                }
            }
            BoundaryCondition::Wrap => {
                let wrapped_x = ((x % self.width as isize + self.width as isize) % self.width as isize) as usize;
                let wrapped_y = ((y % self.height as isize + self.height as isize) % self.height as isize) as usize;
                Ok(Some(self.variable_manager.cell_variable(wrapped_x, wrapped_y, t)?))
            }
            BoundaryCondition::Mirror => {
                let mirrored_x = if x < 0 {
                    (-x - 1) as usize
                } else if x >= self.width as isize {
                    self.width - 1 - (x - self.width as isize) as usize
                } else {
                    x as usize
                };

                let mirrored_y = if y < 0 {
                    (-y - 1) as usize
                } else if y >= self.height as isize {
                    self.height - 1 - (y - self.height as isize) as usize
                } else {
                    y as usize
                };

                if mirrored_x < self.width && mirrored_y < self.height {
                    Ok(Some(self.variable_manager.cell_variable(mirrored_x, mirrored_y, t)?))
                } else {
                    Ok(None)
                }
            }
        }
    }

    /// Generate all combinations of exactly k neighbors being alive
    fn generate_neighbor_combinations(&self, neighbor_vars: &[i32], k: u8) -> Vec<Vec<(i32, bool)>> {
        let n = neighbor_vars.len();
        let k = k as usize;
        let mut combinations = Vec::new();

        if k > n {
            return combinations; // Impossible to have more alive than total neighbors
        }

        // Generate all combinations of choosing k positions to be alive
        let mut alive_indices = Vec::new();
        self.generate_k_combinations(n, k, 0, &mut alive_indices, &mut combinations, neighbor_vars);

        combinations
    }

    /// Generate combinations of exactly k alive neighbors using iterative approach
    fn generate_k_combinations(
        &self,
        n: usize,
        k: usize,
        start: usize,
        current_alive: &mut Vec<usize>,
        result: &mut Vec<Vec<(i32, bool)>>,
        neighbor_vars: &[i32],
    ) {
        if current_alive.len() == k {
            // Create a combination with exactly k alive neighbors
            let mut combination = Vec::new();
            for (i, &var) in neighbor_vars.iter().enumerate() {
                let is_alive = current_alive.contains(&i);
                combination.push((var, is_alive));
            }
            result.push(combination);
            return;
        }

        if start >= n || current_alive.len() + (n - start) < k {
            return; // Not enough remaining positions to reach k
        }

        // Include current position as alive
        current_alive.push(start);
        self.generate_k_combinations(n, k, start + 1, current_alive, result, neighbor_vars);
        current_alive.pop();

        // Skip current position (keep it dead)
        self.generate_k_combinations(n, k, start + 1, current_alive, result, neighbor_vars);
    }


    /// Get the variable manager (for external access)
    pub fn variable_manager(&mut self) -> &mut VariableManager {
        &mut self.variable_manager
    }

    /// Get constraint generation statistics
    pub fn statistics(&self) -> ConstraintStatistics {
        ConstraintStatistics {
            width: self.width,
            height: self.height,
            time_steps: self.time_steps,
            total_variables: self.variable_manager.variable_count(),
        }
    }
}

/// Statistics about constraint generation
#[derive(Debug, Clone)]
pub struct ConstraintStatistics {
    pub width: usize,
    pub height: usize,
    pub time_steps: usize,
    pub total_variables: usize,
}

impl std::fmt::Display for ConstraintStatistics {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "Constraint Generation Statistics:")?;
        writeln!(f, "  Grid size: {}x{}", self.width, self.height)?;
        writeln!(f, "  Time steps: {}", self.time_steps)?;
        writeln!(f, "  Total variables: {}", self.total_variables)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::BoundaryCondition;

    #[test]
    fn test_clause_creation() {
        let clause = Clause::new(vec![1, -2, 3]);
        assert_eq!(clause.literals, vec![1, -2, 3]);
        assert!(!clause.is_empty());
        assert!(!clause.is_unit());

        let unit_clause = Clause::unit(5);
        assert!(unit_clause.is_unit());
        assert_eq!(unit_clause.literals, vec![5]);
    }

    #[test]
    fn test_constraint_generator_creation() {
        let cg = ConstraintGenerator::new(
            3, 3, 2,
            BoundaryCondition::Dead,
            NeighborEncoding::Direct,
            false
        );

        assert_eq!(cg.width, 3);
        assert_eq!(cg.height, 3);
        assert_eq!(cg.time_steps, 2);
        assert!(!cg.use_auxiliary);
    }

    #[test]
    fn test_target_constraints() {
        let mut cg = ConstraintGenerator::new(
            2, 2, 2,
            BoundaryCondition::Dead,
            NeighborEncoding::Direct,
            false
        );

        let cells = vec![
            vec![true, false],
            vec![false, true],
        ];
        let target_grid = Grid::from_cells(cells, BoundaryCondition::Dead).unwrap();

        let constraints = cg.generate_target_constraints(&target_grid).unwrap();
        assert_eq!(constraints.len(), 4); // 2x2 grid = 4 cells

        // Check that constraints fix the target state
        assert!(constraints.iter().any(|c| c.literals == vec![cg.variable_manager.cell_variable(0, 0, 1).unwrap()]));
        assert!(constraints.iter().any(|c| c.literals == vec![-cg.variable_manager.cell_variable(1, 0, 1).unwrap()]));
    }
}