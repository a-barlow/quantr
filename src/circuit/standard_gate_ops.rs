/*
* Copyright (c) 2023 Andrew Rowan Barlow. Licensed under the EUPL-1.2
* or later. You may obtain a copy of the licence at
* https://joinup.ec.europa.eu/collection/eupl/eupl-text-eupl-12. A copy
* of the EUPL-1.2 licence in English is given in LICENCE.txt which is
* found in the root directory of this repository.
*
* Author: Andrew Rowan Barlow <a.barlow.dev@gmail.com>
*/

//! Contains the gate operations for all standard gates.
//!
//! These linear functions are defined by how they act on product states of qubits. Defining the
//! mappings on a basis defines how the gates act on larger product spaces.

use crate::circuit::states::{ProductState, Qubit, SuperPosition};
use crate::complex::Complex;
use crate::{complex, complex_Im, complex_Im_array, complex_Re, complex_Re_array, complex_zero};
use std::f64::consts::FRAC_1_SQRT_2;
use std::ops::{Div, Mul};

// The following gates (inlcuding triple and custom) are mapping qubits via the
// computational basis:
// |a> ----
// |b> ----
// |c> ----
// => |a,b,c>
//
// `cargo fmt` has also been skipped as this shows the connection between matrices (which are
// transposed) in the computational basis, and linear maps!

//
// Single gates
//

#[rustfmt::skip]
pub fn identity(register: Qubit) -> SuperPosition {
    SuperPosition::new(1).set_amplitudes_unchecked(match register {
        Qubit::Zero => &[complex_Re!(1f64), complex_zero!()],
        Qubit::One => &[complex_zero!(), complex_Re!(1f64)],
    }).unwrap()
}

#[rustfmt::skip]
pub fn hadamard(register: Qubit) -> SuperPosition {
    SuperPosition::new(1).set_amplitudes_unchecked(match register {
        Qubit::Zero => &complex_Re_array!(FRAC_1_SQRT_2, FRAC_1_SQRT_2),
        Qubit::One => &complex_Re_array!(FRAC_1_SQRT_2, -FRAC_1_SQRT_2),
    }).unwrap()
}

#[rustfmt::skip]
pub fn rx(register: Qubit, angle: f64) -> SuperPosition {
    let real_parts: Complex<f64> = complex_Re!((0.5f64.mul(angle)).cos());
    let imaginary_part: Complex<f64> = complex_Im!(-(0.5f64.mul(angle)).sin());
    let zero_map: [Complex<f64>; 2] = [real_parts, imaginary_part];
    let one_map: [Complex<f64>; 2] = [imaginary_part, real_parts];

    SuperPosition::new(1).set_amplitudes_unchecked(match register {
        Qubit::Zero => &zero_map,
        Qubit::One => &one_map,
    }).unwrap()
}

#[rustfmt::skip]
pub fn ry(register: Qubit, angle: f64) -> SuperPosition {
    let cos_parts: Complex<f64> = complex_Re!((0.5f64.mul(angle)).cos());
    let sin_part_pos: Complex<f64> = complex_Re!((0.5f64.mul(angle)).sin());
    let sin_part_neg: Complex<f64> = complex_Re!(-(0.5f64.mul(angle)).sin());
    let zero_map: [Complex<f64>; 2] = [cos_parts, sin_part_pos];
    let one_map: [Complex<f64>; 2] = [sin_part_neg, cos_parts];

    SuperPosition::new(1).set_amplitudes_unchecked(match register {
        Qubit::Zero => &zero_map,
        Qubit::One => &one_map,
    }).unwrap()
}

#[rustfmt::skip]
pub fn rz(register: Qubit, angle: f64) -> SuperPosition {
    let neg_exp: Complex<f64> = Complex::<f64>::expi(-angle*0.5f64);
    let pos_exp: Complex<f64> = Complex::<f64>::expi(angle*0.5f64);
    let zero_map: [Complex<f64>; 2] = [neg_exp, complex_zero!()];
    let one_map: [Complex<f64>; 2] = [complex_zero!(), pos_exp];

    SuperPosition::new(1).set_amplitudes_unchecked(match register {
        Qubit::Zero => &zero_map,
        Qubit::One => &one_map,
    }).unwrap()
}

