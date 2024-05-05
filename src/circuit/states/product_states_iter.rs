/*
* Copyright (c) 2024 Andrew Rowan Barlow. Licensed under the EUPL-1.2
* or later. You may obtain a copy of the licence at
* https://joinup.ec.europa.eu/collection/eupl/eupl-text-eupl-12. A copy
* of the EUPL-1.2 licence in English is given in LICENCE.txt which is
* found in the root directory of this repository.
*
* Author: Andrew Rowan Barlow <a.barlow.dev@gmail.com>
*/

use super::{ProductState, Qubit};

/// An iterator of the qubits that label the product state in the computational basis, from left to
/// right in braket notation.
///
/// # Example
/// ```
/// use quantr::states::{ProductState, Qubit, ProductStateIter};
///
/// let state = ProductState::new(&[Qubit::One, Qubit::Zero, Qubit::Zero]).unwrap(); // |100>
///
/// let mut state_iter: ProductStateIter = state.into_iter();
///
/// assert_eq!(state_iter.next(), Some(Qubit::One));
/// assert_eq!(state_iter.next(), Some(Qubit::Zero));
/// assert_eq!(state_iter.next(), Some(Qubit::Zero));
/// assert_eq!(state_iter.next(), None);
/// ```
pub struct ProductStateIter<'a> {
    state: &'a ProductState,
    index: usize,
}

impl<'a> Iterator for ProductStateIter<'a> {
    type Item = Qubit;

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(qubit) = self.state.qubits.get(self.index).copied() {
            self.index += 1;
            Some(qubit)
        } else {
            self.index = 0;
            None
        }
    }
}

impl<'a> IntoIterator for &'a ProductState {
    type Item = Qubit;
    type IntoIter = ProductStateIter<'a>;

    fn into_iter(self) -> Self::IntoIter {
        ProductStateIter {
            state: &self,
            index: 0,
        }
    }
}
