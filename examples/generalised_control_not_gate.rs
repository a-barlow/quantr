/*
* Copyright (c) 2024 Andrew Rowan Barlow. Licensed under the EUPL-1.2
* or later. You may obtain a copy of the licence at
* https://joinup.ec.europa.eu/collection/eupl/eupl-text-eupl-12. A copy
* of the EUPL-1.2 licence in English is given in LICENCE.txt which is
* found in the root directory of this repository.
*
* Author: Andrew Rowan Barlow <a.barlow.dev@gmail.com>
*/

//! This example is a copy of `example/custom_gate.rs`, but instead uses a custom function that
//! showcases a controlled not gate which generalises the number of control nodes.

use quantr::{
    states::{ProductState, Qubit, SuperPosition},
    Circuit, Gate, Measurement, Printer, QuantrError,
};

const CIRCUIT_SIZE: usize = 6;

fn main() -> Result<(), QuantrError> {
    let mut qc: Circuit = Circuit::new(CIRCUIT_SIZE)?;

    // Multi-controlled gate used here.
    qc.add_repeating_gate(Gate::X, &[0, 1, 2, 3, 4, 5])?
        .add_gate(
            Gate::Custom(
                multicnot::<CIRCUIT_SIZE>,
                vec![0, 1, 2, 3, 4],
                "X".to_string(),
            ),
            5,
        )?;

    let mut circuit_printer: Printer = Printer::new(&qc);
    circuit_printer.print_diagram();

    qc.toggle_simulation_progress();
    let simulated = qc.simulate();

    // Prints the bin count of measured states.
    if let Measurement::Observable(bin_count) = simulated.measure_all(50) {
        println!("\nStates observed over 50 measurements:");
        for (states, count) in bin_count.into_iter() {
            println!("|{}> : {}", states, count);
        }
    }

    Ok(())
}

// Implements a multi-controlled Not gate.
fn multicnot<const NUM_CONTROL: usize>(input_state: ProductState) -> Option<SuperPosition> {
    let mut copy_state = input_state;
    if copy_state.get_qubits() == [Qubit::One; NUM_CONTROL] {
        copy_state.get_mut_qubits()[NUM_CONTROL - 1] = Qubit::Zero;
        return Some(copy_state.into());
    } else if copy_state.get_qubits() == {
        let mut temp = [Qubit::One; NUM_CONTROL];
        temp[NUM_CONTROL - 1] = Qubit::Zero;
        temp
    } {
        copy_state.get_mut_qubits()[NUM_CONTROL - 1] = Qubit::One;
        return Some(copy_state.into());
    } else {
        None
    }
}
