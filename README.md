# Schemaforge
This is a domain-specific language for lightweight database schemas.
Schemaforge compiles a schema into a bespoke, in-memory database engine,
emitting source code you can include into your project.

This project is in early development.


## What is this for?

Schemaforge is not a conventional database engine, but a declarative,
automated alternative to managing data with structs and containers. It is
meant to be lightweight enough that you might write a schema for a single
module, or a single operation, letting schemaforge manage memory, indexing,
query optimization, and constraint enforcement details while you focus on the
process you actually care about.

Persistence, network access, migration, and ad-hoc queries are all out of
scope for this tool.


## Quickstart

List passes:

```bash
cargo run -p schemaforge-cli -- list-passes
```

Run a pass (use `-` for stdin/stdout):

```bash
cargo run -p schemaforge-cli -- run-pass ast-to-schema --in fixtures/input.kdl --out -
```

## Tests

```bash
cargo test
```

