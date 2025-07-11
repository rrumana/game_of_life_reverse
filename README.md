# Reverse Game of Life SAT Solver

A Rust implementation that uses SAT solving to find predecessor states for Conway's Game of Life. Given a target state, this tool finds all possible states that could have led to it after a specified number of generations.

## Overview

This project solves the NP-Complete problem of reversing Conway's Game of Life by converting it into a boolean satisfiability (SAT) problem and using the CaDiCaL SAT solver to find solutions.

### Key Features

- **Multiple SAT backends**: Supports both CaDiCaL and ParKissat-RS solvers
- **Multithreaded solving**: ParKissat-RS backend provides parallel SAT solving capabilities
- **SAT-based solving**: Converts Game of Life rules into SAT constraints
- **Multiple solutions**: Finds all valid predecessor states up to a configurable limit
- **Configurable parameters**: Grid size, generations, boundary conditions, and solver options
- **Hybrid encoding**: Uses both direct and auxiliary variables for efficient constraint generation
- **Solution validation**: Verifies that found solutions correctly evolve to the target
- **Multiple output formats**: Text, JSON, and visual representations
- **Pattern analysis**: Detects known Game of Life patterns and analyzes solution quality
- **Comprehensive benchmarking**: Built-in tools to compare solver performance

## Installation

### Prerequisites

- Rust 1.70 or later
- Cargo
- ParKissat-RS (included as dependency via `parkissat-sys`)

### Building from Source

```bash
git clone <repository-url>
cd game_of_life_reverse
cargo build --release
```

## Quick Start

1. **Set up the project structure:**
   ```bash
   cargo run -- setup
   ```

2. **Solve a simple example:**
   ```bash
   cargo run -- solve --config config/examples/simple.yaml
   ```

3. **Analyze a target state:**
   ```bash
   cargo run -- analyze --target input/target_states/blinker.txt
   ```

## Usage

### Commands

#### `solve` - Find predecessor states

```bash
cargo run -- solve [OPTIONS]
```

**Options:**
- `-c, --config <FILE>`: Configuration file (default: config/default.yaml)
- `-t, --target <FILE>`: Target state file (overrides config)
- `-g, --generations <N>`: Number of generations to reverse
- `-m, --max-solutions <N>`: Maximum solutions to find
- `-o, --output <DIR>`: Output directory
- `--show-evolution`: Show complete evolution for each solution
- `-v, --verbose`: Verbose output

**Examples:**
```bash
# Basic usage
cargo run -- solve

# Custom parameters
cargo run -- solve --target input/target_states/glider.txt --generations 3 --max-solutions 5

# Verbose output with evolution
cargo run -- solve --verbose --show-evolution
```

#### `setup` - Initialize project structure

```bash
cargo run -- setup [OPTIONS]
```

**Options:**
- `-d, --directory <DIR>`: Directory to create files in (default: current)
- `-f, --force`: Force overwrite existing files

#### `validate` - Validate a solution manually

```bash
cargo run -- validate [OPTIONS]
```

**Options:**
- `-c, --config <FILE>`: Configuration file
- `-p, --predecessor <FILE>`: Predecessor state file
- `-t, --target <FILE>`: Target state file
- `--show-evolution`: Show evolution path

#### `analyze` - Analyze target state solvability

```bash
cargo run -- analyze [OPTIONS]
```

**Options:**
- `-c, --config <FILE>`: Configuration file
- `-t, --target <FILE>`: Target state file

### Configuration

Configuration is done via YAML files. The default configuration is in `config/default.yaml`:

```yaml
simulation:
  generations: 5
  boundary_condition: "dead"  # "dead", "wrap", "mirror"

solver:
  max_solutions: 10
  timeout_seconds: 300
  optimization_level: "thorough"  # "fast", "balanced", "thorough"
  backend: "cadical"  # "cadical", "parkissat"

input:
  target_state_file: "input/target_states/glider.txt"
  
output:
  format: "text"  # "text", "json", "visual"
  save_intermediate: false
  output_directory: "output/solutions"

encoding:
  symmetry_breaking: false
```

#### Solver Backends

- **CaDiCaL**: Single-threaded, highly optimized SAT solver
- **ParKissat-RS**: Multithreaded SAT solver with parallel solving capabilities

#### Optimization Levels

- **Fast**: Quick solving with minimal preprocessing
- **Balanced**: Good balance between solve time and thoroughness
- **Thorough**: Maximum preprocessing and optimization for complex problems

### Input Format

Target states are specified in text files using a simple format:
- `1` represents a living cell
- `0` represents a dead cell
- Each line represents a row of the grid

Example (`blinker.txt`):
```
000
111
000
```

## Architecture

The project is organized into several key modules:

### Core Components

