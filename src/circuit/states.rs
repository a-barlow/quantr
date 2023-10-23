/*
* Copyright (c) 2023 Andrew Rowan Barlow. Licensed under the EUPL-1.2
* or later. You may obtain a copy of the licence at
* https://joinup.ec.europa.eu/collection/eupl/eupl-text-eupl-12. A copy
* of the EUPL-1.2 licence in English is given in LICENCE.txt which is
* found in the root directory of this repository.
*
* Author: Andrew Rowan Barlow <a.barlow.dev@gmail.com>
*/

//! Sates, product states and superpositions of qubits.
//!
//! The mapping of circuit to product state in the computational basis is defined like so:
//!
//! ``` text
//! |a⟩ ────
//! |b⟩ ────  ⟺ |a,b,c,⋯⟩ ≡ |a⟩⊗|b⟩⊗|c⟩⊗⋯
//! |c⟩ ────
//!  ⋮    ⋮
//!  ```

use crate::complex::Complex;
use crate::QuantrError;
use crate::{complex_Re, complex_zero};
use std::collections::HashMap;
use std::hash::Hash;

/// The fundamental unit in quantum computers, the qubit.
#[derive(Clone, Copy, Hash, PartialEq, Eq, Debug)]
pub enum Qubit {
    /// |0⟩
    Zero,
    /// |1⟩
    One,
}

impl Qubit {
    /// Defines the tensor product on two qubits.
    ///
    /// # Example
    /// ```
    /// use quantr::circuit::states::{ProductState, Qubit};
    ///
    /// let qubit_a: Qubit = Qubit::Zero; // |0>
    /// let qubit_b: Qubit = Qubit::One;  // |1>
    ///
    /// let new_product: ProductState = qubit_a.join(qubit_b); // |0> ⊗ |1> = |01>
    /// assert_eq!(new_product.state.as_slice(), &[Qubit::Zero, Qubit::One])
    /// ```
    pub fn join(self, other: Qubit) -> ProductState {
        ProductState::new(&[self, other])
    }

    /// Converts the [Qubit] to a [ProductState] struct.
    pub fn as_state(self) -> ProductState {
        ProductState::new(&[self])
    }
}

/// A product state in the computational basis.
///
/// The product state usually results from the action of the tensor product on two [Qubit]s, but
/// may also result from the direct conversion of the computational base labelling.
#[derive(Clone, Hash, PartialEq, Eq, Debug)]
pub struct ProductState {
    /// Each element of `state` is mapped to bra-ket notation like so:
    /// `Vec<Qubit>{a, b, ..., c} -> |a...bc>`
    pub state: Vec<Qubit>,
}

impl ProductState {
    // This should return a result. Would be a breaking change, and possibl slow down the main
    // algorithm.
    /// Creates a single product state from a slice of qubits.
    ///
    /// The product state is mapped to bra-ket notation like so:
    /// `&[a, b, ..., c] -> |ab...c>`
    pub fn new(product_state: &[Qubit]) -> ProductState {
        ProductState {
            state: product_state.to_vec(),
        }
    }

    // This operation could just be changed to inverting the qubits, as qubits can only be in two
    // states.
    // Changes the qubits at specified positions within the product state with a slice of other
    // qubits.
    pub(super) fn insert_qubits(&self, qubits: &[Qubit], pos: &[usize]) -> ProductState {
        let mut edited_qubits: Vec<Qubit> = self.state.clone();
        let num_qubits: usize = qubits.len();

        if num_qubits != pos.len() {
            panic!("Size of qubits and positions must be equal.")
        }

        for (index, position) in pos.iter().enumerate() {
            edited_qubits[*position] = qubits[index];
        }
        ProductState::new(&edited_qubits)
    }

    // Returns the dimension of the product state.
    fn num_qubits(&self) -> usize {
        self.state.len()
    }

    /// Inverts a binary digit that represents the product state.
    ///
    /// The position counting starts from the far most left qubit. An error will be returned if the
    /// position is larger or equal to the product dimension of the state.
    pub fn invert_digit(&mut self, place_num: usize) -> Result<(), QuantrError> {
        if place_num >= self.num_qubits() {
            return Err(QuantrError { message: format!("The position of the binary digit, {}, is out of bounds. The product dimension is {}, and so the position must be strictly less.", place_num, self.num_qubits()) });
        }

        let old_qubit: Qubit = self.state[place_num].clone();
        self.state[place_num] = if old_qubit == Qubit::Zero {
            Qubit::One
        } else {
            Qubit::Zero
        };
        Ok(())
    }

