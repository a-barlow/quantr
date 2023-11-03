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
    states::{ProductState, Qubit, SuperPosition},
    Circuit,
    Measurement::{NonObservable, Observable},
    StandardGate,
};
use quantr::{Complex, complex_Re};
use std::f64::consts::FRAC_1_SQRT_2;
const ERROR_MARGIN: f64 = 0.00000001f64;

#[rustfmt::skip]
#[test]
fn grovers_3qubit() {
    let mut circuit = Circuit::new(3).unwrap();

    // Kick state into superposition of equal weights
    circuit
        .add_repeating_gate(StandardGate::H, &[0, 1, 2]).unwrap();

    // Oracle
    circuit.add_gate(StandardGate::CZ(1), 2).unwrap();

    // Amplitude amplification
    circuit
        .add_repeating_gate(StandardGate::H, &[0, 1, 2]).unwrap()
        .add_repeating_gate(StandardGate::X, &[0, 1, 2]).unwrap()
        .add_gate(StandardGate::H, 2).unwrap()
        .add_gate(StandardGate::Toffoli(0, 1), 2).unwrap()
        .add_gate(StandardGate::H, 2).unwrap()
        .add_repeating_gate(StandardGate::X, &[0, 1, 2]).unwrap()
        .add_repeating_gate(StandardGate::H, &[0, 1, 2]).unwrap();

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

const CCC_NUMBER: usize = 4;
const CCCCC_NUMBER: usize = 6;

#[rustfmt::skip]
#[test]
fn x3sudoko() {
    let mut qc: Circuit = Circuit::new(10).unwrap();

    qc.add_repeating_gate(StandardGate::H, &[0, 1, 2, 3, 4, 5]).unwrap()
        .add_gate(StandardGate::X, 8).unwrap()
        .add_gate(StandardGate::X, 9).unwrap()
        .add_gate(StandardGate::H, 9).unwrap();

    // oracle building
    for i in 0..=2 {
        qc.add_gate(StandardGate::Toffoli(i, i + 3), 8).unwrap();
    }
    qc.add_gate(StandardGate::Custom(cccnot, &[0, 1, 2], "X".to_string()), 6)
        .unwrap();
    for i in 0..=2 {
        qc.add_gate(StandardGate::CNot(i), 6).unwrap();
    }
    qc.add_gate(StandardGate::Custom(cccnot, &[3, 4, 5], "X".to_string()), 7)
        .unwrap();
    for i in 3..=5 {
        qc.add_gate(StandardGate::CNot(i), 7).unwrap();
    }

    // The phase kickback
    qc.add_gate(StandardGate::Custom(cccnot, &[6, 7, 8], "X".to_string()), 9)
        .unwrap();

    // Reset by using the oracle again
    for i in 0..=2 {
        qc.add_gate(StandardGate::Toffoli(i, i + 3), 8).unwrap();
    }
    qc.add_gate(StandardGate::Custom(cccnot, &[0, 1, 2], "X".to_string()), 6)
        .unwrap();
    for i in 0..=2 {
        qc.add_gate(StandardGate::CNot(i), 6).unwrap();
    }
    qc.add_gate(StandardGate::Custom(cccnot, &[3, 4, 5], "X".to_string()), 7)
        .unwrap();
    for i in 3..=5 {
        qc.add_gate(StandardGate::CNot(i), 7).unwrap();
    }

    // Amplitude amplification
    qc.add_repeating_gate(StandardGate::H, &[0, 1, 2, 3, 4, 5]).unwrap()
        .add_repeating_gate(StandardGate::X, &[0, 1, 2, 3, 4, 5]).unwrap()
        .add_gate(StandardGate::H, 5).unwrap()
        .add_gate(StandardGate::Custom(cccccnot, &[0, 1, 2, 3, 4], "X".to_string()),5,).unwrap()
        .add_gate(StandardGate::H, 5).unwrap()
        .add_repeating_gate(StandardGate::X, &[0, 1, 2, 3, 4, 5]).unwrap()
        .add_repeating_gate(StandardGate::H, &[0, 1, 2, 3, 4, 5]).unwrap();
    // END

    qc.simulate();

    if let Observable(bin_count) = qc.repeat_measurement(5000).unwrap() {
        for (state, count) in bin_count {
            match &state.as_string()[0..=5] {
                "001100" | "001010" | "010100" | "010001" | "100010" | "100001" => {
                    assert!(count > 150usize)
                }
                _ => assert!(count < 150usize),
            }
        }
    }
}

fn cccnot(input_state: ProductState) -> SuperPosition {
    let mut copy_state = input_state.clone();
    if copy_state.state == [Qubit::One; CCC_NUMBER] {
        copy_state.state[CCC_NUMBER - 1] = Qubit::Zero;
        return copy_state.to_super_position();
    } else if copy_state.state == {
        let mut temp = [Qubit::One; CCC_NUMBER];
        temp[CCC_NUMBER - 1] = Qubit::Zero;
        temp
    } {
        copy_state.state[CCC_NUMBER - 1] = Qubit::One;
        return copy_state.to_super_position();
    } else {
        copy_state.to_super_position()
    }
}

// Implementation of a 5 controlled toffoli gate
fn cccccnot(input_state: ProductState) -> SuperPosition {
    let mut copy_state = input_state.clone();
    if copy_state.state == [Qubit::One; CCCCC_NUMBER] {
        copy_state.state[CCCCC_NUMBER - 1] = Qubit::Zero;
        return copy_state.to_super_position();
    } else if copy_state.state == {
        let mut temp = [Qubit::One; CCCCC_NUMBER];
        temp[CCCCC_NUMBER - 1] = Qubit::Zero;
        temp
    } {
        copy_state.state[CCCCC_NUMBER - 1] = Qubit::One;
        return copy_state.to_super_position();
    } else {
        copy_state.to_super_position()
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
    num < compare_num + ERROR_MARGIN && num > compare_num - ERROR_MARGIN
}
