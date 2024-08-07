/*
* Copyright (c) 2024 Andrew Rowan Barlow. Licensed under the EUPL-1.2
* or later. You may obtain a copy of the licence at
* https://joinup.ec.europa.eu/collection/eupl/eupl-text-eupl-12. A copy
* of the EUPL-1.2 licence in English is given in LICENCE.txt which is
* found in the root directory of this repository.
*
* Author: Andrew Rowan Barlow <a.barlow.dev@gmail.com>
*/

use crate::circuit::{HashMap, QResult};
use crate::complex_re;
use crate::error::QuantrError;
use crate::states::ProductState;
use crate::states::Qubit;
use num_complex::Complex64;

const ZERO_MARGIN: f64 = 1e-6;

/// A superposition of [ProductState]s.
#[derive(PartialEq, Debug, Clone)]
pub struct SuperPosition {
    pub(crate) amplitudes: Vec<Complex64>,
    pub(crate) product_dim: usize,
}

impl SuperPosition {
    /// Creates a superposition in the |0..0> state. The `prod_dimension` specifies the number of
    /// qubits that compose each product state. For example, |000> corresponds to `prod_dimension =
    /// 3`.
    ///
    /// # Example
    /// ```
    /// use quantr::states::SuperPosition;
    /// use quantr::complex_re_array;
    ///
    /// let superpos = SuperPosition::new(2).unwrap();
    ///
    /// assert_eq!(&complex_re_array![1f64, 0f64, 0f64, 0f64], superpos.get_amplitudes());
    /// ```
    pub fn new(prod_dimension: usize) -> QResult<SuperPosition> {
        if prod_dimension == 0 {
            return Err(QuantrError {
                message: String::from("The number of qubits must be non-zero."),
            });
        }

        let mut new_amps: Vec<Complex64> = vec![num_complex::Complex64::ZERO; 1 << prod_dimension];
        new_amps[0] = complex_re!(1f64);
        Ok(SuperPosition {
            amplitudes: new_amps,
            product_dim: prod_dimension,
        })
    }

    /// Creates a superposition based on the complex amplitudes of each state labelled in
    /// the computational basis.
    ///
    /// # Example
    /// ```
    /// use quantr::states::SuperPosition;
    /// use quantr::complex_re_array;
    ///
    /// let superpos = SuperPosition::new_with_amplitudes(&complex_re_array![1f64, 0f64, 0f64, 0f64]).unwrap();
    ///
    /// assert_eq!(&complex_re_array![1f64, 0f64, 0f64, 0f64], superpos.get_amplitudes());
    /// ```
    pub fn new_with_amplitudes(amplitudes: &[Complex64]) -> QResult<SuperPosition> {
        if !Self::equal_within_error(amplitudes.iter().map(|x| x.norm_sqr()).sum::<f64>(), 1f64) {
            return Err(QuantrError{
                message: String::from("Slice given to set amplitudes in super position does not conserve probability, the absolute square sum of the coefficents must be one."),
            });
        }

        let length = amplitudes.len();
        if (length & (length - 1)) != 0 {
            return Err(QuantrError {
                message: String::from(
                    "The length of the array must be of the form 2**n where n is an integer.",
                ),
            });
        }

        Ok(SuperPosition {
            amplitudes: amplitudes.to_vec(),
            product_dim: length.trailing_zeros() as usize,
        })
    }