    /// Concatenate a product state with a qubit.
    ///
    /// In effect, this is using the tensor prodcut to create a new state.
    pub fn join(mut self, other: Qubit) -> ProductState {
        self.state.push(other);
        self
    }

    // Returns the qubit in the product state given a position.
    pub(super) fn get(&self, qubit_number: usize) -> Qubit {
        self.state[qubit_number]
    }

    /// Returns the labelling of the product state as a String.
    pub fn as_string(&self) -> String {
        self.state
            .iter()
            .map(|q| match q {
                Qubit::Zero => "0",
                Qubit::One => "1",
            })
            .collect::<String>()
    }

    /// Returns the [ProductState] as a [SuperPosition].
    pub fn to_super_position(self) -> SuperPosition {
        SuperPosition::new(self.num_qubits())
            .set_amplitudes_from_states_unchecked(&HashMap::from([(self, complex_Re!(1f64))]))
    }

    // Converts the computational basis labelling (a binary integer), into base 10.
    fn comp_basis(&self) -> usize {
        self.state
            .iter()
            .rev()
            .enumerate()
            .map(|(pos, i)| match i {
                Qubit::Zero => 0u32,
                Qubit::One => 2u32.pow(pos as u32),
            })
            .fold(0, |sum, i| sum + i) as usize
    }

    // Produces a product states based on converting a base 10 number to binary, where the product
    // state in the computational basis is defined from this labelling.
    fn binary_basis(index: usize, basis_size: usize) -> ProductState {
        let binary_index: Vec<Qubit> = (0..basis_size)
            .rev()
            .map(|n| match (index >> n) & 1 == 1 {
                false => Qubit::Zero,
                true => Qubit::One,
            })
            .collect();

        ProductState::new(binary_index.as_slice())
    }
}

/// A superposition of [ProductState]s.
///
/// The ordering `amplitudes` correspond to state vectors in the computational basis.
#[derive(PartialEq, Debug, Clone)]
pub struct SuperPosition {
    pub amplitudes: Vec<Complex<f64>>,
    product_dim: usize,
    index: usize,
}

/// Returns the product state and it's respective amplitude in each iteration.
///
/// # Example
/// ```
/// use quantr::circuit::states::{ProductState, Qubit, SuperPosition};
/// use quantr::{complex_Re, complex_Re_vec, complex_zero, complex::Complex};
/// use std::f64::consts::FRAC_1_SQRT_2;
///
/// let super_pos: SuperPosition = SuperPosition::new(2)
///                                 .set_amplitudes(&complex_Re_vec!(0f64, FRAC_1_SQRT_2, FRAC_1_SQRT_2, 0f64)).unwrap();
///
/// let mut iterator_super_pos = super_pos.into_iter();
///
/// assert_eq!(iterator_super_pos.next(),
///     Some((ProductState::new(&[Qubit::Zero, Qubit::Zero]), complex_zero!())));
/// assert_eq!(iterator_super_pos.next(),
///     Some((ProductState::new(&[Qubit::Zero, Qubit::One]), complex_Re!(FRAC_1_SQRT_2))));
/// assert_eq!(iterator_super_pos.next(),
///     Some((ProductState::new(&[Qubit::One, Qubit::Zero]), complex_Re!(FRAC_1_SQRT_2))));
/// assert_eq!(iterator_super_pos.next(),
///     Some((ProductState::new(&[Qubit::One, Qubit::One]), complex_zero!())));
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

impl SuperPosition {
    const ERROR_MARGIN: f64 = 0.00000001f64;

    /// Creates a superposition in the |0..0> state.
    pub fn new(num_qubits: usize) -> SuperPosition {
        let mut new_amps: Vec<Complex<f64>> = vec![complex_zero!(); 2usize.pow(num_qubits as u32)];
        new_amps[0] = complex_Re!(1f64);
        SuperPosition {
            amplitudes: new_amps,
            product_dim: num_qubits,
            index: 0,
        }
    }

