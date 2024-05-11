/*
* Copyright (c) 2024 Andrew Rowan Barlow. Licensed under the EUPL-1.2
* or later. You may obtain a copy of the licence at
* https://joinup.ec.europa.eu/collection/eupl/eupl-text-eupl-12. A copy
* of the EUPL-1.2 licence in English is given in LICENCE.txt which is
* found in the root directory of this repository.
*
* Author: Andrew Rowan Barlow <a.barlow.dev@gmail.com>
*/

// Uses the custom functions to define a Quantum Fourier Transform that can be applied to any
// circuit.
//
// To define the custom function, a new circuit is initialised and simulated.

use quantr::{
    states::{ProductState, SuperPosition},
    Circuit, Gate, Measurement, Printer, QuantrError,
};

fn main() -> Result<(), QuantrError> {
    let mut qc: Circuit = Circuit::new(3)?;

    // Apply qft
    qc.add_repeating_gate(Gate::X, &[1, 2])?
        .add_gate(Gate::Custom(qft, vec![0, 1], "QFT".to_string()), 2)?; // QFT on bits 0, 1 and 2

    let mut printer = Printer::new(&qc);
    printer.print_diagram();

    qc.toggle_simulation_progress();

    let simulated_circuit = qc.simulate();

    if let Ok(Measurement::NonObservable(final_sup)) = simulated_circuit.get_superposition() {
        println!("\nThe final superposition is:");
        for (state, amplitude) in final_sup.into_iter() {
            println!("|{}> : {}", state, amplitude);
        }
    }

    Ok(())
}

// A QFT implementation that can be used for other circuits. Note, the output is reveresed compared
// to usual conventions; swap gates are needed.
fn qft(input_state: ProductState) -> Option<SuperPosition> {
    let qubit_num = input_state.num_qubits();
    let mut mini_circuit: Circuit = Circuit::new(qubit_num).unwrap();

    for pos in 0..qubit_num {
        mini_circuit.add_gate(Gate::H, pos).unwrap();
        for k in 2..=(qubit_num - pos) {
            mini_circuit
                .add_gate(Gate::CRk(k as i32, pos + k - 1), pos)
                .unwrap();
        }
    }

    mini_circuit
        .change_register(SuperPosition::from(input_state))
        .unwrap();

    if let Ok(Measurement::NonObservable(super_pos)) = mini_circuit.simulate().get_superposition() {
        Some(super_pos.clone())
    } else {
        panic!("No superposition was simualted!");
    }
}
