//! Comprehensive benchmark tool for comparing CaDiCaL vs ParKissat-RS solvers
//! 
//! This tool runs both solvers with different optimization levels and measures
//! solve times to compare performance, especially showcasing ParKissat's
//! multithreading capabilities.

use anyhow::{Context, Result};
use game_of_life_reverse::{
    config::{Settings, SolverBackend, OptimizationLevel},
    reverse::ReverseProblem,
};
use std::path::PathBuf;
use std::time::{Duration, Instant};

#[derive(Debug, Clone)]
struct BenchmarkResult {
    solver_backend: SolverBackend,
    optimization_level: OptimizationLevel,
    thread_count: usize,
    run_times: Vec<Duration>,
    average_time: Duration,
    min_time: Duration,
    max_time: Duration,
    success: bool,
    solutions_found: usize,
}

impl BenchmarkResult {
    fn new(
        solver_backend: SolverBackend,
        optimization_level: OptimizationLevel,
        thread_count: usize,
    ) -> Self {
        Self {
            solver_backend,
            optimization_level,
            thread_count,
            run_times: Vec::new(),
            average_time: Duration::ZERO,
            min_time: Duration::MAX,
            max_time: Duration::ZERO,
            success: false,
            solutions_found: 0,
        }
    }

    fn add_run(&mut self, duration: Duration, solutions_found: usize) {
        self.run_times.push(duration);
        self.solutions_found = solutions_found;
        self.success = solutions_found > 0;
        
        if duration < self.min_time {
            self.min_time = duration;
        }
        if duration > self.max_time {
            self.max_time = duration;
        }
        
        // Calculate average
        let total: Duration = self.run_times.iter().sum();
        self.average_time = total / self.run_times.len() as u32;
    }

    fn format_time(duration: Duration) -> String {
        format!("{:.3}s", duration.as_secs_f64())
    }

    fn display(&self) -> String {
        let backend_name = match self.solver_backend {
            SolverBackend::Cadical => "CaDiCaL",
            SolverBackend::Parkissat => "ParKissat-RS",
        };
        
        let opt_level = match self.optimization_level {
            OptimizationLevel::Fast => "Fast",
            OptimizationLevel::Balanced => "Balanced", 
            OptimizationLevel::Thorough => "Thorough",
        };

        let thread_info = if self.solver_backend == SolverBackend::Parkissat {
            format!(" ({} thread{})", self.thread_count, if self.thread_count == 1 { "" } else { "s" })
        } else {
            " (single-threaded)".to_string()
        };

        format!(
            "  {} {}{}:\n    Runs: [{}]\n    Avg: {} | Min: {} | Max: {} | Success: {}",
            backend_name,
            opt_level,
            thread_info,
            self.run_times.iter()
                .map(|d| Self::format_time(*d))
                .collect::<Vec<_>>()
                .join(", "),
            Self::format_time(self.average_time),
            Self::format_time(self.min_time),
            Self::format_time(self.max_time),
            if self.success { "✅" } else { "❌" }
        )
    }
}

struct BenchmarkSuite {
    results: Vec<BenchmarkResult>,
    target_file: PathBuf,
    generations: usize,
    runs_per_config: usize,
}

impl BenchmarkSuite {
    fn new(target_file: PathBuf, generations: usize, runs_per_config: usize) -> Self {
        Self {
            results: Vec::new(),
            target_file,
            generations,
            runs_per_config,
        }
    }

    fn run_comprehensive_benchmark(&mut self) -> Result<()> {
        println!("🚀 Starting Comprehensive SAT Solver Benchmark");
        println!("Target: {} ({} generations, {} runs per config)\n", 
                 self.target_file.display(), self.generations, self.runs_per_config);

        // Define benchmark configurations
        let configs = vec![
            (SolverBackend::Cadical, OptimizationLevel::Fast, 1),
            //(SolverBackend::Cadical, OptimizationLevel::Balanced, 1),
            //(SolverBackend::Cadical, OptimizationLevel::Thorough, 1),
            //(SolverBackend::Parkissat, OptimizationLevel::Fast, 1),
            (SolverBackend::Parkissat, OptimizationLevel::Balanced, 12),
            (SolverBackend::Parkissat, OptimizationLevel::Thorough, 24),
        ];

        for (backend, opt_level, thread_count) in configs {
            println!("🔄 Testing {:?} {:?} ({} thread{})...",
                     backend, opt_level, thread_count, if thread_count == 1 { "" } else { "s" });
            
            let mut result = BenchmarkResult::new(backend, opt_level, thread_count);
            
            for run in 1..=self.runs_per_config {
                print!("  Run {}/{}: ", run, self.runs_per_config);
                
                match self.run_single_benchmark(backend, opt_level) {
                    Ok((duration, solutions_found)) => {
                        result.add_run(duration, solutions_found);
                        println!("✅ {} (found {} solution{})", 
                                BenchmarkResult::format_time(duration),
                                solutions_found,
                                if solutions_found == 1 { "" } else { "s" });
                    }
                    Err(e) => {
                        println!("❌ Failed: {}", e);
                        result.add_run(Duration::from_secs(999), 0);
                    }
                }
            }
            
            self.results.push(result);
            println!();
        }

        Ok(())
    }

