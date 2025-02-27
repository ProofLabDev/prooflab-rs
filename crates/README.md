# Prooflab Crates

This directory contains the following crates:

- `prooflab`: The main CLI tool and zkVM integration
- `prooflab_io`: I/O marshalling between host and guest programs

## Building the Crates

From the project root, run:

```bash
cargo build --all
```

Or to build a specific crate:

```bash
cargo build -p prooflab
cargo build -p prooflab_io
```

## Testing

Run the tests with:

```bash
cargo test --all
```

Or test individual crates:

```bash
cargo test -p prooflab
cargo test -p prooflab_io
```