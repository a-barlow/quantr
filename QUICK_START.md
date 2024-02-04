# Quick Start Guide 

This guide walks through an implementation of Grover's algorithm using
[quantr](https://crates.io/crates/quantr) 0.5.0. It's aimed at beginners in
Rust and requires a little knowledge of the console. Moreover,
it's assumed that [Rust and Cargo are
installed](https://doc.rust-lang.org/stable/book/ch01-00-getting-started.html).

A good starting point to learn Rust is [The Rust
Book](https://doc.rust-lang.org/stable/book/title-page.html). Likewise,
[Qiskit](https://qiskit.org/ecosystem/algorithms/tutorials/06_grover.html)
offers a good explanation of Grover's. This guide does not attempt to
explain either of these subjects in detail.

The complete code that is built from following this guide is available
at the end. Or, the code can be found in `examples/grovers.rs`, and ran
from the root directory with `cargo run --example grovers`.

---

Open the console, and create a new Rust project (known as a cargo
package) by running

``` console
cargo new grovers_example
```

This will create a new directory called `grovers_example` containing
the necessary files for a Rust project. Enter this directory.

Add the latest version of quantr as a dependency by running `cargo add
quantr` on the console. This should add quantr below `[dependecies]` in
you `Cargo.toml` file. Then, run `cargo build`. This will download the
quantr crate from [crates.io](https://crates.io/) and make it
accessible for your project. 

Once quantr has been installed, open `src/main.rs`. This is where the
subsequent code will be written to implement Grover's algorithm.

Add the following lines to the top of `main.rs`, and before `fn main()`:

```rust, ignore
use quantr::{Circuit, Gate, Measurement, Printer};
```

These lines import the structs and enums that will be used throughout 
this guide.

Rust begins execution of the program by calling the `main()` function.
So, the rest of the code will be inserted into this function. Let's
first initialise a three qubit circuit:

```rust,ignore
let mut circuit: Circuit = Circuit::new(3).unwrap();
```

Grover's algorithm requires that the starting state is in a 
superposition of all basis states with equal amplitudes. This can be
achieved by adding a Hadamard gate to each wire, that is to wires 0, 1
and 2:

```rust,ignore
circuit.add_repeating_gate(Gate::H, &[0, 1, 2]).unwrap();
```

The `.unwrap()` forces the program to quit if there is an error in
adding these gates, such as adding a gate to a non-existent wire. There
are no errors in this example, so the method `.unwrap()` does nothing. 

Let's visualise the circuit that has been built so far (or at any other
time throughout this guide) by adding the following:

```rust,ignore
let mut printer: Printer = Printer::new(&circuit);
printer.print_diagram();
```

This will print a circuit diagram to the console. This can be seen by
running the program by entering `cargo run` while in the directory that
cargo built.

The next step in the algorithm requires the oracle to be defined. This
is a function built from unitary gates that flips the sign of the state
that corresponds to a 'solution' of the searching function. In this
example, the states |110> and |111> are chosen to be the solution
states.

This is implemented by adding a controlled-z gate targeting the 0th
wire, with its control node placed on the 1st wire; the first and second
wire from top to bottom in a circuit diagram respectively:

```rust,ignore
circuit.add_gate(Gate::CZ(1), 0).unwrap();
```

The second argument of `circuit.add_gate` specifies which wire is the
target, and the field of the variant `Gate::CZ` specifies the
control wire.

With the solution states marked, these amplitudes are amplified so
that solution states are more likely to be measured than other non-solution
states. This can be achieved by adding:

```rust,ignore
circuit.add_repeating_gate(Gate::H, &[0, 1, 2]).unwrap()
    .add_repeating_gate(Gate::X, &[0, 1, 2]).unwrap();

// CC-Z gate
circuit.add_gate(Gate::H, 2).unwrap()
    .add_gate(Gate::Toffoli(0, 1), 2).unwrap()
    .add_gate(Gate::H, 2).unwrap();

circuit.add_repeating_gate(Gate::X, &[0, 1, 2]).unwrap()
    .add_repeating_gate(Gate::H, &[0, 1, 2]).unwrap();
```

This completes the construction of Grover's algorithm. To make sure that
the gates are placed correctly, run the printing code as shown before.
Once checked that it's correct, the circuit can be simulated by calling

```rust,ignore
circuit.simulate();
```

This effectively attaches the |000> register to the circuit, resulting
in a superposition that can be measured. As usual, the superposition can
be prepared and measured multiple times to collect a bin count of
observed states. This bin count can be found and printed with

```rust,ignore
if let Ok(Measurement::Observable(bin_count)) = circuit.repeat_measurement(500) {
    // bin_count is a HashMap<ProductState, usize>
    for (state, count) in bin_count {
        println!("{} : {}", state.to_string(), count);
    }
}
```

The above prints the number of times each state was observed over 500
measurements. In this situation, the amplitude amplification results in
a superposition of two states: |110> and |111>.
    
Note that the above code is explicit in showing that the measurements
are *physically possible*. This is to distinguish from other data that
can be taken from circuit, such as the resulting superposition itself.
In nature, this cannot be directly observed. However, it can still be
useful to view this "theoretical" superposition:

```rust,ignore
if let Ok(Measurement::NonObservable(output_super_position)) = circuit.get_superposition() 
{
    for (state, amplitude) in super_position.into_iter() {
        println!("{} : {}", state.to_string(), amplitude);
    }
}
```

This completes the construction and measurement of a three qubit
Grover's circuit. Other functions (which include examples in their
documentation) can add gates in other ways. Moreover, custom gates can
be built, which can be seen in examples `examples/qft.rs` and
`examples/custom_gate.rs`.

To improve the readability of this code, the numerous `unwrap()` calls
can be removed while the main function declaration can be edited like
so:

```rust,ignore 
...
use std::error::Error;

fn main() -> Result<(), Box<dyn Error>> {
    ...; 
    Ok(()) 
}
```

A `Ok(())` is returned on the last line; signalling that the program has
exited without errors. Then, effectively all unwrap methods called after
appending gates can be replaced with a `?`. This can be seen explicitly
in the `example/grovers.rs` folder.

The following is the completed code. This can be ran with `cargo run
--example grovers` from the root directory.

```rust
use quantr::{Circuit, Gate, Measurement, Printer};

fn main() {
    let mut circuit = Circuit::new(3).unwrap();

    // Kick state into superposition of equal weights
    circuit
        .add_repeating_gate(Gate::H, &[0, 1, 2])
        .unwrap();

    // Oracle
    circuit.add_gate(Gate::CZ(1), 0).unwrap();

    // Amplitude amplification
    circuit
        .add_repeating_gate(Gate::H, &[0, 1, 2])
        .unwrap()
        .add_repeating_gate(Gate::X, &[0, 1, 2])
        .unwrap();

    circuit.add_gate(Gate::H, 2).unwrap()
        .add_gate(Gate::Toffoli(0, 1), 2).unwrap()
        .add_gate(Gate::H, 2).unwrap();

    circuit
        .add_repeating_gate(Gate::X, &[0, 1, 2])
        .unwrap()
        .add_repeating_gate(Gate::H, &[0, 1, 2])
        .unwrap();

    // Prints the circuit in UTF-8
    let mut printer = Printer::new(&circuit);
    printer.print_diagram();

    // Un-commenting the line below will print the progress of the simulation
    // circuit.toggle_simulation_progress();

    // Simulates the circuit
    circuit.simulate();

    // Displays bin count of the resulting 500 repeat measurements of
    // superpositions. bin_count is a HashMap<ProductState, usize>.
    if let Ok(Measurement::Observable(bin_count)) = circuit.repeat_measurement(500) {
        println!("\n[Observable] Bin count of observed states.");
        for (state, count) in bin_count {
            println!("|{}> observed {} times", state.to_string(), count);
        }
    }

    // Returns the superpsoition that cannot be directly observed.
    if let Ok(Measurement::NonObservable(output_super_position)) = circuit.get_superposition()
    {
        println!("\n[Non-Observable] The amplitudes of each state in the final superposition.");
        for (state, amplitude) in output_super_position.into_iter() {
            println!("|{}> : {}", state.to_string(), amplitude);
        }
    }
}
```
