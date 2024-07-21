/*
* Copyright (c) 2024 Andrew Rowan Barlow. Licensed under the EUPL-1.2
* or later. You may obtain a copy of the licence at
* https://joinup.ec.europa.eu/collection/eupl/eupl-text-eupl-12. A copy
* of the EUPL-1.2 licence in English is given in LICENCE.txt which is
* found in the root directory of this repository.
*
* Author: Andrew Rowan Barlow <a.barlow.dev@gmail.com>
*/
use crate::circuit::HashMap;
use crate::complex_re;
use crate::states::{ProductState, SuperPosition};
use num_complex::Complex64;

impl SuperPosition {
    pub(crate) fn new_with_hash_amplitudes_unchecked(
        hash_amplitudes: HashMap<ProductState, Complex64>,
    ) -> SuperPosition {
        let product_dim: usize = hash_amplitudes.keys().next().unwrap().num_qubits();
        let mut amplitudes: Vec<Complex64> = vec![num_complex::Complex64::ZERO; 1 << product_dim];
        Self::from_hash_to_array(hash_amplitudes, &mut amplitudes);
        SuperPosition {
            amplitudes,
            product_dim,
        }
    }

    // As only used in `standard_gate_ops`, could specify product_dim manually, saves computation.
    /// Used in standard_gate_ops.rs for defining the "standard gates".1
    pub(crate) fn new_with_register_unchecked<const N: usize>(
        amplitudes: [Complex64; N],
    ) -> SuperPosition {
        SuperPosition {
            amplitudes: amplitudes.to_vec(),
            product_dim: N.trailing_zeros() as usize,
        }
    }

    pub(crate) fn new_unchecked(num_qubits: usize) -> SuperPosition {
        let mut new_amps: Vec<Complex64> = vec![num_complex::Complex64::ZERO; 1 << num_qubits];
        new_amps[0] = complex_re!(1f64);
        SuperPosition {
            amplitudes: new_amps,
            product_dim: num_qubits,
        }
    }

    /// Sets the amplitudes of a [SuperPosition] from a HashMap, **without** check on conservation of
    /// probability.
    pub(crate) fn set_amplitudes_from_states_unchecked(
        &mut self,
        mut hash_amplitudes: HashMap<ProductState, Complex64>,
    ) -> &mut SuperPosition {
        for (i, amp) in self.amplitudes.iter_mut().enumerate() {
            *amp = hash_amplitudes
                .remove(&ProductState::binary_basis(i, self.product_dim))
                .unwrap_or(complex_re!(0f64));
        }
        self
    }

    /// Same as [SuperPosition::new_with_amplitudes], but **without** checks on dimension size being a
    /// power of two and the conservation of probability.
    pub fn new_with_amplitudes_unchecked(amplitudes: &[Complex64]) -> SuperPosition {
        let length = amplitudes.len();
        SuperPosition {
            amplitudes: amplitudes.to_vec(),
            product_dim: length.trailing_zeros() as usize,
        }
    }

    /// Same as [SuperPosition::set_amplitudes], but **without** checks on conservation of
    /// probability.
    pub fn set_amplitudes_unchecked(&mut self, amplitudes: &[Complex64]) -> &mut SuperPosition {
        self.amplitudes = amplitudes.to_vec();
        self
    }
}
