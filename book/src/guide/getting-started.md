# Getting Started

## Introduction to Roxi Development

Roxi is designed to feel native to Rust developers. Because it is packaged as a library crate, there are no database servers to initialize, no docker containers to configure for local development, and no network sockets to manage. Instead, you simply declare it as a dependency in your `Cargo.toml` and instantiate a memory-backed graph database directly inside your application thread.

This embedded architecture is perfect for:
* **WebAssembly Browser Apps**: Embed an RDF reasoning engine directly in the frontend.
* **Command Line Interfaces**: Read Turtle or N3 files, run queries, and output results.
* **Microservices**: Spin up fast, isolated, in-memory graph stores for temporary query execution.
* **CI/CD Testing**: Spin up lightweight test beds for validating data shapes.

---

## Creating Your First Project

To get started, initialize a new Rust binary project:

```bash
cargo new roxi_quickstart --bin
cd roxi_quickstart
```

Your directory structure will look like this:

```
roxi_quickstart/
├── Cargo.toml
└── src/
    └── main.rs
```

Next, open the `Cargo.toml` and configure the dependencies block. In the next section, we will cover the installation options and the specific feature gates available in Roxi.
