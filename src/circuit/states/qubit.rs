/*
* Copyright (c) 2023 Andrew Rowan Barlow. Licensed under the EUPL-1.2
* or later. You may obtain a copy of the licence at
* https://joinup.ec.europa.eu/collection/eupl/eupl-text-eupl-12. A copy
* of the EUPL-1.2 licence in English is given in LICENCE.txt which is
* found in the root directory of this repository.
*
* Author: Andrew Rowan Barlow <a.barlow.dev@gmail.com>
*/

use crate::states::ProductState;

/// The fundamental unit in quantum computers, the qubit.
#[derive(Clone, Copy, Hash, PartialEq, Eq, Debug)]
pub enum Qubit {
    /// |0⟩
    Zero,
    /// |1⟩
    One,
}

impl Qubit {
    /// Defines the Kronecker product of two qubits.
    ///
    /// # Example
    /// ```
    /// use quantr::states::{ProductState, Qubit};
    ///
    /// let qubit_a: Qubit = Qubit::Zero; // |0>
    /// let qubit_b: Qubit = Qubit::One;  // |1>
    ///
    /// let new_product: ProductState = qubit_a.kronecker_prod(qubit_b); // |0> ⊗ |1> = |01>
    /// assert_eq!(new_product.qubits.as_slice(), &[Qubit::Zero, Qubit::One])
    /// ```
    pub fn kronecker_prod(self, other: Qubit) -> ProductState {
        ProductState::new_unchecked(&[self, other])
    }

    /// Converts the [Qubit] to a [ProductState] struct.
    pub fn into_state(self) -> ProductState {
        ProductState::new_unchecked(&[self])
    }
}

