//! Demonstration of ParKissat-RS integration
//! 
//! This example shows how to use both CaDiCaL and ParKissat-RS solvers
//! through the unified interface.

use game_of_life_reverse::sat::{UnifiedSatSolver, SolverOptions, OptimizationLevel};
use game_of_life_reverse::sat::constraints::Clause;
use game_of_life_reverse::config::SolverBackend;
use std::time::Duration;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== SAT Solver Backend Demonstration ===\n");

    // Test both solver backends
    test_solver_backend(SolverBackend::Cadical)?;
    test_solver_backend(SolverBackend::Parkissat)?;

    println!("✅ All solver backends working correctly!");
    Ok(())
}

fn test_solver_backend(backend: SolverBackend) -> Result<(), Box<dyn std::error::Error>> {
    println!("Testing {:?} solver backend:", backend);
    
    // Create solver with specified backend
    let mut solver = UnifiedSatSolver::new(backend)?;
    
    // Configure the solver
    let options = SolverOptions {
        optimization_level: OptimizationLevel::Fast,
        timeout: Some(Duration::from_secs(10)),
        random_seed: Some(42),
    };
    solver.configure(&options)?;
    
    // Test 1: Simple satisfiable problem
    println!("  Test 1: Simple satisfiable problem (x1)");
    solver.add_clause(&Clause::new(vec![1]))?;
    
    let result = solver.solve()?;
    match result {
        Some(solution) => {
            println!("    ✅ SAT - Variable 1 = {:?}", solution.assignment.get(&1));
            println!("    ⏱️  Solve time: {:.3}ms", solution.solve_time.as_secs_f64() * 1000.0);
        }
        None => {
            println!("    ❌ Unexpected UNSAT result");
            return Err("Expected SAT but got UNSAT".into());
        }
    }
    
    // Reset solver for next test
    solver.reset()?;
    solver.configure(&options)?;
    
    // Test 2: Unsatisfiable problem
    println!("  Test 2: Unsatisfiable problem (x1 ∧ ¬x1)");
    solver.add_clause(&Clause::new(vec![1]))?;   // x1
    solver.add_clause(&Clause::new(vec![-1]))?;  // ¬x1
    
    let result = solver.solve()?;
    match result {
        Some(_) => {
            println!("    ❌ Unexpected SAT result");
            return Err("Expected UNSAT but got SAT".into());
        }
        None => {
            println!("    ✅ UNSAT - Correctly detected contradiction");
        }
    }
    
    // Reset solver for next test
    solver.reset()?;
    solver.configure(&options)?;
    
    // Test 3: Multiple solutions
    println!("  Test 3: Multiple solutions (x1 ∨ x2)");
    solver.add_clause(&Clause::new(vec![1, 2]))?;  // x1 ∨ x2
    
    let solutions = solver.solve_multiple(3)?;
    println!("    ✅ Found {} solutions", solutions.len());
    for (i, solution) in solutions.iter().enumerate() {
        println!("      Solution {}: x1={:?}, x2={:?}", 
                 i + 1,
                 solution.assignment.get(&1).unwrap_or(&false),
                 solution.assignment.get(&2).unwrap_or(&false));
    }
    
    // Print solver statistics
    let stats = solver.statistics();
    println!("  📊 Statistics:");
    println!("    Variables: {}", stats.variable_count);
    println!("    Clauses: {}", stats.clause_count);
    
    println!("  ✅ {:?} backend tests completed successfully\n", backend);
    Ok(())
}