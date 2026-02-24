# Martial Lang

A domain-specific language (DSL) for modeling martial arts positional systems and pedagogical sequences. Define roles (structural positions like Top/Bottom), states (configurations), and sequences (step-by-step progressions) to create structured martial arts knowledge systems.

## Features

- **Simple Syntax**: Terraform-style multi-file declarations with clear role, state, and sequence definitions
- **Cross-file Validation**: Automatically validates references across all `.martial` files in a directory
- **Graph Analysis**: Visualizes state transitions and analyzes reachability
- **Multiple Export Formats**: JSON and DOT (Graphviz) output for integration with other tools
- **Single Binary**: No dependencies, cross-platform support (Linux, macOS, Windows)

## Installation

### From Release Binaries (Recommended)

Download the latest release for your platform from the [Releases](https://github.com/YOUR_USERNAME/martial-lang/releases) page:

```bash
# Linux x86_64
curl -L https://github.com/YOUR_USERNAME/martial-lang/releases/latest/download/mat-linux-x86_64.tar.gz | tar xz
sudo mv mat /usr/local/bin/

# macOS (Intel)
curl -L https://github.com/YOUR_USERNAME/martial-lang/releases/latest/download/mat-macos-x86_64.tar.gz | tar xz
sudo mv mat /usr/local/bin/

# macOS (Apple Silicon)
curl -L https://github.com/YOUR_USERNAME/martial-lang/releases/latest/download/mat-macos-aarch64.tar.gz | tar xz
sudo mv mat /usr/local/bin/

# Windows (download .exe from releases page and add to PATH)
```

### From Source

Requires Rust 1.93.1 or later:

```bash
git clone https://github.com/YOUR_USERNAME/martial-lang.git
cd martial-lang
make release
make install  # Installs to ~/.local/bin
```

Or using cargo directly:

```bash
cargo build --release
# Binary will be at target/release/mat
```

## Quick Start

Create a directory for your martial arts system with `.martial` files:

```bash
mkdir my-system
cd my-system
```

### Define Roles (`roles.martial`)

```
roles {
    Top, Bottom, Neutral
}
```

### Define States (`states.martial`)

```
state Standing

state ClosedGuard roles {
    Top, Bottom
}

state Mount roles {
    Top, Bottom
}
```

### Define Sequences (`sequences.martial`)

```
sequence GuardPass:
    BreakPosture: Standing[Neutral] -> ClosedGuard[Top]
    PassGuard: ClosedGuard[Top] -> Mount[Top]
```

### Validate Your System

```bash
mat validate my-system
```

## Commands

### `mat validate <directory>`

Validates all `.martial` files in the directory:

```bash
mat validate examples/bjj-basic
# ✓ System 'bjj-basic' is valid!
```

Checks for:
- Syntax errors
- Undefined state or role references
- Invalid role constraints
- Broken sequence chains (where step N's end state ≠ step N+1's start state)

### `mat graph <directory>`

Outputs a JSON representation of the state transition graph:

```bash
mat graph examples/bjj-basic > bjj-graph.json
```

JSON structure:
```json
{
  "system_name": "bjj-basic",
  "nodes": [
    {
      "state": "Standing",
      "role": "Neutral"
    },
    {
      "state": "ClosedGuard",
      "role": "Top"
    }
  ],
  "edges": [
    {
      "from": {
        "state": "Standing",
        "role": "Neutral"
      },
      "to": {
        "state": "ClosedGuard",
        "role": "Top"
      },
      "action": "Takedown",
      "sequence": "BasicTakedownToGuard"
    }
  ]
}
```

### `mat dot <directory>`

Outputs DOT format for Graphviz visualization:

```bash
mat dot examples/bjj-basic | dot -Tpng > bjj-graph.png
```

### `mat stats <directory>`

Displays system statistics:

```bash
mat stats examples/bjj-basic
```

Output:
```
Graph Statistics for 'bjj-basic':
  Nodes: 10
  Edges: 9
  Self-loops: 1

  Source nodes (no incoming edges):
    - Mount[Bottom]
    - Standing[Neutral]

  Sink nodes (no outgoing edges):
    - ClosedGuard[Bottom]
```

## Language Specification

See [spec/spec-1.0.md](spec/spec-1.0.md) for the complete language specification.

### Key Concepts

**Roles**: Structural positions in the system (e.g., `Top`, `Bottom`, `Neutral`)

```
roles {
    Top, Bottom, Neutral
}
```

**States**: Positions or configurations in the martial art

```
state ClosedGuard roles {
    Top, Bottom
}
```

**Sequences**: Step-by-step progressions through states

```
sequence BasicSweep:
    TechnicalStandup: ClosedGuard[Bottom] -> Standing[Neutral]
    Takedown: Standing[Neutral] -> Mount[Top]
```

### Validation Rules

1. **Roles Required**: Every system must declare roles
2. **Explicit States**: All states must be declared before use
3. **Valid References**: State and role references must exist
4. **Chain Connectivity**: In sequences, each step's end state must match the next step's start state
5. **Role Constraints**: States can restrict which roles are valid (if omitted, all roles are allowed)

## Examples

Two example systems are included:

- **`examples/bjj-basic/`**: Brazilian Jiu-Jitsu with guard passes and sweeps
- **`examples/muay-thai-basic/`**: Muay Thai with clinch techniques (demonstrates self-transitions)

Try them:

```bash
mat validate examples/bjj-basic
mat stats examples/bjj-basic
mat dot examples/bjj-basic | dot -Tpng > bjj.png
```

## Development

### Running Tests

```bash
make test
# or
cargo test
```

All 23 unit tests embedded in the source files:
- Lexer: 6 tests
- Parser: 6 tests  
- Semantic: 6 tests
- Graph: 5 tests

### Building

```bash
make build       # Debug build
make release     # Optimized release build
make dist        # Create distribution tarball
```

## Architecture

- **Lexer** ([src/lexer.rs](src/lexer.rs)): Hand-written tokenizer
- **Parser** ([src/parser.rs](src/parser.rs)): Recursive descent parser
- **Semantic** ([src/semantic.rs](src/semantic.rs)): Cross-file validation
- **Graph** ([src/graph.rs](src/graph.rs)): State transition graph analysis
- **AST** ([src/ast.rs](src/ast.rs)): Abstract syntax tree types
- **CLI** ([src/main.rs](src/main.rs)): Command-line interface

## License

MIT

## Contributing

Contributions welcome! Please open an issue or PR.

The tool is designed to be:
- **Easy to use**: Simple syntax, clear error messages
- **Easy to distribute**: Single binary, minimal dependencies
- **Easy to extend**: Clean architecture, comprehensive tests