    /// Creates a superposition based on the complex amplitudes of each state labelled in
    /// the computational basis.
    ///
    /// # Example
    /// ```
    /// use std::collections::HashMap;
    /// use quantr::states::{Qubit, ProductState, SuperPosition};
    /// use quantr::{complex_re_array, complex_re};
    ///
    /// let prod = ProductState::new(&[Qubit::Zero, Qubit::One]).unwrap();
    /// let hash_amps = HashMap::from([(prod, complex_re!(1f64))]);
    /// let superpos = SuperPosition::new_with_hash_amplitudes(hash_amps).unwrap();
    ///
    /// assert_eq!(&complex_re_array![0f64, 1f64, 0f64, 0f64], superpos.get_amplitudes());
    /// ```
    pub fn new_with_hash_amplitudes(
        hash_amplitudes: HashMap<ProductState, Complex64>,
    ) -> QResult<SuperPosition> {
        if hash_amplitudes.is_empty() {
            return Err(QuantrError { message: String::from("An empty HashMap was given. A superposition must have at least one non-zero state.") });
        }

        let product_dim: usize = hash_amplitudes.keys().next().unwrap().num_qubits();
        let mut total_amplitude: f64 = 0f64;
        for (states, amplitude) in &hash_amplitudes {
            if states.num_qubits() != product_dim {
                return Err(QuantrError { message: format!("The first state has product dimension of {}, whilst the state, |{}>, found as a key in the HashMap has dimension {}.", product_dim, states, states.num_qubits()) });
            }
            total_amplitude += amplitude.norm_sqr();
        }

        if !Self::equal_within_error(total_amplitude, 1f64) {
            return Err(QuantrError { message: format!("The total sum of the absolute square of all amplitudes, {}, does not equal 1. That is, the superpositon does not conserve probability.", total_amplitude) });
        }

        let mut amplitudes: Vec<Complex64> = vec![num_complex::Complex64::ZERO; 1 << product_dim];
        Self::from_hash_to_array(hash_amplitudes, &mut amplitudes);
        Ok(SuperPosition {
            amplitudes,
            product_dim,
        })
    }

    /// Retrieves the coefficient of the product state in the computational basis given by the list index. Returns `None` if the
    /// index is greater than the product dimension of the superposition.
    ///
    /// # Example
    /// ```
    /// use quantr::states::SuperPosition;
    /// use quantr::{complex_re_array, complex_re};
    ///
    /// let superpos = SuperPosition::new_with_amplitudes(&complex_re_array![0f64, 1f64, 0f64, 0f64]).unwrap();
    ///
    /// assert_eq!(complex_re!(1f64), superpos.get_amplitude(1).unwrap());
    /// ```
    pub fn get_amplitude(&self, pos: usize) -> Option<Complex64> {
        self.amplitudes.get(pos).cloned()
    }

    /// Returns the number of qubits that each product state in the super position is composed of by using the Kronecker product.
    ///
    /// # Example
    /// ```
    /// use quantr::states::SuperPosition;
    /// use quantr::{complex_re_array, complex_re};
    ///
    /// let superpos = SuperPosition::new_with_amplitudes(&complex_re_array![0f64, 1f64, 0f64, 0f64]).unwrap();
    ///
    /// assert_eq!(2, superpos.get_num_qubits());
    /// ```
    pub fn get_num_qubits(&self) -> usize {
        self.product_dim
    }

    /// Returns the minimum dimension of the Hilbert space that the superposition can exist in.
    ///
    /// # Example
    /// ```
    /// use quantr::states::SuperPosition;
    /// use quantr::{complex_re_array, complex_re};
    ///
    /// let superpos = SuperPosition::new_with_amplitudes(&complex_re_array![0f64, 1f64, 0f64, 0f64]).unwrap();
    ///
    /// assert_eq!(4, superpos.get_dimension());
    /// ```
    pub fn get_dimension(&self) -> usize {
        self.amplitudes.len()
    }

    /// Returns a slice of the coefficients ordered in the computational basis of increasing order from
    /// left to right.
    ///
    /// # Example
    /// ```
    /// use quantr::states::SuperPosition;
    /// use quantr::{complex_re_array, complex_re};
    ///
    /// let superpos = SuperPosition::new(2).unwrap();
    ///
    /// assert_eq!(&complex_re_array![1f64, 0f64, 0f64, 0f64], superpos.get_amplitudes());
    /// ```
    pub fn get_amplitudes(&self) -> &[Complex64] {
        self.amplitudes.as_slice()
    }

    /// Retrieves the coefficient of the product state labelled in the computational basis.
    ///
    /// # Example
    /// ```
    /// use quantr::states::{Qubit, ProductState, SuperPosition};
    /// use quantr::{complex_re};
    ///
    /// let superpos = SuperPosition::new(2).unwrap();
    /// let prod_state = ProductState::new(&[Qubit::Zero, Qubit::Zero]).unwrap();
    ///
    /// assert_eq!(complex_re!(1f64), superpos.get_amplitude_from_state(prod_state).unwrap());
    /// ```
    pub fn get_amplitude_from_state(&self, prod_state: ProductState) -> QResult<Complex64> {
        if 2usize << (prod_state.qubits.len() - 1) != self.amplitudes.len() {
            return Err(QuantrError { message: format!("Unable to retreive product state, |{:?}> with dimension {}. The superposition is a linear combination of states with different dimension. These dimensions should be equal.", prod_state.to_string(), prod_state.num_qubits()),});
        }
        Ok(self.amplitudes[prod_state.comp_basis()])
    }

