/*
* Copyright (c) 2023 Andrew Rowan Barlow. Licensed under the EUPL-1.2
* or later. You may obtain a copy of the licence at
* https://joinup.ec.europa.eu/collection/eupl/eupl-text-eupl-12. A copy
* of the EUPL-1.2 licence in English is given in LICENCE.txt which is
* found in the root directory of this repository.
*
* Author: Andrew Rowan Barlow <a.barlow.dev@gmail.com>
*/

use crate::circuit::QResult;
use crate::states::Qubit;
use crate::QuantrError;

/// A product state in the computational basis.
#[derive(Clone, Hash, PartialEq, Eq, Debug)]
pub struct ProductState {
    /// Each element of `Vec<Qubit>` is mapped to bra-ket notation like so:
    /// `Vec<Qubit>{a, b, ..., c} -> |ab...c>`
    pub(crate) qubits: Vec<Qubit>,
}

impl ProductState {
    /// Creates a single product state from a slice of qubits.
    ///
    /// The product state is mapped to bra-ket notation like so:
    /// `&[a, b, ..., c] -> |ab...c>`
    ///
    /// # Example
    /// ```
    /// use quantr::states::{Qubit, ProductState};
    ///
    /// let prod: ProductState = ProductState::new(&[Qubit::One, Qubit::Zero]).unwrap(); // |10>
    /// ```
    pub fn new(product_state: &[Qubit]) -> QResult<ProductState> {
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

    /// Returns the qubit in the ith position, counting from the left of the ket notation.
    ///
    /// None is returned if the index is out of range, that is the index is greater than the number
    /// of qubits that defines the product state.
    ///
    /// # Example
    /// ```
    /// use quantr::states::{Qubit, ProductState};
    ///
    /// let prod: ProductState = ProductState::new(&[Qubit::One, Qubit::Zero]).unwrap();
    ///
    /// assert_eq!(Some(Qubit::Zero), prod.get(1).copied());
    /// assert_eq!(None, prod.get(2));
    /// ```
    pub fn get(&self, i: usize) -> Option<&Qubit> {
        self.qubits.get(i)
    }

    /// Returns a slice of the qubits that forms the product state.
    ///
    /// See [ProductState::new] for the mapping.
    ///
    /// # Example
    /// ```
    /// use quantr::states::{Qubit, ProductState};
    ///
    /// let prod: ProductState = ProductState::new(&[Qubit::One, Qubit::Zero]).unwrap();
    ///
    /// assert_eq!(&[Qubit::One, Qubit::Zero], prod.get_qubits());
    /// ```
    pub fn get_qubits(&self) -> &[Qubit] {
        self.qubits.as_slice()
    }

    /// Returns a mutable slice of the qubits that forms the product state. This can be used to
    /// directly change the elements within the slice that form the `ProductState`.
    ///
    /// See [ProductState::new] for the mapping.
    ///
    /// # Example
    /// ```
    /// use quantr::states::{Qubit, ProductState};
    ///
    /// let mut prod: ProductState = ProductState::new(&[Qubit::One, Qubit::Zero]).unwrap();
    ///
    /// prod.get_mut_qubits()[1] = Qubit::One;
    ///
    /// assert_eq!(&[Qubit::One, Qubit::One], prod.get_qubits());
    /// ```
    pub fn get_mut_qubits(&mut self) -> &mut [Qubit] {
        self.qubits.as_mut_slice()
    }

    // Unchecked version of new, doesn't need unwrapped.
    pub(crate) fn new_unchecked(product_state: &[Qubit]) -> ProductState {
        ProductState {
            qubits: product_state.to_vec(),
        }
    }

    // Changes the qubits at specified positions within the product state with a slice of other
    // qubits.
    pub(crate) fn insert_qubits(&mut self, qubits: &[Qubit], pos: &[usize]) {
        //let mut edited_qubits: Vec<Qubit> = self.qubits.clone();

        for (enum_i, &i) in pos.iter().enumerate() {
            if self.qubits[i] != qubits[enum_i] {
                self.qubits[i] = match self.qubits[i] {
                    Qubit::Zero => Qubit::One,
                    Qubit::One => Qubit::Zero,
                };
            }
        }
    }

    /// Returns the number of qubits that form the product state.
    ///
    /// # Example
    /// ```
    /// use quantr::states::{Qubit, ProductState};
    ///
    /// let prod: ProductState = ProductState::new(&[Qubit::One, Qubit::Zero, Qubit::One]).unwrap();
    ///
    /// assert_eq!(3, prod.num_qubits());
    /// ```
    pub fn num_qubits(&self) -> usize {
        self.qubits.len()
    }

    /// Inverts a binary digit that represents the product state.
    ///
    /// The position index starts from the far most left qubit. An error will be returned if the
    /// position is larger or equal to the product dimension of the state.
    ///
    /// # Example
    /// ```
    /// use quantr::states::{Qubit, ProductState};
    ///
    /// let mut prod: ProductState = ProductState::new(&[Qubit::One, Qubit::Zero, Qubit::One]).unwrap();
    ///
    /// prod.invert_digit(1);
    ///
    /// assert_eq!(&[Qubit::One, Qubit::One, Qubit::One], prod.get_qubits());
    /// ```
    pub fn invert_digit(&mut self, place_num: usize) -> QResult<&mut ProductState> {
        if place_num >= self.num_qubits() {
            return Err(QuantrError { message: format!("The position of the binary digit, {}, is out of bounds. The product dimension is {}, and so the position must be strictly less.", place_num, self.num_qubits()) });
        }

        let old_qubit: Qubit = self.qubits[place_num];
        self.qubits[place_num] = if old_qubit == Qubit::Zero {
            Qubit::One
        } else {
            Qubit::Zero
        };
        Ok(self)
    }

    /// Performs the Kronecker product of a product state with a qubit on the RHS.
    ///
    /// # Example
    /// ```
    /// use quantr::states::{Qubit, ProductState};
    ///
    /// let mut prod: ProductState = ProductState::new(&[Qubit::Zero, Qubit::Zero]).unwrap();
    ///
    /// let new_prod = prod.kronecker_prod(Qubit::One);
    ///
    /// assert_eq!(&[Qubit::Zero, Qubit::Zero, Qubit::One], new_prod.get_qubits());
    /// ```
    pub fn kronecker_prod(mut self, other: Qubit) -> ProductState {
        self.qubits.push(other);
        self
    }

    // Returns the qubit in the product state given a position.
    pub(crate) fn get_unchecked(&self, qubit_number: usize) -> Qubit {
        self.qubits[qubit_number]
    }

    /// Returns the labelling of the product state as a String.
    ///
    /// # Example
    /// ```
    /// use quantr::states::{Qubit, ProductState};
    ///
    /// let prod: ProductState = ProductState::new(&[Qubit::Zero, Qubit::One]).unwrap();
    ///
    /// assert_eq!(String::from("01"), prod.to_string());
    /// ```
    #[allow(clippy::inherent_to_string)]
    pub fn to_string(&self) -> String {
        self.qubits
            .iter()
            .map(|q| match q {
                Qubit::Zero => "0",
                Qubit::One => "1",
            })
            .collect::<String>()
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
            .sum::<u32>() as usize
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

impl From<Qubit> for ProductState {
    /// Converts the [Qubit] to a [ProductState] struct.
    ///
    /// # Example
    /// ```
    /// use quantr::states::{Qubit, ProductState};
    ///
    /// let prod: ProductState = ProductState::from(Qubit::One);
    ///
    /// assert_eq!(&[Qubit::One], prod.get_qubits());
    /// ```
    fn from(value: Qubit) -> Self {
        ProductState::new_unchecked(&[value])
    }
}

#[cfg(test)]
mod tests {
    use crate::states::{ProductState, Qubit, SuperPosition};
    use crate::{complex_re, Complex, COMPLEX_ZERO};

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
        let mut prod = ProductState::new_unchecked(&[Qubit::One, Qubit::One, Qubit::One]);
        prod.insert_qubits(&[Qubit::Zero, Qubit::Zero], &[0, 2]);
        assert_eq!(
            ProductState::new_unchecked(&[Qubit::Zero, Qubit::One, Qubit::Zero]).qubits,
            prod.qubits
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
            &mut SuperPosition::from(ProductState::new_unchecked(&[Qubit::One, Qubit::Zero])),
            SuperPosition::new_unchecked(2)
                .set_amplitudes(&[COMPLEX_ZERO, COMPLEX_ZERO, complex_re!(1f64), COMPLEX_ZERO])
                .unwrap()
        )
    }
}