#[rustfmt::skip]
pub fn global_phase(register: Qubit, angle: f64) -> SuperPosition {
    let exp: Complex<f64> = Complex::<f64>::expi(angle*0.5f64);
    let zero_map: [Complex<f64>; 2] = [exp, complex_zero!()];
    let one_map: [Complex<f64>; 2] = [complex_zero!(), exp];

    SuperPosition::new(1).set_amplitudes_unchecked(match register {
        Qubit::Zero => &zero_map,
        Qubit::One => &one_map,
    }).unwrap()
}

#[rustfmt::skip]
pub fn x90(register: Qubit) -> SuperPosition {
    SuperPosition::new(1).set_amplitudes_unchecked(match register {
        Qubit::Zero => &[complex_zero!(), complex_Im!(-1f64)],
        Qubit::One => &[complex_Im!(-1f64), complex_zero!()],
    }).unwrap()
}

#[rustfmt::skip]
pub fn y90(register: Qubit) -> SuperPosition {
    SuperPosition::new(1).set_amplitudes_unchecked(match register {
        Qubit::Zero => &[complex_zero!(), complex_Re!(-1f64)],
        Qubit::One => &[complex_Re!(1f64), complex_zero!()],
    }).unwrap()
}

#[rustfmt::skip]
pub fn mx90(register: Qubit) -> SuperPosition {
    SuperPosition::new(1).set_amplitudes_unchecked(match register {
        Qubit::Zero => &[complex_zero!(), complex_Im!(1f64)],
        Qubit::One => &[complex_Im!(1f64), complex_zero!()],
    }).unwrap()
}

#[rustfmt::skip]
pub fn my90(register: Qubit) -> SuperPosition {
    SuperPosition::new(1).set_amplitudes_unchecked(match register {
        Qubit::Zero => &[complex_zero!(), complex_Re!(1f64)],
        Qubit::One => &[complex_Re!(-1f64), complex_zero!()],
    }).unwrap()
}

#[rustfmt::skip]
pub fn tgate(register: Qubit) -> SuperPosition {
    SuperPosition::new(1).set_amplitudes_unchecked(match register {
        Qubit::Zero => &[complex_Re!(1f64), complex_zero!()],
        Qubit::One => &[complex_zero!(), complex!(FRAC_1_SQRT_2, FRAC_1_SQRT_2)],
    }).unwrap()
}

#[rustfmt::skip]
pub fn tgatedag(register: Qubit) -> SuperPosition {
    SuperPosition::new(1).set_amplitudes_unchecked(match register {
        Qubit::Zero => &[complex_Re!(1f64), complex_zero!()],
        Qubit::One => &[complex_zero!(), complex!(FRAC_1_SQRT_2, -FRAC_1_SQRT_2)],
    }).unwrap()
}

#[rustfmt::skip]
pub fn phase(register: Qubit) -> SuperPosition {
    SuperPosition::new(1).set_amplitudes_unchecked(match register {
        Qubit::Zero => &[complex_Re!(1f64), complex_zero!()],
        Qubit::One => &[complex_zero!(), complex_Im!(1f64)],
    }).unwrap()
}

#[rustfmt::skip]
pub fn phasedag(register: Qubit) -> SuperPosition {
    SuperPosition::new(1).set_amplitudes_unchecked(match register {
        Qubit::Zero => &[complex_Re!(1f64), complex_zero!()],
        Qubit::One => &[complex_zero!(), complex_Im!(-1f64)],
    }).unwrap()
}