    /// Retrieves the coefficient of the product state given by the list index.
    pub fn get_amplitude(&self, pos: usize) -> Result<Complex<f64>, QuantrError> {
        if pos >= self.amplitudes.len() {
            let length = self.amplitudes.len();
            Err(QuantrError { message: format!("Failed to retrieve amplitude from list. Index given was, {pos}, which is greater than length of list, {length}."), 
            })
        } else {
            Ok(*self.amplitudes.get(pos).unwrap())
        }
    }

    /// Retrieves the coefficient of the product state labelled in the computational basis.
    pub fn get_amplitude_from_state(
        &self,
        prod_state: ProductState,
    ) -> Result<Complex<f64>, QuantrError> {
        if 2usize.pow(prod_state.state.len() as u32) != self.amplitudes.len() {
            return Err(QuantrError { message: format!("Unable to retreive product state, |{:?}> with dimension {}. The superposition is a linear combination of states with different dimension. These dimensions should be equal.", prod_state.as_string(), prod_state.num_qubits()),});
        }
        Ok(*self.amplitudes.get(prod_state.comp_basis()).unwrap())
    }

    /// Returns a new superposition in the computational basis.
    ///
    /// Checks to see if the amplitudes completely specify the amplitude of each state, in addition
    /// to it conserving probability.
    pub fn set_amplitudes(self, amplitudes: &[Complex<f64>]) -> Result<SuperPosition, QuantrError> {
        if amplitudes.len() != self.amplitudes.len() {
            return Err(QuantrError {
                message: format!("The slice given to set the amplitudes in the computational basis has length {}, when it should have length {}.", amplitudes.len(), self.amplitudes.len()),
            });
        }

        if !Self::equal_within_error(amplitudes.iter().map(|x| x.abs_square()).sum::<f64>(), 1f64) {
            return Err(QuantrError {
                message: String::from("Slice given to set amplitudes in super position does not conserve probability, the absolute square sum of the coefficents must be one."),
            });
        }

        let mut new_amps: Vec<Complex<f64>> = (*self.amplitudes).to_vec();
        Self::copy_slice_to_vec(&mut new_amps, amplitudes);
        Ok(SuperPosition {
            amplitudes: new_amps,
            product_dim: self.product_dim,
            index: 0,
        })
    }

    fn copy_slice_to_vec(vector: &mut Vec<Complex<f64>>, slice: &[Complex<f64>]) {
        for (pos, amp) in slice.iter().enumerate() {
            vector[pos] = *amp;
        }
    }

    fn equal_within_error(num: f64, compare_num: f64) -> bool {
        num < compare_num + Self::ERROR_MARGIN && num > compare_num - Self::ERROR_MARGIN
    }

    /// Returns a superposition constructed from a HashMap with [ProductState] keys and amplitudes
    /// that are `Complex<f64>` values.
    ///
    /// The amplitudes are checked for probability conservation, and that the product states are
    /// dimensionally consistent. States that are missing will assume to have zero amplitude.
    pub fn set_amplitudes_from_states(
        &self,
        amplitudes: &HashMap<ProductState, Complex<f64>>,
    ) -> Result<SuperPosition, QuantrError> {
        // Check if amplitudes and product states are correct.
        if amplitudes.is_empty() {
            return Err(QuantrError { message: String::from("An empty HashMap was given. A superposition must have at least one non-zero state.") });
        }

        let product_size: usize = self.amplitudes.len().trailing_zeros() as usize;
        let mut total_amplitude: f64 = 0f64;
        for (states, amplitude) in amplitudes {
            if states.num_qubits() != product_size {
                return Err(QuantrError { message: format!("The first state has product dimension of {}, whilst the state, |{}>, found as a key in the HashMap has dimension {}.", product_size, states.as_string(), states.num_qubits()) });
            }
            total_amplitude += amplitude.abs_square();
        }

        if !Self::equal_within_error(total_amplitude, Self::ERROR_MARGIN) {
            return Err(QuantrError { message: String::from("The total sum of the absolute square of all amplitudes does not equal 1. That is, the superpositon does not conserve probability.") });
        }

        // Start conversion

        let mut new_amps: Vec<Complex<f64>> =
            vec![complex_zero!(); 2usize.pow(product_size as u32)];

        Self::from_hash_to_array(amplitudes, &mut new_amps);

        Ok(SuperPosition {
            amplitudes: new_amps,
            product_dim: self.product_dim,
            index: 0,
        })
    }