    /// Returns a new superposition in the computational basis.
    ///
    /// Checks to see if the amplitudes completely specify the amplitude of each state, in addition
    /// to it conserving probability.
    ///
    /// # Example
    /// ```
    /// use quantr::states::SuperPosition;
    /// use quantr::complex_re_array;
    ///
    /// let mut superpos = SuperPosition::new(2).unwrap();
    /// superpos.set_amplitudes(&complex_re_array![0f64, 1f64, 0f64, 0f64]).unwrap();
    ///
    /// assert_eq!(&complex_re_array![0f64, 1f64, 0f64, 0f64], superpos.get_amplitudes());
    /// ```
    pub fn set_amplitudes(&mut self, amplitudes: &[Complex64]) -> QResult<&mut SuperPosition> {
        if amplitudes.len() != self.amplitudes.len() {
            return Err(QuantrError {
                message: format!("The slice given to set the amplitudes in the computational basis has length {}, when it should have length {}.", amplitudes.len(), self.amplitudes.len()),
            });
        }

        if !Self::equal_within_error(amplitudes.iter().map(|x| x.norm_sqr()).sum::<f64>(), 1f64) {
            return Err(QuantrError {
                message: String::from("Slice given to set amplitudes in super position does not conserve probability, the absolute square sum of the coefficents must be one."),
            });
        }

        self.amplitudes = amplitudes.to_vec();
        Ok(self)
    }

    fn equal_within_error(num: f64, compare_num: f64) -> bool {
        num < compare_num + ZERO_MARGIN && num > compare_num - ZERO_MARGIN
    }

    /// Returns a superposition constructed from a HashMap with [ProductState] keys and amplitudes
    /// that are `Complex64` values.
    ///
    /// The amplitudes are checked for probability conservation, and that the product states are
    /// dimensionally consistent. States that are missing will assume to have zero amplitude.
    ///
    /// # Example
    /// ```
    /// use std::collections::HashMap;
    /// use quantr::states::{Qubit, ProductState, SuperPosition};
    /// use quantr::{complex_re_array, complex_re};
    ///
    /// let mut superpos = SuperPosition::new(2).unwrap();
    ///
    /// let prod_state = ProductState::new(&[Qubit::Zero, Qubit::One]).unwrap();
    /// let hash_states = HashMap::from([(prod_state, complex_re!(1f64))]);
    /// superpos.set_amplitudes_from_states(hash_states).unwrap();
    ///
    /// assert_eq!(&complex_re_array![0f64, 1f64, 0f64, 0f64], superpos.get_amplitudes());
    /// ```
    pub fn set_amplitudes_from_states(
        &mut self,
        amplitudes: HashMap<ProductState, Complex64>,
    ) -> QResult<&mut SuperPosition> {
        // Check if amplitudes and product states are correct.
        if amplitudes.is_empty() {
            return Err(QuantrError { message: String::from("An empty HashMap was given. A superposition must have at least one non-zero state.") });
        }

        let product_size: usize = self.amplitudes.len().trailing_zeros() as usize;
        let mut total_amplitude: f64 = 0f64;
        for (states, amplitude) in &amplitudes {
            if states.num_qubits() != product_size {
                return Err(QuantrError { message: format!("The first state has product dimension of {}, whilst the state, |{}>, found as a key in the HashMap has dimension {}.", product_size, states, states.num_qubits()) });
            }
            total_amplitude += amplitude.norm_sqr();
        }

        if !Self::equal_within_error(total_amplitude, 1f64) {
            return Err(QuantrError { message: String::from("The total sum of the absolute square of all amplitudes does not equal 1. That is, the superpositon does not conserve probability.") });
        }

        Self::from_hash_to_array(amplitudes, &mut self.amplitudes);

        Ok(self)
    }

