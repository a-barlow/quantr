/*
* Copyright (c) 2023 Andrew Rowan Barlow. Licensed under the EUPL-1.2
* or later. You may obtain a copy of the licence at
* https://joinup.ec.europa.eu/collection/eupl/eupl-text-eupl-12. A copy
* of the EUPL-1.2 licence in English is given in LICENCE.txt which is
* found in the root directory of this repository.
*
* Author: Andrew Rowan Barlow <a.barlow.dev@gmail.com>
*/

// A 3 qubit circuit that implementes Grovers algorithm. The oracle target the states |110> and
// |111>. This example will also print the circuit, and show the simulaion in real time.
//
// This example will print a bin count of measured states from 500 repeated simulations, and the
// superposition itself.

use quantr::{Circuit, Measurement, Printer, StandardGate};

#[rustfmt::skip]
fn main() {
    let mut circuit = Circuit::new(3).unwrap();

    // Kick state into superposition of equal weights
    circuit
        .add_repeating_gate(StandardGate::H, &[0, 1, 2])
        .unwrap();

    // Oracle
    circuit.add_gate(StandardGate::CZ(1), 0).unwrap();

    // Amplitude amplification
    circuit
        .add_repeating_gate(StandardGate::H, &[0, 1, 2]).unwrap()
        .add_repeating_gate(StandardGate::X, &[0, 1, 2]).unwrap()
        .add_gate(StandardGate::H, 2).unwrap()
        .add_gate(StandardGate::Toffoli(0, 1), 2).unwrap()
        .add_gate(StandardGate::H, 2).unwrap()
        .add_repeating_gate(StandardGate::X, &[0, 1, 2]).unwrap()
        .add_repeating_gate(StandardGate::H, &[0, 1, 2]).unwrap();

    // Prints the circuit in UTF-8
    let mut printer = Printer::new(&circuit);
    printer.print_diagram();

    // Un-commenting the line below will print the progress of the simulation
    circuit.toggle_simulation_progress();

    // Simulates the circuit
    circuit.simulate();
    println!("");

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
