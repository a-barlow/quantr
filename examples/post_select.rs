/*
* Copyright (c) 2024 Andrew Rowan Barlow. Licensed under the EUPL-1.2
* or later. You may obtain a copy of the licence at
* https://joinup.ec.europa.eu/collection/eupl/eupl-text-eupl-12. A copy
* of the EUPL-1.2 licence in English is given in LICENCE.txt which is
* found in the root directory of this repository.
*
* Author: Andrew Rowan Barlow <a.barlow.dev@gmail.com>
*/

use quantr::{
    complex_re_array,
    states::{ProductState, Qubit, SuperPosition},
    Circuit, Gate, Measurement, Printer, QuantrError,
};

fn main() -> Result<(), QuantrError> {
    let mut qc = Circuit::new(3)?;

    qc.add_repeating_gate(Gate::H, &[0, 1, 2])?
        .add_gate(Gate::Custom(post_select, vec![], "P".to_string()), 1)?;

    let mut printer = Printer::new(&qc);
    printer.print_diagram();

    qc.set_print_progress(true);
    let simulated_qc = qc.simulate();

    if let Measurement::NonObservable(final_sup) = simulated_qc.get_state() {
        println!("\nThe final superposition is:");
        for (state, amplitude) in final_sup.into_iter() {
            println!("|{}> : {}", state, amplitude);
        }
    }

    Ok(())
}

fn post_select(state_in: ProductState) -> Option<SuperPosition> {
    let qubit: Qubit = state_in.get_qubits()[0];
    match qubit {
        Qubit::Zero => Some(SuperPosition::new_with_amplitudes_unchecked(
            &complex_re_array!(2f64.sqrt(), 0f64),
        )), // |0>, scaled for post-selection
        Qubit::One => Some(SuperPosition::new_with_amplitudes_unchecked(
            &complex_re_array!(0f64, 0f64),
        )),
    }
}
