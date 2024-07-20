/*
* Copyright (c) 2024 Andrew Rowan Barlow. Licensed under the EUPL-1.2
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

use crate::states::{Qubit, SuperPosition};
use crate::{complex_im, complex_im_array, complex_re, complex_re_array};
use num_complex::{c64, Complex64};
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
pub fn hadamard(register: Qubit) -> SuperPosition {
    SuperPosition::new_with_register_unchecked::<2>(match register {
        Qubit::Zero => complex_re_array!(FRAC_1_SQRT_2, FRAC_1_SQRT_2),
        Qubit::One => complex_re_array!(FRAC_1_SQRT_2, -FRAC_1_SQRT_2),
    })
}

#[rustfmt::skip]
pub fn rx(register: Qubit, angle: f64) -> SuperPosition {
    let real_parts: Complex64 = complex_re!((0.5f64.mul(angle)).cos());
    let imaginary_part: Complex64 = complex_im!(-(0.5f64.mul(angle)).sin());
    let zero_map: [Complex64; 2] = [real_parts, imaginary_part];
    let one_map: [Complex64; 2] = [imaginary_part, real_parts];

    SuperPosition::new_with_register_unchecked::<2>(match register {
        Qubit::Zero => zero_map,
        Qubit::One => one_map,
    })
}

#[rustfmt::skip]
pub fn ry(register: Qubit, angle: f64) -> SuperPosition {
    let cos_parts: Complex64 = complex_re!((0.5f64.mul(angle)).cos());
    let sin_part_pos: Complex64 = complex_re!((0.5f64.mul(angle)).sin());
    let sin_part_neg: Complex64 = complex_re!(-(0.5f64.mul(angle)).sin());
    let zero_map: [Complex64; 2] = [cos_parts, sin_part_pos];
    let one_map: [Complex64; 2] = [sin_part_neg, cos_parts];

    SuperPosition::new_with_register_unchecked::<2>(match register {
        Qubit::Zero => zero_map,
        Qubit::One => one_map,
    })
}

#[rustfmt::skip]
pub fn rz(register: Qubit, angle: f64) -> SuperPosition {
    let neg_exp: Complex64 = (c64(0f64, -angle*0.5f64)).exp();
    let pos_exp: Complex64 = (c64(0f64, angle*0.5f64)).exp();
    let zero_map: [Complex64; 2] = [neg_exp, num_complex::Complex64::ZERO];
    let one_map: [Complex64; 2] = [num_complex::Complex64::ZERO, pos_exp];

    SuperPosition::new_with_register_unchecked::<2>(match register {
        Qubit::Zero => zero_map,
        Qubit::One => one_map,
    })
}

#[rustfmt::skip]
pub fn global_phase(register: Qubit, angle: f64) -> SuperPosition {
    let exp: Complex64 = (c64(0f64, angle*0.5f64)).exp();
    let zero_map: [Complex64; 2] = [exp, num_complex::Complex64::ZERO];
    let one_map: [Complex64; 2] = [num_complex::Complex64::ZERO, exp];

    SuperPosition::new_with_register_unchecked::<2>(match register {
        Qubit::Zero => zero_map,
        Qubit::One => one_map,
    })
}

#[rustfmt::skip]
pub fn x90(register: Qubit) -> SuperPosition {
    SuperPosition::new_with_register_unchecked::<2>(match register {
        Qubit::Zero => [num_complex::Complex64::ZERO, complex_im!(-1f64)],
        Qubit::One => [complex_im!(-1f64), num_complex::Complex64::ZERO],
    })
}

#[rustfmt::skip]
pub fn y90(register: Qubit) -> SuperPosition {
    SuperPosition::new_with_register_unchecked::<2>(match register {
        Qubit::Zero => [num_complex::Complex64::ZERO, complex_re!(-1f64)],
        Qubit::One => [complex_re!(1f64), num_complex::Complex64::ZERO],
    })
}

#[rustfmt::skip]
pub fn mx90(register: Qubit) -> SuperPosition {
    SuperPosition::new_with_register_unchecked::<2>(match register {
        Qubit::Zero => [num_complex::Complex64::ZERO, complex_im!(1f64)],
        Qubit::One => [complex_im!(1f64), num_complex::Complex64::ZERO],
    })
}

#[rustfmt::skip]
pub fn my90(register: Qubit) -> SuperPosition {
    SuperPosition::new_with_register_unchecked::<2>(match register {
        Qubit::Zero => [num_complex::Complex64::ZERO, complex_re!(1f64)],
        Qubit::One => [complex_re!(-1f64), num_complex::Complex64::ZERO],
    })
}

#[rustfmt::skip]
pub fn tgate(register: Qubit) -> SuperPosition {
    SuperPosition::new_with_register_unchecked::<2>(match register {
        Qubit::Zero => [complex_re!(1f64), num_complex::Complex64::ZERO],
        Qubit::One => [num_complex::Complex64::ZERO, c64(FRAC_1_SQRT_2, FRAC_1_SQRT_2)],
    })
}

#[rustfmt::skip]
pub fn tgatedag(register: Qubit) -> SuperPosition {
    SuperPosition::new_with_register_unchecked::<2>(match register {
        Qubit::Zero => [complex_re!(1f64), num_complex::Complex64::ZERO],
        Qubit::One => [num_complex::Complex64::ZERO, c64(FRAC_1_SQRT_2, -FRAC_1_SQRT_2)],
    })
}

#[rustfmt::skip]
pub fn phase(register: Qubit) -> SuperPosition {
    SuperPosition::new_with_register_unchecked::<2>(match register {
        Qubit::Zero => [complex_re!(1f64), num_complex::Complex64::ZERO],
        Qubit::One => [num_complex::Complex64::ZERO, complex_im!(1f64)],
    })
}

#[rustfmt::skip]
pub fn phasedag(register: Qubit) -> SuperPosition {
    SuperPosition::new_with_register_unchecked::<2>(match register {
        Qubit::Zero => [complex_re!(1f64), num_complex::Complex64::ZERO],
        Qubit::One => [num_complex::Complex64::ZERO, complex_im!(-1f64)],
    })
}

#[rustfmt::skip]
pub fn pauli_x(register: Qubit) -> SuperPosition {
    SuperPosition::new_with_register_unchecked::<2>(match register {
        Qubit::Zero => [num_complex::Complex64::ZERO, complex_re!(1f64)],
        Qubit::One => [complex_re!(1f64), num_complex::Complex64::ZERO],
    })
}

#[rustfmt::skip]
pub fn pauli_y(register: Qubit) -> SuperPosition {
    SuperPosition::new_with_register_unchecked::<2>(match register {
        Qubit::Zero => [num_complex::Complex64::ZERO, complex_im!(1f64)],
        Qubit::One => [complex_im!(-1f64), num_complex::Complex64::ZERO],
    })
}

#[rustfmt::skip]
pub fn pauli_z(register: Qubit) -> SuperPosition {
    SuperPosition::new_with_register_unchecked::<2>(match register {
        Qubit::Zero => [complex_re!(1f64), num_complex::Complex64::ZERO],
        Qubit::One => [num_complex::Complex64::ZERO, complex_re!(-1f64)],
    })
}

//
// Double gates
//

#[rustfmt::skip]
pub fn cnot(qubit_one: Qubit, qubit_two: Qubit) -> SuperPosition {
    SuperPosition::new_with_register_unchecked::<4>(match [qubit_one, qubit_two] {
        [Qubit::Zero, Qubit::Zero] => complex_re_array!(1f64, 0f64, 0f64, 0f64),
        [Qubit::Zero, Qubit::One]  => complex_re_array!(0f64, 1f64, 0f64, 0f64),
        [Qubit::One, Qubit::Zero]  => complex_re_array!(0f64, 0f64, 0f64, 1f64),
        [Qubit::One, Qubit::One]   => complex_re_array!(0f64, 0f64, 1f64, 0f64),
    })
}

#[rustfmt::skip]
pub fn cy(qubit_one: Qubit, qubit_two: Qubit) -> SuperPosition {
    SuperPosition::new_with_register_unchecked::<4>(match [qubit_one, qubit_two] {
        [Qubit::Zero, Qubit::Zero] => complex_re_array!(1f64, 0f64, 0f64, 0f64),
        [Qubit::Zero, Qubit::One]  => complex_re_array!(0f64, 1f64, 0f64, 0f64),
        [Qubit::One, Qubit::Zero]  => complex_im_array!(0f64, 0f64, 0f64, 1f64),
        [Qubit::One, Qubit::One]   => complex_im_array!(0f64, 0f64, -1f64, 0f64),
    })
}

#[rustfmt::skip]
pub fn cz(qubit_one: Qubit, qubit_two: Qubit) -> SuperPosition {
    SuperPosition::new_with_register_unchecked::<4>(match [qubit_one, qubit_two] {
        [Qubit::Zero, Qubit::Zero] => complex_re_array!(1f64, 0f64, 0f64, 0f64),
        [Qubit::Zero, Qubit::One]  => complex_re_array!(0f64, 1f64, 0f64, 0f64),
        [Qubit::One, Qubit::Zero]  => complex_re_array!(0f64, 0f64, 1f64, 0f64),
        [Qubit::One, Qubit::One]   => complex_re_array!(0f64, 0f64, 0f64, -1f64),
    })
}

#[rustfmt::skip]
pub fn swap(qubit_one: Qubit, qubit_two: Qubit) -> SuperPosition {
    SuperPosition::new_with_register_unchecked::<4>(match [qubit_one, qubit_two] {
        [Qubit::Zero, Qubit::Zero] => complex_re_array!(1f64, 0f64, 0f64, 0f64),
        [Qubit::Zero, Qubit::One]  => complex_re_array!(0f64, 0f64, 1f64, 0f64),
        [Qubit::One, Qubit::Zero]  => complex_re_array!(0f64, 1f64, 0f64, 0f64),
        [Qubit::One, Qubit::One]   => complex_re_array!(0f64, 0f64, 0f64, 1f64),
    })
}

#[rustfmt::skip]
pub fn cr(qubit_one: Qubit, qubit_two: Qubit, angle: f64) -> SuperPosition {
    let exp_array: [Complex64; 4] = [num_complex::Complex64::ZERO, num_complex::Complex64::ZERO, num_complex::Complex64::ZERO, (c64(0f64, angle)).exp()];
    SuperPosition::new_with_register_unchecked::<4>(match [qubit_one, qubit_two] {
        [Qubit::Zero, Qubit::Zero] => complex_re_array!(1f64, 0f64, 0f64, 0f64),
        [Qubit::Zero, Qubit::One]  => complex_re_array!(0f64, 1f64, 0f64, 0f64),
        [Qubit::One, Qubit::Zero]  => complex_re_array!(0f64, 0f64, 1f64, 0f64),
        [Qubit::One, Qubit::One]   => exp_array,
    })
}

#[rustfmt::skip]
pub fn crk(qubit_one: Qubit, qubit_two: Qubit, k: i32) -> SuperPosition {
    let exp_array: [Complex64; 4] = 
        [num_complex::Complex64::ZERO, num_complex::Complex64::ZERO, num_complex::Complex64::ZERO, (c64(0f64, (2f64*std::f64::consts::PI).div(2f64.powi(k)))).exp()];
    SuperPosition::new_with_register_unchecked::<4>(match [qubit_one, qubit_two] {
        [Qubit::Zero, Qubit::Zero] => complex_re_array!(1f64, 0f64, 0f64, 0f64),
        [Qubit::Zero, Qubit::One]  => complex_re_array!(0f64, 1f64, 0f64, 0f64),
        [Qubit::One, Qubit::Zero]  => complex_re_array!(0f64, 0f64, 1f64, 0f64),
        [Qubit::One, Qubit::One]   => exp_array,
    })
}

//
// Triple gates
//

#[rustfmt::skip]
pub fn toffoli(qubit_one: Qubit, qubit_two: Qubit, qubit_three: Qubit) -> SuperPosition {
    SuperPosition::new_with_register_unchecked::<8>(match [qubit_one, qubit_two, qubit_three] {
        [Qubit::Zero, Qubit::Zero, Qubit::Zero] => { complex_re_array!(1f64, 0f64, 0f64, 0f64, 0f64, 0f64, 0f64, 0f64) }
        [Qubit::Zero, Qubit::Zero, Qubit::One] => {  complex_re_array!(0f64, 1f64, 0f64, 0f64, 0f64, 0f64, 0f64, 0f64) }
        [Qubit::Zero, Qubit::One, Qubit::Zero] => {  complex_re_array!(0f64, 0f64, 1f64, 0f64, 0f64, 0f64, 0f64, 0f64) }
        [Qubit::Zero, Qubit::One, Qubit::One] => {   complex_re_array!(0f64, 0f64, 0f64, 1f64, 0f64, 0f64, 0f64, 0f64) }
        [Qubit::One, Qubit::Zero, Qubit::Zero] => {  complex_re_array!(0f64, 0f64, 0f64, 0f64, 1f64, 0f64, 0f64, 0f64) }
        [Qubit::One, Qubit::Zero, Qubit::One] => {   complex_re_array!(0f64, 0f64, 0f64, 0f64, 0f64, 1f64, 0f64, 0f64) }
        [Qubit::One, Qubit::One, Qubit::Zero] => {   complex_re_array!(0f64, 0f64, 0f64, 0f64, 0f64, 0f64, 0f64, 1f64) }
        [Qubit::One, Qubit::One, Qubit::One] => {    complex_re_array!(0f64, 0f64, 0f64, 0f64, 0f64, 0f64, 1f64, 0f64) }
    })
}
