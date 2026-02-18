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


## Usage

To build, simply `make`, then look for `build/schemaforge`.

Invoke `schemaforge <FILE>` to process a schema file.


