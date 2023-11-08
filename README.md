# 🚧 quantr 🚧 

[![Crates.io](https://img.shields.io/crates/v/quantr?style=flat-square&color=%23B94700)](https://crates.io/crates/quantr)
[![Static Badge](https://img.shields.io/badge/version%20-%201.73.0%20-%20white?style=flat-square&logo=rust&color=%23B94700)](https://releases.rs/)
[![GitHub Workflow Status (with event)](https://img.shields.io/github/actions/workflow/status/a-barlow/quantr/rust.yml?style=flat-square&label=tests&color=%2349881B)](https://github.com/a-barlow/quantr/actions/workflows/rust.yml)
[![GitHub Workflow Status (with event)](https://img.shields.io/github/actions/workflow/status/a-barlow/quantr/rust_dev.yml?style=flat-square&label=tests%20(dev)&color=%2349881B)](https://github.com/a-barlow/quantr/actions/workflows/rust_dev.yml)
![docs.rs](https://img.shields.io/docsrs/quantr?style=flat-square&color=%2349881B)
![Crates.io](https://img.shields.io/crates/d/quantr?style=flat-square&color=%23009250)
![Crates.io](https://img.shields.io/crates/l/quantr?style=flat-square&label=licence&color=%23009982)

> This crate is not production ready and so should **not** be considered
> stable, nor produce correct answers. It is still under heavy
> development and requires many more optimisations. Hence, it's likely 
> that near future updates will induce breaking changes. Please 
> always check answers with 
> [other simulations](#other-quantum-computer-simulators) if you are 
> intending to use quantr for projects.  

A Rust library crate that builds, prints and simulates a quantum
computer.

This crate allows the user to build a quantum circuit by adding columns
of gates via various methods. Once the circuit has been built, then it
can be simulated, which attaches the register |00..0> resulting in a
superposition that can be measured.

For a brief example of using quantr, see the 
[quick start guide](QUICK_START.md) which walks through an
implementation of Grover's algorithm.

### Defining features

- Aimed to be accessible for beginners in Rust.
- The distinction between physical observables and non-physical
  observables is made clear; but the latter is still made possible to
  retrieve. 
- Prints the circuit diagram to the terminal, or saves it to a text
  file, as a UTF-8 string.
- Custom gates can be implemented easily by giving their explicit linear
  mappings on product states. This allows the user to avoid representing
  the gates as matrices.
- Attempts to minimise memory consumption by not using matrices nor
  sparse matrices, but instead uses functions to represent the linear
  mapping of gates.
- Only safe Rust code is used, and the only dependency is the
  [rand](https://docs.rs/rand/latest/rand/) crate and its
  sub-dependencies.

### Usage

An example of simulating and printing a two qubit circuit:

```rust
use quantr::{Circuit, StandardGate, Printer, Measurement::Observable};

fn main() {

    let mut quantum_circuit: Circuit = Circuit::new(2).unwrap();

    quantum_circuit 
        .add_gates(&[StandardGate::H, StandardGate::H]).unwrap()
        .add_gate(StandardGate::CNot(0), 1).unwrap();
    
    let mut printer = Printer::new(&quantum_circuit);
    printer.print_diagram();
    // The above prints the following:
    // ┏━━━┓     
    // ┨ H ┠──█──
    // ┗━━━┛  │  
    //        │  
    // ┏━━━┓┏━┷━┓
    // ┨ H ┠┨ X ┠
    // ┗━━━┛┗━━━┛

    quantum_circuit.simulate();

    // Below prints the number of times that each state was observered 
    // over 500 measurements of superpositions.

    if let Observable(bin_count) = quantum_circuit.repeat_measurement(500).unwrap() {
        println!("[Observable] Bin count of observed states.");
        for (state, count) in bin_count {
            println!("|{}> observed {} times", state.to_string(), count);
        }
    }

}
```

A more detailed example of using quantr is given in the [quick start
guide](QUICK_START.md).
 
### Limitations (currently)

- There is **no noise** consideration, or ability to introduce noise.
- There is **no ability to add classical wires**.
- The circuit size has an **upper bound of 50 qubits**. Although, due to
  incomplete optimisations, it's recommended that the circuit size
  should be much less.

### Conventions

The ordering of the wires labelling the product states in the
computational basis is defined as:

``` text 
|a⟩ ──── 
|b⟩ ────  ⟺ |a,b,c,⋯⟩ ≡ |a⟩⊗|b⟩⊗|c⟩⊗⋯ 
|c⟩ ────
 ⋮    ⋮
``` 

When defining a custom function that depends on the position of control
nodes to define gates (such as the CNot and Toffoli gates), it must be
defined so that the most far right state of the product state, is
assumed to be the gate that is 'activated'. In general, it is better to
assume that the custom function doesn't define control nodes, but rather 
it extends the dimension of the function's domain. 

### Documentation 

> [The Quantr Book](https://a-barlow.github.io/quantr-book/) is planned
> to serve as extended documentation to quantr, such as explaining the
> motivations behind chosen algorithms. For now, it only contains the
> start guide.

For the online code documentation, please refer to 
[crates.io](https://crates.io/crates/quantr). This can also be built and 
opened in your favourite web browser locally by cloning the project, 
moving into the directory, and running `cargo doc --open`. 

### Other quantum computer simulators 

The website [Are We Quantum Yet](https://arewequantumyet.github.io)
(checked 24/10/23) lists all things quantum computing in Rust. 

A useful and very practical simulator for learning quantum computing is
[Quirk](https://algassert.com/quirk). It's a real-time online simulator
that interfaces via drag-and-drop gates. Note that the labelling of the
states in the computational basis in Quirk is reversed when compared to
quantr's labelling of such states.

### Licence 

Quantr is licensed under the EUPL-1.2 or later. You may obtain a copy of
the licence at
<https://joinup.ec.europa.eu/collection/eupl/eupl-text-eupl-12>. A copy
of the EUPL-1.2 licence in English is given in
[LICENCE.txt](LICENCE.txt) which is found in the root of this
repository. Details of the licenses of third party software, and the
quantr project, can be found in [COPYRIGHT.txt](COPYRIGHT.txt).
 
