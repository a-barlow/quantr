/*
* Copyright (c) 2023 Andrew Rowan Barlow. Licensed under the EUPL-1.2
* or later. You may obtain a copy of the licence at
* https://joinup.ec.europa.eu/collection/eupl/eupl-text-eupl-12. A copy
* of the EUPL-1.2 licence in English is given in LICENCE.txt which is
* found in the root directory of this repository.
*
* Author: Andrew Rowan Barlow <a.barlow.dev@gmail.com>
*/

use crate::states::{ProductState, SuperPosition};
use crate::Complex;

/// Returns the product state and it's respective amplitude in each iteration.
///
/// # Example
/// ```
/// use quantr::states::{ProductState, Qubit, SuperPosition};
/// use quantr::{complex_Re, complex_Re_vec, COMPLEX_ZERO, Complex};
/// use std::f64::consts::FRAC_1_SQRT_2;
///
/// let super_pos: SuperPosition
///     = SuperPosition::new_with_amplitudes(&complex_Re_vec!(0f64, FRAC_1_SQRT_2, FRAC_1_SQRT_2, 0f64))
///         .unwrap();
///
/// let mut iterator_super_pos = super_pos.into_iter();
///
/// assert_eq!(iterator_super_pos.next(),
///     Some((ProductState::new(&[Qubit::Zero, Qubit::Zero]).unwrap(), COMPLEX_ZERO)));
/// assert_eq!(iterator_super_pos.next(),
///     Some((ProductState::new(&[Qubit::Zero, Qubit::One]).unwrap(), complex_Re!(FRAC_1_SQRT_2))));
/// assert_eq!(iterator_super_pos.next(),
///     Some((ProductState::new(&[Qubit::One, Qubit::Zero]).unwrap(), complex_Re!(FRAC_1_SQRT_2))));
/// assert_eq!(iterator_super_pos.next(),
///     Some((ProductState::new(&[Qubit::One, Qubit::One]).unwrap(), COMPLEX_ZERO)));
/// assert_eq!(iterator_super_pos.next(), None);
/// ```
pub struct SuperPositionIterator<'a> {
    super_position: &'a SuperPosition,
    index: usize,
}

impl<'a> IntoIterator for &'a SuperPosition {
    type Item = (ProductState, Complex<f64>);
    type IntoIter = SuperPositionIterator<'a>;

    fn into_iter(self) -> Self::IntoIter {
        SuperPositionIterator {
            super_position: self,
            index: 0,
        }
    }
}

impl<'a> Iterator for SuperPositionIterator<'a> {
    type Item = (ProductState, Complex<f64>);

    fn next(&mut self) -> Option<Self::Item> {
        if self.index < self.super_position.amplitudes.len() {
            let option_state: Self::Item = (
                ProductState::binary_basis(self.index, self.super_position.product_dim),
                self.super_position.amplitudes[self.index],
            );
            self.index += 1;
            Some(option_state)
        } else {
            self.index = 0;
            None
        }
    }
}