    /// Creates a HashMap of the superposition with [ProductState] as keys.
    ///
    /// The HashMap will not include states with amplitudes that are near zero
    /// (with tolerance 1-e6 of the conjugate squared if the amplitude).
    ///
    /// # Example
    /// ```
    /// use std::collections::HashMap;
    /// use quantr::states::{Qubit, ProductState, SuperPosition};
    /// use quantr::complex_re;
    ///
    /// let mut superpos = SuperPosition::new(2).unwrap();
    ///
    /// let prod_state = ProductState::new(&[Qubit::Zero, Qubit::Zero]).unwrap();
    /// let hash_compare = HashMap::from([(prod_state, complex_re!(1f64))]);
    ///
    /// assert_eq!(hash_compare, superpos.to_hash_map());
    /// ```
    pub fn to_hash_map(&self) -> HashMap<ProductState, Complex64> {
        let mut super_pos_as_hash: HashMap<ProductState, Complex64> = Default::default();
        for (i, amp) in self.amplitudes.iter().enumerate() {
            if !Self::equal_within_error(amp.norm_sqr(), 0f64) {
                super_pos_as_hash.insert(ProductState::binary_basis(i, self.product_dim), *amp);
            }
        }
        super_pos_as_hash
    }

    /// Observe the superposition and return the measuremed state in the computational basis.
    ///
    /// If `None` is returned, then the state vector does not conserve probability. More
    /// precisely, the sum of the conjugate square of coefficients is less than one. The sum could
    /// be greater than one, however a `Some(Complex64)` type would be returned. The
    /// non-conservation of probability can happen due to the use of implementing non-unitary
    /// gates through `Custom::gate`.
    pub fn measure(&self) -> Option<ProductState> {
        let mut cummalitive: f64 = 0f64;
        let dice_roll: f64 = fastrand::f64();
        for (i, probability) in self.amplitudes.iter().map(|x| x.norm_sqr()).enumerate() {
            cummalitive += probability;
            if dice_roll < cummalitive {
                return Some(ProductState::binary_basis(i, self.product_dim));
            }
        }
        None
    }

    pub(super) fn from_hash_to_array(
        hash_amplitudes: HashMap<ProductState, Complex64>,
        vec_amplitudes: &mut Vec<Complex64>,
    ) {
        let length: usize = vec_amplitudes.len();
        let trailing_length: usize = length.trailing_zeros() as usize;
        for (i, amp) in vec_amplitudes.iter_mut().enumerate() {
            let key: ProductState = ProductState::binary_basis(i, trailing_length);
            match hash_amplitudes.get(&key) {
                Some(val) => *amp = *val,
                None => *amp = num_complex::Complex64::ZERO,
            }
        }
    }
}

impl From<ProductState> for SuperPosition {
    /// Returns the [ProductState] as a [SuperPosition].
    ///
    /// # Example
    /// ```
    /// use quantr::states::{Qubit, ProductState, SuperPosition};
    ///
    /// let prod_state = ProductState::new(&[Qubit::Zero, Qubit::Zero]).unwrap();
    /// let super_pos = SuperPosition::from(prod_state);
    /// ```
    fn from(value: ProductState) -> Self {
        SuperPosition::new_with_hash_amplitudes_unchecked(HashMap::from([(
            value,
            complex_re!(1f64),
        )]))
    }
}

impl From<Qubit> for SuperPosition {
    /// Returns the [] as a [SuperPosition].
    ///
    /// # Example
    /// ```
    /// use quantr::states::{Qubit, SuperPosition};
    ///
    /// let super_pos = SuperPosition::from(Qubit::Zero);
    /// ```
    fn from(value: Qubit) -> Self {
        SuperPosition::new_with_hash_amplitudes_unchecked(HashMap::from([(
            value.into(),
            complex_re!(1f64),
        )]))
    }
}

#[cfg(test)]
mod tests {
    use crate::circuit::HashMap;
    use crate::states::{ProductState, Qubit, SuperPosition};
    use crate::{complex_im, complex_re};
    use num_complex::Complex64;
    use std::f64::consts::FRAC_1_SQRT_2;

    #[test]
    fn retrieve_amplitude_from_state() {
        assert_eq!(
            SuperPosition::new_unchecked(2)
                .set_amplitudes(&[
                    num_complex::Complex64::ZERO,
                    complex_re!(FRAC_1_SQRT_2),
                    complex_im!(-FRAC_1_SQRT_2),
                    num_complex::Complex64::ZERO
                ])
                .unwrap()
                .get_amplitude_from_state(ProductState::new_unchecked(&[Qubit::Zero, Qubit::One]))
                .unwrap(),
            complex_re!(FRAC_1_SQRT_2)
        )
    }