    // Sets the amplitudes of a [SuperPosition] from a HashMap.
    // States that are missing from the HashMap will be assumed to have 0 amplitude.
    pub(super) fn set_amplitudes_from_states_unchecked(
        &self,
        amplitudes: &HashMap<ProductState, Complex<f64>>,
    ) -> SuperPosition {
        let product_size: usize = amplitudes.keys().next().unwrap().num_qubits();
        let mut new_amps: Vec<Complex<f64>> =
            vec![complex_zero!(); 2usize.pow(product_size as u32)];

        Self::from_hash_to_array(amplitudes, &mut new_amps);

        SuperPosition {
            amplitudes: new_amps,
            product_dim: self.product_dim,
            index: 0,
        }
    }

    /// Creates a HashMap of the superposition with [ProductState] as keys.
    ///
    /// The HashMap will not include states with amplitudes that are near zero.
    pub fn as_hash_map(&self) -> HashMap<ProductState, Complex<f64>> {
        let mut super_pos_as_hash: HashMap<ProductState, Complex<f64>> = Default::default();
        for (i, amp) in self.amplitudes.iter().enumerate() {
            if !Self::equal_within_error(amp.abs_square(), 0f64) {
                super_pos_as_hash.insert(ProductState::binary_basis(i, self.product_dim), *amp);
            }
        }
        super_pos_as_hash
    }

