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
use crate::{Complex, COMPLEX_ZERO};

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
    /// States that are missing from the HashMap will be assumed to have 0 amplitude.
    pub(crate) fn set_amplitudes_from_states_unchecked(
        &mut self,
        amplitudes: HashMap<ProductState, Complex<f64>>,
    ) -> &mut SuperPosition {
        Self::from_hash_to_array(amplitudes, &mut self.amplitudes);
        self
    }
}
