# Quick Start Guide 

This guide is aimed to someone that does not know Rust, or who have
just begun their Rust journey. Explanations of Rust won't be given here,
nor how the Grover's algorithm works. More detail can be found in [The Rust
Book](https://doc.rust-lang.org/stable/book/title-page.html) (that
offers a great starting point for learning Rust) and [Qiskit's Grover's
Algorithm](https://qiskit.org/ecosystem/algorithms/tutorials/06_grover.html) 
explanation. This start guide also assumes that [Rust and Cargo are
installed](https://doc.rust-lang.org/stable/book/ch01-00-getting-started.html)
and the user has experience with the command line. 

Create a new Rust project by entering `cargo new grovers_example`. This
will create a new directory (in your working directory). Enter this 
directory.

Add the latest version of quantr as a dependency by adding `quantr = *`
below `[dependecies]` in you `Cargo.toml` file. This notifies the
project manager, cargo, and make the library accesible. This can be
accomplished by running `cargo check` on the command line, where it'll 
show the progress of downloading quantr and the rand crate which quantr
uses.

Once quantr has been installed, open `src/main.rs`. This is where 
Grover's algorithm will be implemented.

The following is a list of imports from the quantr crate that will be
needed to implement Grover's algorithm:

```rust, ignore
use quantr::circuit::{Circuit, StandardGate, printer::Printer, state::ProductState};
use std::HashMap;
```

Now for building the circuit. Rust begins execution of the program by
calling the `main` function. So, the rest of the code will be inserted
into this fucntion. A 3 qubit circuit is initialised by:

```rust,ignore
let mut circuit: Circuit = Circuit::new(3);
```

The mutable variable `circuit` represents the new 3 qubit circuit. 

The start of Grover's algorithm requires that the state be in a
superposition of all states with equal amplitudes. This can be acheived
by adding 3 hadamard gates to each wire. This can be acheived by

```rust,ignore
circuit.add_repeating_gates(StandardGate::H, vec![0, 1, 2]).unwrap();
```

The `vec!` macro is a quick way to create a vector (a list on the heap).
The `.unwrap()` forces the program to quit if there is an error in
adding these gates, such as adding a gate on a non-existent wire. There
is no error in this example, so the method `.unwrap()` does nothing. To 
visualise the circuit currently, or at any time throughout this
process, the following can be added to print the circuit to the
terminal:

```rust,ignore
let mut printer: Printer = Printer::new(&circuit);
printer.print_diagram();
```

Next, the oracle needs to be defined. This is the function that flips
the sign of the state that corresponds to the solution of the search
function. In this example, the sign of the amplitude of state |110> and 
|111> will be flipped. This is implemented by adding a swap gate (or 
controlled-z gate) on the 0th and 1st position of the wires (the first
and second wire from top to bottom respectively).

```rust,ignore
circuit.add_gate(StandardGate::Swap(1), 0).unwrap();
```

The second argument of `circuit.add_gate` specifies which wire is the
target, and the field of the variant `StandardGate::Swap` specifies the
control wire. The swap gate is symmetric in these arguments, so in this
example it doesn't matter. But to clarify, placing a C-Not gate such
that it gives

```markdown
--*--
  |
--X--

-----
```

is acheived by calling `circuit.add_gate(StandardGate::C-Not(0),
1).unwrap();`

With the solution statse chosen, then amplitudes are amplified so that
these states are more likely to be measured than other non-solution states.
This is implemented by

```rust,ignore
circuit.add_repeating_gate(StandardGate::H, vec![0, 1, 2]).unwrap();
circuit.add_repeating_gate(StandardGate::X, vec![0, 1, 2]).unwrap();

// CC-Z gate
circuit.add_gate(StandardGate::H, 2).unwrap();
circuit.add_gate(StandardGate::Toffoli(0, 1), 2).unwrap();
circuit.add_gate(StandardGate::H, 2).unwrap();

circuit.add_repeating_gate(StandardGate::X, vec![0, 1, 2]).unwrap();
circuit.add_repeating_gate(StandardGate::H, vec![0, 1, 2]).unwrap();
``` 

This completes the construction of Grover's algorithm. To make sure that
the gates are placed correctly, run the printing code as shown before.
Once checked that it's correct, the circuit can be simulated by calling

```rust,ignore
circuit.simulate();
```

This effectively attatches the register to the circuit, resulting in a
super position that can be measured. As per quantum mechanical rules,
the superposition can be prepared and measured multiple times to collect
a bin count of observed states. This bin count can be found and printed 
with

```rust,ignore
if let Measurement::Observable(bin_count) = circuit.repeat_measurement(500).unwrap() {
    // bin_count is a HashMap<ProductState, usize>
    for (state, count) in bin_count {
        println!("{} : {}", state.as_str(), count);
    }
}
```

The above prints the number of times the state was observed over a 500
measurements. In this situtation, the amplitude amplification results in
a superposition of two states: |110> and |111>.

Note in the code above, that the code is explicit in showing that the
bin count is a physical observable. This is to distinguish from other
data that can be taken from circuit, such as viewing the resulting super
position directly. This cannot be a direct observable as this is
forbidden by the laws of quantum mechanics. To get the explicit 
superposition:

```rust,ignore
if let Measurement::NonObservable(output_super_position) = circuit.get_superposition().unwrap() 
{
    for (state, amplitude) in super_position.into_iter() {
        println!("{} : {}", state.as_string(), amplitude);
    }
}
```

This completes the construction and measurment of a 3 qubit Grovers
circuit. Other functions with examples that add gates in various ways 
can be found in the code documentation. 

The following is a the completed code that can be copied
directly to your `main.rs` and ran with `cargo run` on the command line.
Make sure that quantr is added as a dependency.

```rust,ignore
use quantr::circuit::{printer::Printer, Circuit, Measurement, StandardGate};

fn main() {
    let mut circuit = Circuit::new(3);

    // Kick state into superposition of equal weights
    circuit
        .add_repeating_gate(StandardGate::H, vec![0, 1, 2])
        .unwrap();

    // Oracle
    circuit.add_gate(StandardGate::CZ(0), 1).unwrap();

    // Amplitude amplification
    circuit
        .add_repeating_gate(StandardGate::H, vec![0, 1, 2])
        .unwrap();
    circuit
        .add_repeating_gate(StandardGate::X, vec![0, 1, 2])
        .unwrap();

    circuit.add_gate(StandardGate::H, 2).unwrap();
    circuit.add_gate(StandardGate::Toffoli(0, 1), 2).unwrap();
    circuit.add_gate(StandardGate::H, 2).unwrap();

    circuit
        .add_repeating_gate(StandardGate::X, vec![0, 1, 2])
        .unwrap();
    circuit
        .add_repeating_gate(StandardGate::H, vec![0, 1, 2])
        .unwrap();

    // Prints the circuit in UTF-8
    let mut printer = Printer::new(&circuit);
    printer.print_diagram();

    // Simulates the circuit
    circuit.simulate();

    // Displays bin count of the resulting 500 repeat measurements of
    // superpositions. bin_count is a HashMap<ProductState, usize>.
    if let Measurement::Observable(bin_count) = circuit.repeat_measurement(500).unwrap() {
        println!("[Observable] Bin count of observed states.");
        for (state, count) in bin_count {
            println!("|{}> observed {} times", state.as_string(), count);
        }
    }

    // Returns the superpsoition that cannot be directly observed.
    if let Measurement::NonObservable(output_super_position) = circuit.get_superposition().unwrap()
    {
        println!("\n[Non-Observable] The amplitudes of each state in the final superposition.");
        for (state, amplitude) in output_super_position.into_iter() {
            println!("|{}> : {}", state.as_string(), amplitude);
        }
    }
}
```
