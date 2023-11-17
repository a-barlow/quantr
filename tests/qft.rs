/*
* Copyright (c) 2023 Andrew Rowan Barlow. Licensed under the EUPL-1.2
* or later. You may obtain a copy of the licence at
* https://joinup.ec.europa.eu/collection/eupl/eupl-text-eupl-12. A copy
* of the EUPL-1.2 licence in English is given in LICENCE.txt which is
* found in the root directory of this repository.
*
* Author: Andrew Rowan Barlow <a.barlow.dev@gmail.com>
*/

use quantr::{
    complex, complex_Im, complex_Re,
    states::{ProductState, SuperPosition},
    Circuit, Complex, Gate, Measurement, QuantrError,
};
use std::f64::consts::FRAC_1_SQRT_2;

const ERROR_MARGIN: f64 = 0.00000001f64;

#[test]
fn simple_qft() -> Result<(), QuantrError> {
    let mut qc: Circuit = Circuit::new(3)?;

    // Apply qft
    qc.add_repeating_gate(Gate::X, &[1, 2])?
        .add_gate(Gate::Custom(qft, &[0, 1], "QFT".to_string()), 2)?;

    qc.simulate();

    let correct_super = [
        complex_Re!(FRAC_1_SQRT_2 * 0.5f64),
        complex_Re!(-FRAC_1_SQRT_2 * 0.5f64),
        complex_Im!(-FRAC_1_SQRT_2 * 0.5f64),
        complex_Im!(FRAC_1_SQRT_2 * 0.5f64),
        complex!(-0.25f64, 0.25f64),
        complex!(0.25f64, -0.25f64),
        complex!(0.25f64, 0.25f64),
        complex!(-0.25f64, -0.25f64),
    ];

    if let Measurement::NonObservable(super_pos) = qc.get_superposition().unwrap() {
        compare_complex_lists_and_register(&correct_super, &super_pos);
    }

    Ok(())
}

// A QFT implementation that can be used for other circuits. Note, the output is reveresed, swap
// gates are needed.
fn qft(input_state: ProductState) -> SuperPosition {
    let qubit_num = input_state.qubits.len();
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
        .change_register(input_state.into_super_position())
        .unwrap()
        .simulate();

    if let Measurement::NonObservable(super_pos) = mini_circuit.get_superposition().unwrap() {
        super_pos.clone()
    } else {
        panic!("No superposition was simualted!");
    }
}

fn compare_complex_lists_and_register(correct_list: &[Complex<f64>], register: &SuperPosition) {
    for (i, &comp_num) in register.amplitudes.iter().enumerate() {
        // Make sure that it turns up complex
        assert!(equal_within_error(comp_num.re, correct_list[i].re));
        assert!(equal_within_error(comp_num.im, correct_list[i].im));
    }
}

fn equal_within_error(num: f64, compare_num: f64) -> bool {
    num < compare_num + ERROR_MARGIN && num > compare_num - ERROR_MARGIN
}