    fn run_single_benchmark(
        &self,
        backend: SolverBackend,
        opt_level: OptimizationLevel,
    ) -> Result<(Duration, usize)> {
        // Create settings for this benchmark
        let mut settings = Settings::default();
        settings.solver.backend = backend;
        settings.solver.optimization_level = opt_level;
        settings.solver.max_solutions = 1;
        settings.simulation.generations = self.generations;
        settings.input.target_state_file = self.target_file.clone();
        settings.encoding.symmetry_breaking = false; // Disabled - current implementation is counterproductive

        // Create and solve the problem
        let start_time = Instant::now();
        let mut problem = ReverseProblem::new(settings)
            .context("Failed to create reverse problem")?;
        
        let solutions = problem.solve()
            .context("Failed to solve reverse problem")?;
        
        let duration = start_time.elapsed();
        Ok((duration, solutions.len()))
    }

    fn generate_report(&self) {
        println!("═══════════════════════════════════════════════════════════");
        println!("📊 COMPREHENSIVE SAT SOLVER BENCHMARK RESULTS");
        println!("═══════════════════════════════════════════════════════════");
        println!("Problem: {} ({} generations, {} solution)", 
                 self.target_file.display(), self.generations, 1);
        println!();

        // Group results by solver backend
        let cadical_results: Vec<_> = self.results.iter()
            .filter(|r| r.solver_backend == SolverBackend::Cadical)
            .collect();
        
        let parkissat_results: Vec<_> = self.results.iter()
            .filter(|r| r.solver_backend == SolverBackend::Parkissat)
            .collect();

        // Display CaDiCaL results
        println!("🔧 CaDiCaL Results:");
        for result in &cadical_results {
            println!("{}", result.display());
        }
        println!();

        // Display ParKissat results
        println!("⚡ ParKissat-RS Results:");
        for result in &parkissat_results {
            println!("{}", result.display());
        }
        println!();

        // Threading analysis
        self.generate_threading_analysis(&parkissat_results);

        // Find optimal configuration
        self.find_optimal_configuration();
    }

    fn generate_threading_analysis(&self, parkissat_results: &[&BenchmarkResult]) {
        println!("🧵 THREADING PERFORMANCE ANALYSIS");
        println!("───────────────────────────────────────────────────────────");
        
        if let Some(single_thread) = parkissat_results.iter().find(|r| r.thread_count == 1) {
            if let Some(multi_thread) = parkissat_results.iter().find(|r| r.thread_count == 4) {
                let speedup = single_thread.average_time.as_secs_f64() / multi_thread.average_time.as_secs_f64();
                println!("Single-threaded (Fast):     {}", BenchmarkResult::format_time(single_thread.average_time));
                println!("Multi-threaded (4 threads): {}", BenchmarkResult::format_time(multi_thread.average_time));
                println!("Threading Speedup:          {:.2}x", speedup);
                println!();
            }
        }
    }

    fn find_optimal_configuration(&self) {
        println!("🏆 PERFORMANCE SUMMARY");
        println!("───────────────────────────────────────────────────────────");
        
        // Find fastest overall
        if let Some(fastest) = self.results.iter()
            .filter(|r| r.success)
            .min_by_key(|r| r.average_time) {
            
            println!("🥇 Fastest Configuration:");
            println!("   {:?} {:?} ({} thread{}) - {}", 
                     fastest.solver_backend,
                     fastest.optimization_level,
                     fastest.thread_count,
                     if fastest.thread_count == 1 { "" } else { "s" },
                     BenchmarkResult::format_time(fastest.average_time));
            
            // Compare with best single-threaded
            if let Some(best_single) = self.results.iter()
                .filter(|r| r.success && r.thread_count == 1)
                .min_by_key(|r| r.average_time) {
                
                if fastest.thread_count > 1 {
                    let improvement = (best_single.average_time.as_secs_f64() / fastest.average_time.as_secs_f64() - 1.0) * 100.0;
                    println!("   {:.1}% faster than best single-threaded", improvement);
                }
            }
            
            // Compare with best CaDiCaL
            if let Some(best_cadical) = self.results.iter()
                .filter(|r| r.success && r.solver_backend == SolverBackend::Cadical)
                .min_by_key(|r| r.average_time) {
                
                if fastest.solver_backend == SolverBackend::Parkissat {
                    let improvement = (best_cadical.average_time.as_secs_f64() / fastest.average_time.as_secs_f64() - 1.0) * 100.0;
                    println!("   {:.1}% faster than best CaDiCaL", improvement);
                }
            }
        }
        
        println!();
        println!("✅ Benchmark completed successfully!");
    }
}

fn main() -> Result<()> {
    let target_file = PathBuf::from("input/target_states/name.txt");
    let generations = 5;
    let runs_per_config = 1;

    // Verify target file exists
    if !target_file.exists() {
        anyhow::bail!("Target file does not exist: {}", target_file.display());
    }

    let mut benchmark = BenchmarkSuite::new(target_file, generations, runs_per_config);
    
    benchmark.run_comprehensive_benchmark()
        .context("Failed to run benchmark suite")?;
    
    benchmark.generate_report();
    
    Ok(())
}