    fn from_hash_to_array(
        hash_amplitudes: &HashMap<ProductState, Complex<f64>>,
        vec_amplitudes: &mut Vec<Complex<f64>>,
    ) {
        for (key, val) in hash_amplitudes {
            vec_amplitudes[key.comp_basis()] = *val;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::complex_Im;
    use std::f64::consts::FRAC_1_SQRT_2;

    #[test]
    fn converts_productstate_to_superpos() {
        assert_eq!(
            ProductState::new(&[Qubit::One, Qubit::Zero]).to_super_position(),
            SuperPosition::new(2)
                .set_amplitudes(&[
                    complex_zero!(),
                    complex_zero!(),
                    complex_Re!(1f64),
                    complex_zero!()
                ])
                .unwrap()
        )
    }

    #[test]
    fn converts_from_binary_to_comp_basis() {
        assert_eq!(
            ProductState::new(&[Qubit::One, Qubit::Zero, Qubit::One]).comp_basis(),
            5usize
        );
        assert_eq!(
            ProductState::new(&[Qubit::One, Qubit::One, Qubit::One]).comp_basis(),
            7usize
        );
        assert_eq!(
            ProductState::new(&[Qubit::One, Qubit::Zero]).comp_basis(),
            2usize
        );
        assert_eq!(
            ProductState::new(&[Qubit::One, Qubit::Zero, Qubit::One, Qubit::One]).comp_basis(),
            11usize
        );
    }

    #[test]
    fn retrieve_amplitude_from_state() {
        assert_eq!(
            SuperPosition::new(2)
                .set_amplitudes(&[
                    complex_zero!(),
                    complex_Re!(FRAC_1_SQRT_2),
                    complex_Im!(-FRAC_1_SQRT_2),
                    complex_zero!()
                ])
                .unwrap()
                .get_amplitude_from_state(ProductState::new(&[Qubit::Zero, Qubit::One]))
                .unwrap(),
            complex_Re!(FRAC_1_SQRT_2)
        )
    }

    #[test]
    fn retrieve_amplitude_from_list_pos() {
        assert_eq!(
            SuperPosition::new(2)
                .set_amplitudes(&[
                    complex_zero!(),
                    complex_Re!(FRAC_1_SQRT_2),
                    complex_Im!(-FRAC_1_SQRT_2),
                    complex_zero!()
                ])
                .unwrap()
                .get_amplitude(2)
                .unwrap(),
            complex_Im!(-FRAC_1_SQRT_2)
        )
    }

    #[test]
    fn insert_qubits_in_state() {
        assert_eq!(
            ProductState::new(&[Qubit::Zero, Qubit::Zero, Qubit::One]).state,
            ProductState::new(&[Qubit::One, Qubit::One, Qubit::One])
                .insert_qubits(&[Qubit::Zero, Qubit::Zero], &[0, 1])
                .state
        );
    }

    #[test]
    fn sets_amplitude_from_states() {
        let states: HashMap<ProductState, Complex<f64>> = HashMap::from([
            (
                ProductState::new(&[Qubit::Zero, Qubit::One]),
                complex_Re!(FRAC_1_SQRT_2),
            ),
            (
                ProductState::new(&[Qubit::One, Qubit::Zero]),
                complex_Im!(-FRAC_1_SQRT_2),
            ),
        ]);

        assert_eq!(
            SuperPosition::new(2)
                .set_amplitudes(&[
                    complex_zero!(),
                    complex_Re!(FRAC_1_SQRT_2),
                    complex_Im!(-FRAC_1_SQRT_2),
                    complex_zero!()
                ])
                .unwrap()
                .amplitudes,
            SuperPosition::new(2)
                .set_amplitudes_from_states_unchecked(&states)
                .amplitudes
        )
    }

    #[test]
    #[should_panic]
    fn sets_amplitude_from_states_wrong_dimension() {
        let states: HashMap<ProductState, Complex<f64>> = HashMap::from([
            (
                ProductState::new(&[Qubit::Zero, Qubit::One]),
                complex_Re!(FRAC_1_SQRT_2),
            ),
            (
                ProductState::new(&[Qubit::One, Qubit::Zero, Qubit::One]),
                complex_Im!(-FRAC_1_SQRT_2),
            ),
        ]);

        SuperPosition::new(2)
            .set_amplitudes_from_states(&states)
            .unwrap();
    }

    #[test]
    #[should_panic]
    fn sets_amplitude_from_states_breaks_probability() {
        let states: HashMap<ProductState, Complex<f64>> = HashMap::from([
            (
                ProductState::new(&[Qubit::Zero, Qubit::One]),
                complex_Re!(FRAC_1_SQRT_2),
            ),
            (
                ProductState::new(&[Qubit::One, Qubit::Zero]),
                complex_Im!(-FRAC_1_SQRT_2 * 0.5f64),
            ),
        ]);

        SuperPosition::new(2)
            .set_amplitudes_from_states(&states)
            .unwrap();
    }

    #[test]
    #[should_panic]
    fn catches_retrieve_amplitude_from_wrong_state() {
        SuperPosition::new(2)
            .set_amplitudes(&[
                complex_zero!(),
                complex_Re!(FRAC_1_SQRT_2),
                complex_Im!(-FRAC_1_SQRT_2),
                complex_zero!(),
            ])
            .unwrap()
            .get_amplitude_from_state(ProductState::new(&[Qubit::Zero, Qubit::One, Qubit::One]))
            .unwrap();
    }

    #[test]
    #[should_panic]
    fn catches_retrieve_amplitude_from_wrong_list_pos() {
        SuperPosition::new(2)
            .set_amplitudes(&[
                complex_zero!(),
                complex_Re!(FRAC_1_SQRT_2),
                complex_Im!(-FRAC_1_SQRT_2),
                complex_zero!(),
            ])
            .unwrap()
            .get_amplitude(4)
            .unwrap();
    }

    #[test]
    #[should_panic]
    fn catches_super_position_breaking_conservation() {
        SuperPosition::new(2)
            .set_amplitudes(&[
                complex_zero!(),
                complex_Re!(0.5f64),
                complex_zero!(),
                complex_Im!(-0.5f64),
            ])
            .unwrap();
    }

    #[test]
    fn converts_from_integer_to_product_state() {
        assert_eq!(
            ProductState::new(&[Qubit::One, Qubit::One, Qubit::Zero]),
            ProductState::binary_basis(6, 3)
        )
    }

    #[test]
    fn inverting_binary_digit() {
        let mut inverted = ProductState::new(&[Qubit::One, Qubit::One, Qubit::Zero]);
        inverted.invert_digit(2).unwrap();
        assert_eq!(
            ProductState::new(&[Qubit::One, Qubit::One, Qubit::One]),
            inverted
        )
    }
}
