/*
* Copyright (c) 2023 Andrew Rowan Barlow. Licensed under the EUPL-1.2
* or later. You may obtain a copy of the licence at
* https://joinup.ec.europa.eu/collection/eupl/eupl-text-eupl-12. A copy
* of the EUPL-1.2 licence in English is given in LICENCE.txt which is
* found in the root directory of this repository.
*
* Author: Andrew Rowan Barlow <a.barlow.dev@gmail.com>
*/

// Shows the use of `Gate::Custom` in implementing the CCC-not gate.

use std::error::Error;

use quantr::{
    states::{ProductState, Qubit, SuperPosition},
    Circuit, Gate, Measurement, Printer,
};

fn main() -> Result<(), Box<dyn Error>> {
    let mut qc: Circuit = Circuit::new(4)?;

    // Build a circuit using a CCC-not gate, placing the control nodes on positions 0, 1, 2 and
    // the target on 3.
    qc.add_repeating_gate(Gate::X, &[0, 1, 2])?
        .add_gate(Gate::Custom(cccnot, &[0, 1, 2], "X".to_string()), 3)?;

    // Prints the circuit, viewing the custom gate, and then simulating it.
    let mut circuit_printer: Printer = Printer::new(&qc);
    circuit_printer.print_diagram();

    qc.toggle_simulation_progress(); // prints the simulation toggle_simulation_progress
    qc.simulate();

    // Prints the bin count of measured states.
    if let Ok(Measurement::Observable(bin_count)) = qc.repeat_measurement(50) {
        println!("\nStates observed over 50 measurements:");
        for (states, count) in bin_count.into_iter() {
            println!("|{}> : {}", states.to_string(), count);
        }
    }

    Ok(())
}

// Implements the CCC-not gate.
fn cccnot(input_state: ProductState) -> Option<SuperPosition> {
    let state: &[Qubit] = input_state.get_qubits();
    let state_slice: [Qubit; 4] = [state[0], state[1], state[2], state[3]]; // In this format, this
                                                                            // guarantees that state_slice has length 4 to the rust compiler. Useful for the match
                                                                            // statement.
    match state_slice {
        [Qubit::One, Qubit::One, Qubit::One, Qubit::Zero] => {
            Some(ProductState::new(&[Qubit::One; 4]).unwrap().into())
        }
        [Qubit::One, Qubit::One, Qubit::One, Qubit::One] => Some(
            ProductState::new(&[Qubit::One, Qubit::One, Qubit::One, Qubit::Zero])
                .unwrap()
                .into(),
        ),
        _ => return None,
    }
}