- **`config`**: Configuration management and YAML parsing
- **`game_of_life`**: Grid representation, Game of Life rules, and I/O
- **`sat`**: SAT encoding, constraint generation, and solver integration
- **`reverse`**: Problem definition, solution handling, and validation
- **`utils`**: Display utilities and output formatting

### SAT Encoding Strategy

The solver uses a hybrid encoding approach:

1. **Primary Variables**: `cell(x, y, t)` - boolean variable for each cell at each time step
2. **Auxiliary Variables**: Helper variables for neighbor counts and transitions
3. **Constraints**: Game of Life rules encoded as SAT clauses

### Key Algorithms

1. **Constraint Generation**: Converts Game of Life rules into SAT clauses
2. **Variable Management**: Efficiently maps grid coordinates to SAT variables
3. **Solution Extraction**: Converts SAT solutions back to Game of Life grids
4. **Validation**: Verifies solutions by forward simulation

## Examples

### Finding Predecessors of a Blinker

```bash
# Create a blinker target state
echo -e "000\n111\n000" > input/target_states/my_blinker.txt

# Find predecessors
cargo run -- solve --target input/target_states/my_blinker.txt --generations 1
```

### Using Different Solver Backends

```bash
# Use CaDiCaL solver (default)
cargo run -- solve --config config/cadical_fast.yaml

# Use ParKissat-RS for multithreaded solving
cargo run -- solve --config config/parkissat_thorough.yaml
```

### Analyzing a Complex Pattern

```bash
# Analyze the solvability of a glider
cargo run -- analyze --target input/target_states/glider.txt
```

### Running Benchmarks

```bash
# Compare solver performance
cargo run --example benchmark_multithreaded

# Test solver backends
cargo run --example solver_demo
```

## Performance Considerations

### Problem Complexity

The complexity of the SAT problem grows with:
- Grid size (quadratically)
- Number of generations (linearly)
- Use of auxiliary variables (increases variables but may improve solving)

### Optimization Tips

1. **Choose the right solver**: Use ParKissat-RS for complex problems that benefit from parallelization
2. **Start small**: Begin with small grids (5x5 to 10x10) and few generations
3. **Use appropriate optimization**: Set `optimization_level: "fast"` for quicker results, "thorough" for complex problems
4. **Limit solutions**: Set a reasonable `max_solutions` limit
5. **Monitor memory**: Large problems can consume significant memory

### Solver Performance Comparison

| Solver Backend | Threading | Best For | Typical Use Case |
|----------------|-----------|----------|------------------|
| CaDiCaL        | Single    | Small-medium problems | Quick solving, simple patterns |
| ParKissat-RS   | Multi     | Large/complex problems | Complex patterns, many generations |

### Expected Performance

| Grid Size | Generations | CaDiCaL Time | ParKissat-RS Time |
|-----------|-------------|--------------|-------------------|
| 5x5       | 1-2         | < 1 second   | < 1 second        |
| 10x10     | 1-3         | 1-30 seconds | 1-15 seconds      |
| 20x20     | 1-5         | 30s - 5 min | 15s - 2 min       |
| 30x30     | 1-3         | 5-30 minutes | 2-15 minutes      |


## Troubleshooting

### Common Issues

1. **No solutions found**:
   - Check if the target state is reachable
   - Try reducing the number of generations
   - Verify the target state format

2. **Solver timeout**:
   - Increase `timeout_seconds` in config
   - Reduce grid size or generations
   - Use "fast" optimization level

### Debug Mode

Run with verbose output to see detailed information:
```bash
cargo run -- solve --verbose
```

## Contributing

1. Fork the repository
2. Create a feature branch
3. Make your changes
4. Add tests for new functionality
5. Run the test suite: `cargo test`
6. Submit a pull request

## Testing

Run the test suite:
```bash
# Run all tests
cargo test

# Run tests with output
cargo test -- --nocapture

# Run specific module tests
cargo test game_of_life
cargo test sat

# Run examples
cargo run --example solver_demo
cargo run --example benchmark_multithreaded
```

## License

This project is licensed under the MIT License - see the LICENSE file for details.

## References

- [Conway's Game of Life](https://en.wikipedia.org/wiki/Conway%27s_Game_of_Life)
- [Boolean Satisfiability Problem](https://en.wikipedia.org/wiki/Boolean_satisfiability_problem)
- [CaDiCaL SAT Solver](https://github.com/arminbiere/cadical)
- [ParKissat-RS](https://github.com/rrumana/parkissat-rs) - My own Rust bindings for ParKissat
- [SAT Solving in Practice](https://www.satcompetition.org/)

## Acknowledgments

- John Conway for creating the Game of Life
- The SAT solving community for developing efficient solvers
- The ParKissat team for the multithreaded SAT solver
- The Rust community for excellent tooling and libraries