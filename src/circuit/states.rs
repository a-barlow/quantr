/*
* Copyright (c) 2024 Andrew Rowan Barlow. Licensed under the EUPL-1.2
* or later. You may obtain a copy of the licence at
* https://joinup.ec.europa.eu/collection/eupl/eupl-text-eupl-12. A copy
* of the EUPL-1.2 licence in English is given in LICENCE.txt which is
* found in the root directory of this repository.
*
* Author: Andrew Rowan Barlow <a.barlow.dev@gmail.com>
*/

//! Defines the qubit, product states and super positions including relevant operations.
//!
//! The [Qubit] is constructed as an enum, with variants [Qubit::Zero] and [Qubit::One] to represent
//! |0> and |1> respectively.
//!
//! The [ProductState] gives structure to a slice of qubits which represents a state in the
//! computational basis. For example the product state |01> which can be constructed from performing the Kronecker
//! product on |0> and |1>. Another example of a [ProductState] is |01101>.
//!
//! Finally, the [SuperPosition] represents a linear combination of [ProductState] with [crate::Complex] as
//! coefficients. The sum of the absolute conjugate square of each coefficient is 1 (conservation of
//! probability).
//!
//! These three objects all have operations that help manipulate the states in the computational
//! basis, or easily transform them into each other. Examples include
//! [ProductState::invert_digit] and [SuperPosition::from] respectively.

mod product_states;
mod qubit;
mod super_position_iter;
mod super_positions;
mod super_positions_unchecked;

pub use product_states::ProductState;
pub use qubit::Qubit;
pub use super_position_iter::SuperPositionIterator;
pub use super_positions::SuperPosition;
