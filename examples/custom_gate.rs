/*
* Copyright (c) 2023 Andrew Rowan Barlow. Licensed under the EUPL-1.2
* or later. You may obtain a copy of the licence at
* https://joinup.ec.europa.eu/collection/eupl/eupl-text-eupl-12. A copy
* of the EUPL-1.2 licence in English is given in LICENCE.txt which is
* found in the root directory of this repository.
*
* Author: Andrew Rowan Barlow <a.barlow.dev@gmail.com>
*/

// Shows the use of `StandardGate::Custom` in implementing the CCC-not gate.

use quantr::{
    states::{ProductState, Qubit, SuperPosition},
    Circuit, Measurement, Printer, QuantrError, StandardGate,
};

fn main() -> Result<(), QuantrError> {
    let mut qc: Circuit = Circuit::new(4)?;

    // Build a circuit using a CCC-not gate, placing the control nodes on positions 0, 1, 2 and
    // the target on 3.
    qc.add_repeating_gate(StandardGate::X, &[0, 1, 2])?
        .add_gate(StandardGate::Custom(cccnot, &[0, 1, 2], "X".to_string()), 3)?;

    // Prints the circuit, viewing the custom gate, and then simulating it.
    let mut circuit_printer: Printer = Printer::new(&qc);
    circuit_printer.print_diagram();

    qc.toggle_simulation_progress(); // prints the simulation toggle_simulation_progress
    qc.simulate();

    // Prints the bin count of measured states.
    if let Measurement::Observable(bin_count) = qc.repeat_measurement(50).unwrap() {
        for (states, count) in bin_count.into_iter() {
            println!("|{}> : {}", states.to_string(), count);
        }
    }

    Ok(())
}

// Implements the CCC-not gate.
fn cccnot(input_state: ProductState) -> SuperPosition {
    let state: Vec<Qubit> = input_state.qubits;
    let state_slice: [Qubit; 4] = [state[0], state[1], state[2], state[3]]; // In this format, this
                                                                            // guarantees that state_slice has length 4 to the rust compiler. Useful for the match
                                                                            // statement.
    match state_slice {
        [Qubit::One, Qubit::One, Qubit::One, Qubit::Zero] => {
            ProductState::new(&[Qubit::One; 4]).into_super_position()
        }
        [Qubit::One, Qubit::One, Qubit::One, Qubit::One] => {
            ProductState::new(&[Qubit::One, Qubit::One, Qubit::One, Qubit::Zero])
                .into_super_position()
        }
        other_state => ProductState::new(&other_state).into_super_position(),
    }
}