    #[test]
    fn retrieve_amplitude_from_list_pos() {
        assert_eq!(
            SuperPosition::new_unchecked(2)
                .set_amplitudes(&[
                    num_complex::Complex64::ZERO,
                    complex_re!(FRAC_1_SQRT_2),
                    complex_im!(-FRAC_1_SQRT_2),
                    num_complex::Complex64::ZERO
                ])
                .unwrap()
                .get_amplitude(2)
                .unwrap(),
            complex_im!(-FRAC_1_SQRT_2)
        )
    }

    #[test]
    fn sets_amplitude_from_states() {
        let states: HashMap<ProductState, Complex64> = HashMap::from([
            (
                ProductState::new_unchecked(&[Qubit::Zero, Qubit::One]),
                complex_re!(FRAC_1_SQRT_2),
            ),
            (
                ProductState::new_unchecked(&[Qubit::One, Qubit::Zero]),
                complex_im!(-FRAC_1_SQRT_2),
            ),
        ]);

        assert_eq!(
            SuperPosition::new_with_amplitudes(&[
                num_complex::Complex64::ZERO,
                complex_re!(FRAC_1_SQRT_2),
                complex_im!(-FRAC_1_SQRT_2),
                num_complex::Complex64::ZERO
            ])
            .unwrap()
            .amplitudes,
            SuperPosition::new_with_hash_amplitudes(states)
                .unwrap()
                .amplitudes
        )
    }

    #[test]
    #[should_panic]
    fn sets_amplitude_from_states_wrong_dimension() {
        let states: HashMap<ProductState, Complex64> = HashMap::from([
            (
                ProductState::new_unchecked(&[Qubit::Zero, Qubit::One]),
                complex_re!(FRAC_1_SQRT_2),
            ),
            (
                ProductState::new_unchecked(&[Qubit::One, Qubit::Zero, Qubit::One]),
                complex_im!(-FRAC_1_SQRT_2),
            ),
        ]);

        SuperPosition::new_with_hash_amplitudes(states).unwrap();
    }

    #[test]
    #[should_panic]
    fn sets_amplitude_from_states_breaks_probability() {
        let states: HashMap<ProductState, Complex64> = HashMap::from([
            (
                ProductState::new_unchecked(&[Qubit::Zero, Qubit::One]),
                complex_re!(FRAC_1_SQRT_2),
            ),
            (
                ProductState::new_unchecked(&[Qubit::One, Qubit::Zero]),
                complex_im!(-FRAC_1_SQRT_2 * 0.5f64),
            ),
        ]);

        SuperPosition::new_with_hash_amplitudes(states).unwrap();
    }

    #[test]
    #[should_panic]
    fn catches_retrieve_amplitude_from_wrong_state() {
        SuperPosition::new_unchecked(2)
            .set_amplitudes(&[
                num_complex::Complex64::ZERO,
                complex_re!(FRAC_1_SQRT_2),
                complex_im!(-FRAC_1_SQRT_2),
                num_complex::Complex64::ZERO,
            ])
            .unwrap()
            .get_amplitude_from_state(ProductState::new_unchecked(&[
                Qubit::Zero,
                Qubit::One,
                Qubit::One,
            ]))
            .unwrap();
    }

    #[test]
    #[should_panic]
    fn catches_retrieve_amplitude_from_wrong_list_pos() {
        SuperPosition::new_unchecked(2)
            .set_amplitudes(&[
                num_complex::Complex64::ZERO,
                complex_re!(FRAC_1_SQRT_2),
                complex_im!(-FRAC_1_SQRT_2),
                num_complex::Complex64::ZERO,
            ])
            .unwrap()
            .get_amplitude(4)
            .unwrap();
    }

    #[test]
    #[should_panic]
    fn catches_super_position_breaking_conservation() {
        SuperPosition::new_unchecked(2)
            .set_amplitudes(&[
                num_complex::Complex64::ZERO,
                complex_re!(0.5f64),
                num_complex::Complex64::ZERO,
                complex_im!(-0.5f64),
            ])
            .unwrap();
    }
}
