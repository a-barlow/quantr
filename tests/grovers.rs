/*
* Copyright (c) 2023 Andrew Rowan Barlow. Licensed under the EUPL-1.2
* or later. You may obtain a copy of the licence at
* https://joinup.ec.europa.eu/collection/eupl/eupl-text-eupl-12. A copy
* of the EUPL-1.2 licence in English is given in LICENCE.txt which is
* found in the root directory of this repository.
*
* Author: Andrew Rowan Barlow <a.barlow.dev@gmail.com>
*/

use quantr::circuit::{
    states::SuperPosition,
    Circuit,
    Measurement::{NonObservable, Observable},
    StandardGate,
};
use quantr::{complex::Complex, complex_Re};
use std::f64::consts::FRAC_1_SQRT_2;
const ERROR_MARGIN: f64 = 0.00000001f64;

#[test]
fn grovers_3qubit() {
    let mut circuit = Circuit::new(3).unwrap();

    // Kick state into superposition of equal weights
    circuit
        .add_repeating_gate(StandardGate::H, vec![0, 1, 2])
        .unwrap();

    // Oracle
    circuit.add_gate(StandardGate::CZ(1), 2).unwrap();

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

    // Simulates the circuit so that the final register can be
    // calculated.
    circuit.simulate();

    let correct_super: [Complex<f64>; 8] = [
        complex_Re!(0f64),
        complex_Re!(0f64),
        complex_Re!(0f64),
        complex_Re!(-FRAC_1_SQRT_2),
        complex_Re!(0f64),
        complex_Re!(0f64),
        complex_Re!(0f64),
        complex_Re!(-FRAC_1_SQRT_2),
    ];

    if let NonObservable(output_register) = circuit.get_superposition().unwrap() {
        compare_complex_lists_and_register(&correct_super, output_register);
    }

    if let Observable(bin_count) = circuit.repeat_measurement(500).unwrap() {
        for (state, count) in bin_count {
            match state.as_string().as_str() {
                "011" | "111" => assert!(count > 200usize),
                _ => assert_eq!(count, 0usize),
            }
        }
    }
}

fn compare_complex_lists_and_register(correct_list: &[Complex<f64>], register: &SuperPosition) {
    for (i, &comp_num) in register.amplitudes.iter().enumerate() {
        // Make sure that it turns up complex
        assert!(equal_within_error(comp_num.real, correct_list[i].real));
        assert!(equal_within_error(
            comp_num.imaginary,
            correct_list[i].imaginary
        ));
    }
}

fn equal_within_error(num: f64, compare_num: f64) -> bool {
    if num < compare_num + ERROR_MARGIN && num > compare_num - ERROR_MARGIN {
        true
    } else {
        false
    }
}
