/*
* Copyright (c) 2023 Andrew Rowan Barlow. Licensed under the EUPL-1.2
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
    Circuit, Measurement, Printer, QuantrError, StandardGate,
};

fn main() -> Result<(), QuantrError> {
    let mut qc: Circuit = Circuit::new(5)?;

    // Apply qft
    qc.add_repeating_gate(StandardGate::H, &[0, 1, 2])?
        .add_gate(StandardGate::Custom(qft, &[0, 1], "QFT".to_string()), 2)? // QFT on bits 0, 1 and 2
        .add_gate(StandardGate::CNot(1), 3)?
        .add_gate(StandardGate::CNot(2), 4)?;

    let mut printer = Printer::new(&qc);
    printer.print_diagram();

    qc.simulate();

    if let Measurement::Observable(bin_count) = qc.repeat_measurement(100).unwrap() {
        for (state, count) in bin_count {
            println!("|{}> : {}", state.as_string(), count);
        }
    }

    Ok(())
}

// A QFT implementation that can be used for other circuits.
fn qft(input_state: ProductState) -> SuperPosition {
    let qubit_num = input_state.state.len();
    let mut mini_circuit: Circuit = Circuit::new(qubit_num).unwrap();

    for pos in 0..qubit_num {
        mini_circuit.add_gate(StandardGate::H, pos).unwrap();
        for k in 1..(qubit_num - pos) {
            mini_circuit
                .add_gate(StandardGate::CRk(k as i32, k), pos)
                .unwrap();
        }
    }

    mini_circuit.simulate();

    if let Measurement::NonObservable(super_pos) = mini_circuit.get_superposition().unwrap() {
        super_pos.clone()
    } else {
        panic!("No superposition was simualted!");
    }
}
