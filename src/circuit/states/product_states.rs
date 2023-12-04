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
use crate::complex_Re;
use crate::states::{Qubit, SuperPosition};
use crate::{Complex, QuantrError};

/// A product state in the computational basis.
#[derive(Clone, Hash, PartialEq, Eq, Debug)]
pub struct ProductState {
    /// Each element of `Vec<Qubit>` is mapped to bra-ket notation like so:
    /// `Vec<Qubit>{a, b, ..., c} -> |a...bc>`
    pub qubits: Vec<Qubit>,
}

impl ProductState {
    /// Creates a single product state from a slice of qubits.
    ///
    /// The product state is mapped to bra-ket notation like so:
    /// `&[a, b, ..., c] -> |ab...c>`
    pub fn new(product_state: &[Qubit]) -> Result<ProductState, QuantrError> {
        if product_state.is_empty() {
            return Err(QuantrError {
                message: String::from(
                    "The slice of qubits is empty, it needs to at least have one element.",
                ),
            });
        }
        Ok(ProductState {
            qubits: product_state.to_vec(),
        })
    }

    // Unchecked version of new, doesn't need unwrapped.
    pub(crate) fn new_unchecked(product_state: &[Qubit]) -> ProductState {
        ProductState {
            qubits: product_state.to_vec(),
        }
    }

    // Changes the qubits at specified positions within the product state with a slice of other
    // qubits.
    pub(crate) fn insert_qubits(&self, qubits: &[Qubit], pos: &[usize]) -> ProductState {
        let mut edited_qubits: Vec<Qubit> = self.qubits.clone();
        let num_qubits: usize = qubits.len();

        if num_qubits != pos.len() {
            panic!("Size of qubits and positions must be equal.")
        }

        for (index, position) in pos.iter().enumerate() {
            edited_qubits[*position] = qubits[index];
        }
        ProductState::new_unchecked(&edited_qubits)
    }

    // Returns the dimension of the product state.
    pub(super) fn num_qubits(&self) -> usize {
        self.qubits.len()
    }

    /// Inverts a binary digit that represents the product state.
    ///
    /// The position index starts from the far most left qubit. An error will be returned if the
    /// position is larger or equal to the product dimension of the state.
    pub fn invert_digit(&mut self, place_num: usize) -> Result<(), QuantrError> {
        if place_num >= self.num_qubits() {
            return Err(QuantrError { message: format!("The position of the binary digit, {}, is out of bounds. The product dimension is {}, and so the position must be strictly less.", place_num, self.num_qubits()) });
        }

        let old_qubit: Qubit = self.qubits[place_num].clone();
        self.qubits[place_num] = if old_qubit == Qubit::Zero {
            Qubit::One
        } else {
            Qubit::Zero
        };
        Ok(())
    }

    /// Concatenate a product state with a qubit.
    ///
    /// In effect, this is using the Kronecker product to create a new state.
    pub fn kronecker_prod(mut self, other: Qubit) -> ProductState {
        self.qubits.push(other);
        self
    }

    // Returns the qubit in the product state given a position.
    pub(crate) fn get(&self, qubit_number: usize) -> Qubit {
        self.qubits[qubit_number]
    }

    /// Returns the labelling of the product state as a String.
    pub fn to_string(&self) -> String {
        self.qubits
            .iter()
            .map(|q| match q {
                Qubit::Zero => "0",
                Qubit::One => "1",
            })
            .collect::<String>()
    }

    /// Returns the [ProductState] as a [SuperPosition].
    pub fn into_super_position(self) -> SuperPosition {
        SuperPosition::new_with_hash_amplitudes_unchecked(HashMap::from([(
            self,
            complex_Re!(1f64),
        )]))
    }

    // Converts the computational basis labelling (a binary integer), into base 10.
    pub(super) fn comp_basis(&self) -> usize {
        self.qubits
            .iter()
            .rev()
            .enumerate()
            .map(|(pos, i)| match i {
                Qubit::Zero => 0u32,
                Qubit::One => 1 << pos,
            })
            .fold(0, |sum, i| sum + i) as usize
    }

    // Produces a product states based on converting a base 10 number to binary, where the product
    // state in the computational basis is defined from this labelling.
    pub(super) fn binary_basis(index: usize, basis_size: usize) -> ProductState {
        let binary_index: Vec<Qubit> = (0..basis_size)
            .rev()
            .map(|n| match (index >> n) & 1 == 1 {
                false => Qubit::Zero,
                true => Qubit::One,
            })
            .collect();

        ProductState::new_unchecked(binary_index.as_slice())
    }
}

#[cfg(test)]
mod tests {
    use crate::states::{ProductState, Qubit, SuperPosition};
    use crate::{complex_Re, Complex, COMPLEX_ZERO};

    #[test]
    fn converts_from_integer_to_product_state() {
        assert_eq!(
            ProductState::new_unchecked(&[Qubit::One, Qubit::One, Qubit::Zero]),
            ProductState::binary_basis(6, 3)
        )
    }

    #[test]
    fn inverting_binary_digit() {
        let mut inverted = ProductState::new_unchecked(&[Qubit::One, Qubit::One, Qubit::Zero]);
        inverted.invert_digit(2).unwrap();
        assert_eq!(
            ProductState::new_unchecked(&[Qubit::One, Qubit::One, Qubit::One]),
            inverted
        )
    }

    #[test]
    fn insert_qubits_in_state() {
        assert_eq!(
            ProductState::new_unchecked(&[Qubit::Zero, Qubit::Zero, Qubit::One]).qubits,
            ProductState::new_unchecked(&[Qubit::One, Qubit::One, Qubit::One])
                .insert_qubits(&[Qubit::Zero, Qubit::Zero], &[0, 1])
                .qubits
        );
    }

    #[test]
    fn converts_from_binary_to_comp_basis() {
        assert_eq!(
            ProductState::new_unchecked(&[Qubit::One, Qubit::Zero, Qubit::One]).comp_basis(),
            5usize
        );
        assert_eq!(
            ProductState::new_unchecked(&[Qubit::One, Qubit::One, Qubit::One]).comp_basis(),
            7usize
        );
        assert_eq!(
            ProductState::new_unchecked(&[Qubit::One, Qubit::Zero]).comp_basis(),
            2usize
        );
        assert_eq!(
            ProductState::new_unchecked(&[Qubit::One, Qubit::Zero, Qubit::One, Qubit::One])
                .comp_basis(),
            11usize
        );
    }

    #[test]
    fn converts_productstate_to_superpos() {
        assert_eq!(
            &mut ProductState::new_unchecked(&[Qubit::One, Qubit::Zero]).into_super_position(),
            SuperPosition::new(2)
                .set_amplitudes(&[COMPLEX_ZERO, COMPLEX_ZERO, complex_Re!(1f64), COMPLEX_ZERO])
                .unwrap()
        )
    }
}