#[rustfmt::skip]
pub fn pauli_x(register: Qubit) -> SuperPosition {
    SuperPosition::new(1).set_amplitudes_unchecked(match register {
        Qubit::Zero => &[complex_zero!(), complex_Re!(1f64)],
        Qubit::One => &[complex_Re!(1f64), complex_zero!()],
    }).unwrap()
}

#[rustfmt::skip]
pub fn pauli_y(register: Qubit) -> SuperPosition {
    SuperPosition::new(1).set_amplitudes_unchecked(match register {
        Qubit::Zero => &[complex_zero!(), complex_Im!(1f64)],
        Qubit::One => &[complex_Im!(-1f64), complex_zero!()],
    }).unwrap()
}

#[rustfmt::skip]
pub fn pauli_z(register: Qubit) -> SuperPosition {
    SuperPosition::new(1).set_amplitudes_unchecked(match register {
        Qubit::Zero => &[complex_Re!(1f64), complex_zero!()],
        Qubit::One => &[complex_zero!(), complex_Re!(-1f64)],
    }).unwrap()
}

//
// Double gates
//

#[rustfmt::skip]
pub fn cnot(register: ProductState) -> SuperPosition {
    let input_register: [Qubit; 2] = [register.state[0], register.state[1]];
    SuperPosition::new(2).set_amplitudes_unchecked(match input_register {
        [Qubit::Zero, Qubit::Zero] => &complex_Re_array!(1f64, 0f64, 0f64, 0f64),
        [Qubit::Zero, Qubit::One]  => &complex_Re_array!(0f64, 1f64, 0f64, 0f64),
        [Qubit::One, Qubit::Zero]  => &complex_Re_array!(0f64, 0f64, 0f64, 1f64),
        [Qubit::One, Qubit::One]   => &complex_Re_array!(0f64, 0f64, 1f64, 0f64),
    }).unwrap()
}

#[rustfmt::skip]
pub fn cy(register: ProductState) -> SuperPosition {
    let input_register: [Qubit; 2] = [register.state[0], register.state[1]];
    SuperPosition::new(2).set_amplitudes_unchecked(match input_register {
        [Qubit::Zero, Qubit::Zero] => &complex_Re_array!(1f64, 0f64, 0f64, 0f64),
        [Qubit::Zero, Qubit::One]  => &complex_Re_array!(0f64, 1f64, 0f64, 0f64),
        [Qubit::One, Qubit::Zero]  => &complex_Im_array!(0f64, 0f64, 0f64, 1f64),
        [Qubit::One, Qubit::One]   => &complex_Im_array!(0f64, 0f64, -1f64, 0f64),
    }).unwrap()
}

#[rustfmt::skip]
pub fn cz(register: ProductState) -> SuperPosition {
    let input_register: [Qubit; 2] = [register.state[0], register.state[1]];
    SuperPosition::new(2).set_amplitudes_unchecked(match input_register {
        [Qubit::Zero, Qubit::Zero] => &complex_Re_array!(1f64, 0f64, 0f64, 0f64),
        [Qubit::Zero, Qubit::One]  => &complex_Re_array!(0f64, 1f64, 0f64, 0f64),
        [Qubit::One, Qubit::Zero]  => &complex_Re_array!(0f64, 0f64, 1f64, 0f64),
        [Qubit::One, Qubit::One]   => &complex_Re_array!(0f64, 0f64, 0f64, -1f64),
    }).unwrap()
}

#[rustfmt::skip]
pub fn swap(register: ProductState) -> SuperPosition {
    let input_register: [Qubit; 2] = [register.state[0], register.state[1]];
    SuperPosition::new(2).set_amplitudes_unchecked(match input_register {
        [Qubit::Zero, Qubit::Zero] => &complex_Re_array!(1f64, 0f64, 0f64, 0f64),
        [Qubit::Zero, Qubit::One]  => &complex_Re_array!(0f64, 0f64, 1f64, 0f64),
        [Qubit::One, Qubit::Zero]  => &complex_Re_array!(0f64, 1f64, 0f64, 0f64),
        [Qubit::One, Qubit::One]   => &complex_Re_array!(0f64, 0f64, 0f64, 1f64),
    }).unwrap()
}

