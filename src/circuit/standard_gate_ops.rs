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
use crate::{complex_Im, complex_Im_array, complex_Re, complex_Re_array, complex_zero};
use std::f64::consts::FRAC_1_SQRT_2;

// The following gates (inlcuding triple and custom) are mapping qubits via the 
// computational basis:
// |a> ---- 
// |b> ----
// |c> ----
// => |a,b,c>
//
// `cargo fmt` has also been skipped as this shows the connection between matrices in the
// computational basis, and linear maps!

//
// Single gates
//

#[rustfmt::skip]
pub fn identity(register: Qubit) -> SuperPosition {
    SuperPosition::new(1).set_amplitudes(match register {
        Qubit::Zero => &[complex_Re!(1f64), complex_zero!()],
        Qubit::One => &[complex_zero!(), complex_Re!(1f64)],
    }).unwrap()
}

#[rustfmt::skip]
pub fn hadamard(register: Qubit) -> SuperPosition {
    SuperPosition::new(1).set_amplitudes(match register {
        Qubit::Zero => &complex_Re_array!(FRAC_1_SQRT_2, FRAC_1_SQRT_2),
        Qubit::One => &complex_Re_array!(FRAC_1_SQRT_2, -FRAC_1_SQRT_2),
    }).unwrap()
}

#[rustfmt::skip]
pub fn pauli_x(register: Qubit) -> SuperPosition {
    SuperPosition::new(1).set_amplitudes(match register {
        Qubit::Zero => &[complex_zero!(), complex_Re!(1f64)],
        Qubit::One => &[complex_Re!(1f64), complex_zero!()],
    }).unwrap()
}

#[rustfmt::skip]
pub fn pauli_y(register: Qubit) -> SuperPosition {
    SuperPosition::new(1).set_amplitudes(match register {
        Qubit::Zero => &[complex_zero!(), complex_Im!(1f64)],
        Qubit::One => &[complex_Im!(-1f64), complex_zero!()],
    }).unwrap()
}

