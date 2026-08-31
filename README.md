# SimTPU

**SimTPU** is a cycle-aware simulator for a configurable Tensor Processing Unit (TPU), written in Rust.

The project models a TPU-like accelerator at the hardware and ISA level, with an emphasis on **explicit hardware state, cycle-by-cycle execution, and architectural experimentation**.

> **Status:** Work in Progress

---

## Overview

SimTPU is intended to bridge the gap between high-level machine-learning frameworks and actual accelerator execution.

Rather than treating a neural network as a sequence of abstract tensor operations, SimTPU models the underlying hardware and instruction stream:

- instruction fetch, decode, and execution;
- configurable processing-element arrays;
- data movement and synchronization;
- fixed-width arithmetic;
- explicit hardware state;
- cycle-aware execution.

The ISA and microarchitecture are both under active development.

---

## Architecture

At a high level, the simulator consists of:

```text
                  ┌───────────────────┐
                  │      Program      │
                  │      / ISA        │
                  └─────────┬─────────┘
                            │
                            ▼
                  ┌───────────────────┐
                  │   TPU / Control   │
                  │ Instruction Exec. │
                  └─────────┬─────────┘
                            │
                            ▼
                  ┌───────────────────┐
                  │       MMU         │
                  │  PE Array / MACs  │
                  └─────────┬─────────┘
                            │
                            ▼
                  ┌───────────────────┐
                  │  Memory / State   │
                  └───────────────────┘
```

Detailed diagrams and explanations of individual hardware components are maintained in [`diagrams/`](diagrams/).

> **TODO:** Add a top-level architecture diagram here.

---

## ISA

SimTPU includes a preliminary instruction set for controlling the simulated hardware.

The ISA is designed around:

- strongly typed instructions;
- explicit encoding and decoding;
- deterministic execution;
- extensibility;
- a clear hardware/software boundary.

The ISA is currently experimental and will evolve alongside the architecture.

---

## Current Workload

The first complete workload being developed is a **two-layer ReLU neural network for XOR classification**.

The XOR workload is primarily an end-to-end integration test intended to exercise:

```text
ISA
 │
 ▼
Instruction Execution
 │
 ▼
MMU / PE Array
 │
 ▼
Matrix Operations
 │
 ▼
ReLU / Layer Processing
 │
 ▼
Inference Result
```

The goal is to execute the network entirely through the simulated TPU rather than evaluating it through a conventional high-level ML framework.

---

## Configuration

SimTPU is designed to support configurable hardware parameters, including:

- PE array dimensions;
- activation width;
- weight width;
- partial-sum width;
- other architectural parameters.

This allows different hardware configurations to be explored without rewriting the simulator.

---

## Repository Structure

```text
SimTPU/
├── config/              # Hardware / simulator configuration
├── diagrams/            # Architecture and hardware diagrams
├── src/                 # Simulator implementation
├── .github/             # CI / GitHub Actions
├── build.rs             # Build-time configuration
├── Cargo.toml
└── README.md
```

---

## Building

```bash
git clone https://github.com/kristopherpaul/SimTPU.git
cd SimTPU

cargo build
cargo test
```

> **TODO:** Document the simulator invocation and workload format once the CLI is finalized.

---

## Roadmap

SimTPU has already progressed beyond the initial hardware-modeling stage. The simulator currently includes a preliminary ISA, configurable hardware components, cycle-aware execution infrastructure, and the foundations required to run complete workloads.

### Completed

* [x] Core TPU simulator infrastructure
* [x] Cycle-aware hardware simulation
* [x] Processing element (PE) model
* [x] MMU / PE-array simulation
* [x] Configurable numeric widths and generated hardware configuration
* [x] Preliminary TPU ISA
* [x] Instruction encoding / decoding
* [x] ISA codec infrastructure and error handling
* [x] Initial hardware-module unit tests
* [x] Build-time configuration generation

### In Progress

* [ ] Complete machine-code execution pipeline
* [ ] Machine code → microcode translation
* [ ] End-to-end matrix multiplication
* [ ] Two-layer ReLU XOR inference
* [ ] Cycle-by-cycle execution tracing

### Planned

* [ ] ISA expansion and stabilization
* [ ] Additional TPU workloads and benchmarks
* [ ] Architectural experimentation across hardware configurations
* [ ] More detailed simulator introspection and debugging tools


---

## Goals

SimTPU is ultimately intended as a platform for experimenting with:

- TPU and accelerator architecture;
- systolic / matrix computation;
- hardware/software interfaces;
- ISA design;
- cycle-level simulation;
- data movement and scheduling;
- numerical precision;
- architectural trade-offs.

---

## License

SimTPU is distributed under a custom permissive license.

The software may be used, modified, and distributed for personal, academic, research, and commercial purposes, **but may not be used to train, fine-tune, distill, or otherwise improve a large language model**.

See [`LICENSE`](LICENSE) for the complete terms.