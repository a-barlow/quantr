/*
* Copyright (c) 2023 Andrew Rowan Barlow. Licensed under the EUPL-1.2
* or later. You may obtain a copy of the licence at
* https://joinup.ec.europa.eu/collection/eupl/eupl-text-eupl-12. A copy
* of the EUPL-1.2 licence in English is given in LICENCE.txt which is
* found in the root directory of this repository.
*
* Author: Andrew Rowan Barlow <a.barlow.dev@gmail.com>
*/
use crate::circuit::HashMap;
use crate::states::{ProductState, SuperPosition};
use crate::{complex_re, Complex, COMPLEX_ZERO};

impl SuperPosition {
    pub(crate) fn new_with_hash_amplitudes_unchecked(
        hash_amplitudes: HashMap<ProductState, Complex<f64>>,
    ) -> SuperPosition {
        let product_dim: usize = hash_amplitudes.keys().next().unwrap().num_qubits();
        let mut amplitudes: Vec<Complex<f64>> = vec![COMPLEX_ZERO; 1 << product_dim];
        Self::from_hash_to_array(hash_amplitudes, &mut amplitudes);
        SuperPosition {
            amplitudes,
            product_dim,
        }
    }

    /// Used in standard_gate_ops.rs for defining the "standard gates".1
    pub(crate) fn new_with_register_unchecked<const N: usize>(
        amplitudes: [Complex<f64>; N],
    ) -> SuperPosition {
        SuperPosition {
            amplitudes: amplitudes.to_vec(),
            product_dim: N.trailing_zeros() as usize,
        }
    }

    /// Sets the amplitudes of a [SuperPosition] from a HashMap.
    /// BUT, it does not clear the pre-existing vector, so it does produce state vectors that do
    /// not conserve probability. See circuit::tests::custom_gate for an example. This is needed
    /// for the main algorithm to currently operate (quantr 0.4.0).
    /// States that are missing from the HashMap will be assumed to have 0 amplitude.
    pub(crate) fn set_amplitudes_from_states_unchecked(
        &mut self,
        hash_amplitudes: HashMap<ProductState, Complex<f64>>,
    ) -> &mut SuperPosition {
        for (key, val) in hash_amplitudes {
            self.amplitudes[key.comp_basis()] = val;
        }
        self
    }

    pub(crate) fn new_unchecked(num_qubits: usize) -> SuperPosition {
        let mut new_amps: Vec<Complex<f64>> = vec![COMPLEX_ZERO; 1 << num_qubits];
        new_amps[0] = complex_re!(1f64);
        SuperPosition {
            amplitudes: new_amps,
            product_dim: num_qubits,
        }
    }
}