#[rustfmt::skip]
pub fn pauli_z(register: Qubit) -> SuperPosition {
    SuperPosition::new(1).set_amplitudes(match register {
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
    SuperPosition::new(2).set_amplitudes(match input_register {
        [Qubit::Zero, Qubit::Zero] => &complex_Re_array!(1f64, 0f64, 0f64, 0f64),
        [Qubit::Zero, Qubit::One]  => &complex_Re_array!(0f64, 1f64, 0f64, 0f64),
        [Qubit::One, Qubit::Zero]  => &complex_Re_array!(0f64, 0f64, 0f64, 1f64),
        [Qubit::One, Qubit::One]   => &complex_Re_array!(0f64, 0f64, 1f64, 0f64),
    }).unwrap()
}

#[rustfmt::skip]
pub fn cy(register: ProductState) -> SuperPosition {
    let input_register: [Qubit; 2] = [register.state[0], register.state[1]];
    SuperPosition::new(2).set_amplitudes(match input_register {
        [Qubit::Zero, Qubit::Zero] => &complex_Re_array!(1f64, 0f64, 0f64, 0f64),
        [Qubit::Zero, Qubit::One]  => &complex_Re_array!(0f64, 1f64, 0f64, 0f64),
        [Qubit::One, Qubit::Zero]  => &complex_Im_array!(0f64, 0f64, 0f64, 1f64),
        [Qubit::One, Qubit::One]   => &complex_Im_array!(0f64, 0f64, -1f64, 0f64),
    }).unwrap()
}

#[rustfmt::skip]
pub fn cz(register: ProductState) -> SuperPosition {
    let input_register: [Qubit; 2] = [register.state[0], register.state[1]];
    SuperPosition::new(2).set_amplitudes(match input_register {
        [Qubit::Zero, Qubit::Zero] => &complex_Re_array!(1f64, 0f64, 0f64, 0f64),
        [Qubit::Zero, Qubit::One]  => &complex_Re_array!(0f64, 1f64, 0f64, 0f64),
        [Qubit::One, Qubit::Zero]  => &complex_Re_array!(0f64, 0f64, 1f64, 0f64),
        [Qubit::One, Qubit::One]   => &complex_Re_array!(0f64, 0f64, 0f64, -1f64),
    }).unwrap()
}

#[rustfmt::skip]
pub fn swap(register: ProductState) -> SuperPosition {
    let input_register: [Qubit; 2] = [register.state[0], register.state[1]];
    SuperPosition::new(2).set_amplitudes(match input_register {
        [Qubit::Zero, Qubit::Zero] => &complex_Re_array!(1f64, 0f64, 0f64, 0f64),
        [Qubit::Zero, Qubit::One]  => &complex_Re_array!(0f64, 0f64, 1f64, 0f64),
        [Qubit::One, Qubit::Zero]  => &complex_Re_array!(0f64, 1f64, 0f64, 0f64),
        [Qubit::One, Qubit::One]   => &complex_Re_array!(0f64, 0f64, 0f64, -1f64),
    }).unwrap()
}

//
// Triple gates
//

#[rustfmt::skip]
pub fn toffoli(register: ProductState) -> SuperPosition {
    let input_register: [Qubit; 3] = [register.state[0], register.state[1], register.state[2]];
    SuperPosition::new(3)
        .set_amplitudes(match input_register {
            [Qubit::Zero, Qubit::Zero, Qubit::Zero] => {
                &complex_Re_array!(1f64, 0f64, 0f64, 0f64, 0f64, 0f64, 0f64, 0f64)
            }
            [Qubit::Zero, Qubit::Zero, Qubit::One] => {
                &complex_Re_array!(0f64, 1f64, 0f64, 0f64, 0f64, 0f64, 0f64, 0f64)
            }
            [Qubit::Zero, Qubit::One, Qubit::Zero] => {
                &complex_Re_array!(0f64, 0f64, 1f64, 0f64, 0f64, 0f64, 0f64, 0f64)
            }
            [Qubit::Zero, Qubit::One, Qubit::One] => {
                &complex_Re_array!(0f64, 0f64, 0f64, 1f64, 0f64, 0f64, 0f64, 0f64)
            }
            [Qubit::One, Qubit::Zero, Qubit::Zero] => {
                &complex_Re_array!(0f64, 0f64, 0f64, 0f64, 1f64, 0f64, 0f64, 0f64)
            }
            [Qubit::One, Qubit::Zero, Qubit::One] => {
                &complex_Re_array!(0f64, 0f64, 0f64, 0f64, 0f64, 1f64, 0f64, 0f64)
            }
            [Qubit::One, Qubit::One, Qubit::Zero] => {
                &complex_Re_array!(0f64, 0f64, 0f64, 0f64, 0f64, 0f64, 0f64, 1f64)
            }
            [Qubit::One, Qubit::One, Qubit::One] => {
                &complex_Re_array!(0f64, 0f64, 0f64, 0f64, 0f64, 0f64, 1f64, 0f64)
            }
        })
        .unwrap()
}
pub fn toffoli(register: ProductState) -> SuperPosition {
    let input_register: [Qubit; 3] = [register.state[0], register.state[1], register.state[2]];
    SuperPosition::new(3)
        .set_amplitudes(match input_register {
            [Qubit::Zero, Qubit::Zero, Qubit::Zero] => {
                &complex_Re_array!(1f64, 0f64, 0f64, 0f64, 0f64, 0f64, 0f64, 0f64)
            }
            [Qubit::Zero, Qubit::Zero, Qubit::One] => {
                &complex_Re_array!(0f64, 1f64, 0f64, 0f64, 0f64, 0f64, 0f64, 0f64)
            }
            [Qubit::Zero, Qubit::One, Qubit::Zero] => {
                &complex_Re_array!(0f64, 0f64, 1f64, 0f64, 0f64, 0f64, 0f64, 0f64)
            }
            [Qubit::Zero, Qubit::One, Qubit::One] => {
                &complex_Re_array!(0f64, 0f64, 0f64, 1f64, 0f64, 0f64, 0f64, 0f64)
            }
            [Qubit::One, Qubit::Zero, Qubit::Zero] => {
                &complex_Re_array!(0f64, 0f64, 0f64, 0f64, 1f64, 0f64, 0f64, 0f64)
            }
            [Qubit::One, Qubit::Zero, Qubit::One] => {
                &complex_Re_array!(0f64, 0f64, 0f64, 0f64, 0f64, 1f64, 0f64, 0f64)
            }
            [Qubit::One, Qubit::One, Qubit::Zero] => {
                &complex_Re_array!(0f64, 0f64, 0f64, 0f64, 0f64, 0f64, 0f64, 1f64)
            }
            [Qubit::One, Qubit::One, Qubit::One] => {
                &complex_Re_array!(0f64, 0f64, 0f64, 0f64, 0f64, 0f64, 1f64, 0f64)
            }
        })
        .unwrap()
}
pub fn toffoli(register: ProductState) -> SuperPosition {
    let input_register: [Qubit; 3] = [register.state[0], register.state[1], register.state[2]];
    SuperPosition::new(3)
        .set_amplitudes(match input_register {
            [Qubit::Zero, Qubit::Zero, Qubit::Zero] => {
                &complex_Re_array!(1f64, 0f64, 0f64, 0f64, 0f64, 0f64, 0f64, 0f64)
            }
            [Qubit::Zero, Qubit::Zero, Qubit::One] => {
                &complex_Re_array!(0f64, 1f64, 0f64, 0f64, 0f64, 0f64, 0f64, 0f64)
            }
            [Qubit::Zero, Qubit::One, Qubit::Zero] => {
                &complex_Re_array!(0f64, 0f64, 1f64, 0f64, 0f64, 0f64, 0f64, 0f64)
            }
            [Qubit::Zero, Qubit::One, Qubit::One] => {
                &complex_Re_array!(0f64, 0f64, 0f64, 1f64, 0f64, 0f64, 0f64, 0f64)
            }
            [Qubit::One, Qubit::Zero, Qubit::Zero] => {
                &complex_Re_array!(0f64, 0f64, 0f64, 0f64, 1f64, 0f64, 0f64, 0f64)
            }
            [Qubit::One, Qubit::Zero, Qubit::One] => {
                &complex_Re_array!(0f64, 0f64, 0f64, 0f64, 0f64, 1f64, 0f64, 0f64)
            }
            [Qubit::One, Qubit::One, Qubit::Zero] => {
                &complex_Re_array!(0f64, 0f64, 0f64, 0f64, 0f64, 0f64, 0f64, 1f64)
            }
            [Qubit::One, Qubit::One, Qubit::One] => {
                &complex_Re_array!(0f64, 0f64, 0f64, 0f64, 0f64, 0f64, 1f64, 0f64)
            }
        })
        .unwrap()
}
pub fn toffoli(register: ProductState) -> SuperPosition {
    let input_register: [Qubit; 3] = [register.state[0], register.state[1], register.state[2]];
    SuperPosition::new(3)
        .set_amplitudes(match input_register {
            [Qubit::Zero, Qubit::Zero, Qubit::Zero] => {
                &complex_Re_array!(1f64, 0f64, 0f64, 0f64, 0f64, 0f64, 0f64, 0f64)
            }
            [Qubit::Zero, Qubit::Zero, Qubit::One] => {
                &complex_Re_array!(0f64, 1f64, 0f64, 0f64, 0f64, 0f64, 0f64, 0f64)
            }
            [Qubit::Zero, Qubit::One, Qubit::Zero] => {
                &complex_Re_array!(0f64, 0f64, 1f64, 0f64, 0f64, 0f64, 0f64, 0f64)
            }
            [Qubit::Zero, Qubit::One, Qubit::One] => {
                &complex_Re_array!(0f64, 0f64, 0f64, 1f64, 0f64, 0f64, 0f64, 0f64)
            }
            [Qubit::One, Qubit::Zero, Qubit::Zero] => {
                &complex_Re_array!(0f64, 0f64, 0f64, 0f64, 1f64, 0f64, 0f64, 0f64)
            }
            [Qubit::One, Qubit::Zero, Qubit::One] => {
                &complex_Re_array!(0f64, 0f64, 0f64, 0f64, 0f64, 1f64, 0f64, 0f64)
            }
            [Qubit::One, Qubit::One, Qubit::Zero] => {
                &complex_Re_array!(0f64, 0f64, 0f64, 0f64, 0f64, 0f64, 0f64, 1f64)
            }
            [Qubit::One, Qubit::One, Qubit::One] => {
                &complex_Re_array!(0f64, 0f64, 0f64, 0f64, 0f64, 0f64, 1f64, 0f64)
            }
        })
        .unwrap()
}
