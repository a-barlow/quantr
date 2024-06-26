/*
* Copyright (c) 2024 Andrew Rowan Barlow. Licensed under the EUPL-1.2
* or later. You may obtain a copy of the licence at
* https://joinup.ec.europa.eu/collection/eupl/eupl-text-eupl-12. A copy
* of the EUPL-1.2 licence in English is given in LICENCE.txt which is
* found in the root directory of this repository.
*
* Author: Andrew Rowan Barlow <a.barlow.dev@gmail.com>
*/

//! Construct, simulate and measure quantum circuits.
//!
//! Initialise a new quantum circuit by using [Circuit::new] where the argument defines the number
//! of qubits. Afterwards, various methods can be called to append gates onto the circuit in columns.
//! For instance, [Circuit::add_gate] will add a single gate, whilst
//! [Circuit::add_gates_with_positions] and [Circuit::add_repeating_gate] will add multiple.
//!
//! Before committing to a simulation, the circuit can be printed to the console or exported as a
//! UTF-8 string to an external file using [Printer::print_diagram] and [Printer::save_diagram]
//! respectively. The printer is created with [Printer::new] an by passing a reference to the
//! circuit that should be printed.
//!
//! The circuit can then be simulated with [Circuit::simulate]. The progress of the simulation can
//! be printed to the terminal by calling [Circuit::toggle_simulation_progress] before simulating
//! the circuit.
//!
//! A bin count of states that are observed over a period of measurements can be performed with
//! [Circuit::repeat_measurement], where a new register is attached before each measurement. Or, the
//! explicit superposition can be retrieved using [Circuit::get_superposition].
//!
//! All errors resulting from the incorrect use of quantr are propagated by `QuantrError` and
//! `QuantrErrorConst` that implement the [std::error::Error] trait.
//!
//! More complex examples can be found in the `examples` folder within this repository.
//!
//! # Example
//! ```
//! use quantr::{Circuit, Gate, Printer, Measurement::Observable};
//!
//! let mut quantum_circuit: Circuit = Circuit::new(2).unwrap();
//!
//! quantum_circuit
//!     .add_gates(&[Gate::H, Gate::Y]).unwrap()
//!     .add_gate(Gate::CNot(0), 1).unwrap();
//!
//! let mut printer = Printer::new(&quantum_circuit);
//! printer.print_diagram();
//! // The above prints the following:
//! // ┏━━━┓     
//! // ┨ H ┠──█──
//! // ┗━━━┛  │  
//! //        │  
//! // ┏━━━┓┏━┷━┓
//! // ┨ Y ┠┨ X ┠
//! // ┗━━━┛┗━━━┛
//!
//! quantum_circuit.simulate();
//!
//! // Below prints the number of times that each state was observered
//! // over 500 measurements of superpositions.
//!
//! if let Ok(Observable(bin_count)) = quantum_circuit.repeat_measurement(500) {
//!     println!("[Observable] Bin count of observed states.");
//!     for (state, count) in bin_count {
//!         println!("|{}> observed {} times", state, count);
//!     }
//! }
//!

mod circuit;
mod complex;
mod error;

//  Make available for public use.
pub use circuit::gate::Gate;
pub use circuit::printer::Printer;
pub use circuit::states;
pub use circuit::{Circuit, Measurement};
pub use complex::{Complex, COMPLEX_ZERO};
