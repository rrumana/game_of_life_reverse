//! Variable management for SAT encoding

use std::collections::HashMap;
use anyhow::Result;

/// Types of variables used in the SAT encoding
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum VariableType {
    /// Cell state at position (x, y, t)
    Cell { x: usize, y: usize, t: usize },
}

/// Manages SAT variables and their mapping to integers
#[derive(Debug)]
pub struct VariableManager {
    /// Map from variable type to SAT variable ID (positive integer)
    variable_map: HashMap<VariableType, i32>,
    /// Next available variable ID
    next_id: i32,
    /// Grid dimensions
    width: usize,
    height: usize,
    /// Number of time steps
    time_steps: usize,
}

impl VariableManager {
    /// Create a new variable manager
    pub fn new(width: usize, height: usize, time_steps: usize, _use_auxiliary: bool) -> Self {
        Self {
            variable_map: HashMap::new(),
            next_id: 1, // SAT variables start from 1
            width,
            height,
            time_steps,
        }
    }

    /// Get or create a variable ID for the given variable type
    pub fn get_variable(&mut self, var_type: VariableType) -> Result<i32> {
        if let Some(&id) = self.variable_map.get(&var_type) {
            return Ok(id);
        }

        // Validate the variable type
        self.validate_variable(&var_type)?;

        let id = self.next_id;
        self.next_id += 1;
        self.variable_map.insert(var_type, id);
        Ok(id)
    }

    /// Get variable ID for a cell at specific coordinates and time
    pub fn cell_variable(&mut self, x: usize, y: usize, t: usize) -> Result<i32> {
        self.get_variable(VariableType::Cell { x, y, t })
    }


    /// Get all cell variables for a specific time step
    pub fn all_cell_variables_at_time(&mut self, t: usize) -> Result<Vec<i32>> {
        let mut variables = Vec::new();
        for y in 0..self.height {
            for x in 0..self.width {
                variables.push(self.cell_variable(x, y, t)?);
            }
        }
        Ok(variables)
    }


    /// Get the total number of variables created
    pub fn variable_count(&self) -> usize {
        (self.next_id - 1) as usize
    }

    /// Get grid dimensions
    pub fn dimensions(&self) -> (usize, usize, usize) {
        (self.width, self.height, self.time_steps)
    }


    /// Validate that a variable type is within bounds
    fn validate_variable(&self, var_type: &VariableType) -> Result<()> {
        match var_type {
            VariableType::Cell { x, y, t } => {
                if *x >= self.width {
                    anyhow::bail!("Cell x coordinate {} out of bounds (width: {})", x, self.width);
                }
                if *y >= self.height {
                    anyhow::bail!("Cell y coordinate {} out of bounds (height: {})", y, self.height);
                }
                if *t >= self.time_steps {
                    anyhow::bail!("Time step {} out of bounds (time_steps: {})", t, self.time_steps);
                }
            }
        }
        Ok(())
    }

    /// Get statistics about variable usage
    pub fn statistics(&self) -> VariableStatistics {
        let mut cell_vars = 0;

        for var_type in self.variable_map.keys() {
            match var_type {
                VariableType::Cell { .. } => cell_vars += 1,
            }
        }

        VariableStatistics {
            total_variables: self.variable_count(),
            cell_variables: cell_vars,
        }
    }

    /// Clear all variables (useful for testing)
    pub fn clear(&mut self) {
        self.variable_map.clear();
        self.next_id = 1;
    }
}

/// Statistics about variable usage
#[derive(Debug, Clone)]
pub struct VariableStatistics {
    pub total_variables: usize,
    pub cell_variables: usize,
}

impl std::fmt::Display for VariableStatistics {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "Variable Statistics:")?;
        writeln!(f, "  Total variables: {}", self.total_variables)?;
        writeln!(f, "  Cell variables: {}", self.cell_variables)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_variable_creation() {
        let mut vm = VariableManager::new(3, 3, 2, true);
        
        // Test cell variable creation
        let var1 = vm.cell_variable(0, 0, 0).unwrap();
        let var2 = vm.cell_variable(1, 1, 1).unwrap();
        
        assert_eq!(var1, 1);
        assert_eq!(var2, 2);
        
        // Test that same variable returns same ID
        let var1_again = vm.cell_variable(0, 0, 0).unwrap();
        assert_eq!(var1, var1_again);
    }

    #[test]
    fn test_cell_variables() {
        let mut vm = VariableManager::new(2, 2, 2, false);
        
        let cell_var1 = vm.cell_variable(0, 0, 0).unwrap();
        let cell_var2 = vm.cell_variable(1, 1, 1).unwrap();
        
        assert!(cell_var1 > 0);
        assert!(cell_var2 > 0);
        assert_ne!(cell_var1, cell_var2);
    }

    #[test]
    fn test_variable_bounds() {
        let mut vm = VariableManager::new(2, 2, 2, false);
        
        // These should work
        assert!(vm.cell_variable(0, 0, 0).is_ok());
        assert!(vm.cell_variable(1, 1, 1).is_ok());
        
        // These should fail (out of bounds)
        assert!(vm.cell_variable(2, 0, 0).is_err()); // x out of bounds
        assert!(vm.cell_variable(0, 2, 0).is_err()); // y out of bounds
        assert!(vm.cell_variable(0, 0, 2).is_err()); // t out of bounds
    }

    #[test]
    fn test_all_variables_at_time() {
        let mut vm = VariableManager::new(2, 2, 2, false);
        
        let vars = vm.all_cell_variables_at_time(0).unwrap();
        assert_eq!(vars.len(), 4); // 2x2 grid
        
        // All variables should be unique
        let mut unique_vars = vars.clone();
        unique_vars.sort();
        unique_vars.dedup();
        assert_eq!(vars.len(), unique_vars.len());
    }

    #[test]
    fn test_statistics() {
        let mut vm = VariableManager::new(2, 2, 2, false);
        
        vm.cell_variable(0, 0, 0).unwrap();
        vm.cell_variable(1, 1, 1).unwrap();
        
        let stats = vm.statistics();
        assert_eq!(stats.total_variables, 2);
        assert_eq!(stats.cell_variables, 2);
    }
}