#[rustfmt::skip]
pub fn cr(register: ProductState, angle: f64) -> SuperPosition {
    let input_register: [Qubit; 2] = [register.state[0], register.state[1]];
    let exp_array: [Complex<f64>; 4] = [complex_zero!(), complex_zero!(), complex_zero!(), Complex::<f64>::expi(angle)];
    SuperPosition::new(2).set_amplitudes_unchecked(match input_register {
        [Qubit::Zero, Qubit::Zero] => &complex_Re_array!(1f64, 0f64, 0f64, 0f64),
        [Qubit::Zero, Qubit::One]  => &complex_Re_array!(0f64, 1f64, 0f64, 0f64),
        [Qubit::One, Qubit::Zero]  => &complex_Re_array!(0f64, 0f64, 1f64, 0f64),
        [Qubit::One, Qubit::One]   => &exp_array,
    }).unwrap()
}

#[rustfmt::skip]
pub fn crk(register: ProductState, k: i32) -> SuperPosition {
    let input_register: [Qubit; 2] = [register.state[0], register.state[1]];
    let exp_array: [Complex<f64>; 4] = 
        [complex_zero!(), complex_zero!(), complex_zero!(), Complex::<f64>::expi((2f64*std::f64::consts::PI).div(2f64.powi(k)))];
    SuperPosition::new(2).set_amplitudes_unchecked(match input_register {
        [Qubit::Zero, Qubit::Zero] => &complex_Re_array!(1f64, 0f64, 0f64, 0f64),
        [Qubit::Zero, Qubit::One]  => &complex_Re_array!(0f64, 1f64, 0f64, 0f64),
        [Qubit::One, Qubit::Zero]  => &complex_Re_array!(0f64, 0f64, 1f64, 0f64),
        [Qubit::One, Qubit::One]   => &exp_array,
    }).unwrap()
}

//
// Triple gates
//

#[rustfmt::skip]
pub fn toffoli(register: ProductState) -> SuperPosition {
    let input_register: [Qubit; 3] = [register.state[0], register.state[1], register.state[2]];
    SuperPosition::new(3)
        .set_amplitudes_unchecked(match input_register {
            [Qubit::Zero, Qubit::Zero, Qubit::Zero] => {&complex_Re_array!(1f64, 0f64, 0f64, 0f64, 0f64, 0f64, 0f64, 0f64) }
            [Qubit::Zero, Qubit::Zero, Qubit::One] => { &complex_Re_array!(0f64, 1f64, 0f64, 0f64, 0f64, 0f64, 0f64, 0f64) }
            [Qubit::Zero, Qubit::One, Qubit::Zero] => { &complex_Re_array!(0f64, 0f64, 1f64, 0f64, 0f64, 0f64, 0f64, 0f64) }
            [Qubit::Zero, Qubit::One, Qubit::One] => {  &complex_Re_array!(0f64, 0f64, 0f64, 1f64, 0f64, 0f64, 0f64, 0f64) }
            [Qubit::One, Qubit::Zero, Qubit::Zero] => { &complex_Re_array!(0f64, 0f64, 0f64, 0f64, 1f64, 0f64, 0f64, 0f64) }
            [Qubit::One, Qubit::Zero, Qubit::One] => {  &complex_Re_array!(0f64, 0f64, 0f64, 0f64, 0f64, 1f64, 0f64, 0f64) }
            [Qubit::One, Qubit::One, Qubit::Zero] => {  &complex_Re_array!(0f64, 0f64, 0f64, 0f64, 0f64, 0f64, 0f64, 1f64) }
            [Qubit::One, Qubit::One, Qubit::One] => {   &complex_Re_array!(0f64, 0f64, 0f64, 0f64, 0f64, 0f64, 1f64, 0f64) }
        })
        .unwrap()
}
