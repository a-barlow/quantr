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

/// Gates that can be added to a [crate::Circuit] struct.
///
/// Matrix representations of these gates can be found at
/// <https://www.quantum-inspire.com/kbase/cqasm-qubit-gate-operations/>.
#[derive(Clone, PartialEq, Debug)]
pub enum Gate<'a> {
    /// Identity.
    Id,
    /// Hadamard.
    H,
    /// Pauli-X.
    X,
    /// Pauli-Y.
    Y,
    /// Pauli-Z.
    Z,
    /// Phase, rotation of +π/2 around the z-axis.
    S,
    /// Phase dagger, rotation of -π/2 around the z-axis.
    Sdag,
    /// T.
    T,
    /// T dagger.
    Tdag,
    /// Rotation around x-axis, with angle.
    Rx(f64),
    /// Rotation around y-axis, with angle.
    Ry(f64),
    /// Rotation around z-axis, with angle.
    Rz(f64),
    /// Rotation of +π/2 around x-axis.
    X90,
    /// Rotation of +π/2 around y-axis.
    Y90,
    /// Rotation of -π/2 around x-axis.
    MX90,
    /// Rotation of -π/2 around y-axis.
    MY90,
    /// Global phase, `exp(i*theta/2) * Identity`, with angle.
    Phase(f64),
    /// Controlled phase shift, with rotation and position of control node respectively.
    CR(f64, usize),
    /// Controlled phase shift for Quantum Fourier Transforms, with rotation and position
    /// of control node respectively.
    CRk(i32, usize),
    /// Controlled Pauli-Z, with position of control node.
    CZ(usize),
    /// Controlled Pauli-Y, with position of control node.
    CY(usize),
    /// Controlled Not, with position of control node.
    CNot(usize),
    /// Swap, with position of control node.
    Swap(usize),
    /// Toffoli, with position of control nodes.
    Toffoli(usize, usize),
    /// Defines a custom gate.
    ///
    /// *Note*, that the custom function isn't checked for unitarity.
    ///
    /// The arguments define the mapping of the gate; the position of the control node and a name that
    /// will be displayed in the printed diagram respectively. The name of the custom gate
    /// should be in ASCII for it to render properly when printing the circuit diagram.
    ///
    /// # Example
    /// ```
    /// use quantr::{Circuit, Gate};
    /// use quantr::states::{SuperPosition, ProductState, Qubit};
    /// use quantr::{Complex, complex_re_array};
    ///
    /// // Defines a C-Not gate
    /// fn example_cnot(prod: ProductState) -> Option<SuperPosition> {
    ///    let input_register: [Qubit; 2] = [prod.get_qubits()[0], prod.get_qubits()[0]];
    ///    Some(SuperPosition::new_with_amplitudes(match input_register {
    ///        [Qubit::Zero, Qubit::Zero] => return None,
    ///        [Qubit::Zero, Qubit::One]  => return None,
    ///        [Qubit::One, Qubit::Zero]  => &complex_re_array!(0f64, 0f64, 0f64, 1f64),
    ///        [Qubit::One, Qubit::One]   => &complex_re_array!(0f64, 0f64, 1f64, 0f64),
    ///    }).unwrap())
    /// }
    ///
    /// let mut quantum_circuit = Circuit::new(3).unwrap();
    /// quantum_circuit.add_gate(Gate::Custom(example_cnot, &[2], String::from("X")), 1).unwrap();
    ///
    /// // This is equivalent to
    /// quantum_circuit.add_gate(Gate::CNot(2), 1).unwrap();
    ///
    /// ```
    Custom(
        fn(ProductState) -> Option<SuperPosition>,
        &'a [usize],
        String,
    ),
}

impl<'a> Gate<'a> {
    // Retrieves the list of nodes within a gate.
    pub(super) fn get_nodes(&self) -> Option<Vec<usize>> {
        match self {
            Gate::Id
            | Gate::H
            | Gate::S
            | Gate::Sdag
            | Gate::T
            | Gate::Tdag
            | Gate::X
            | Gate::Y
            | Gate::Z
            | Gate::Rx(_)
            | Gate::Ry(_)
            | Gate::Rz(_)
            | Gate::Phase(_)
            | Gate::X90
            | Gate::Y90
            | Gate::MX90
            | Gate::MY90 => None,
            Gate::CNot(c)
            | Gate::Swap(c)
            | Gate::CZ(c)
            | Gate::CY(c)
            | Gate::CR(_, c)
            | Gate::CRk(_, c) => Some(vec![*c]),
            Gate::Toffoli(c1, c2) => Some(vec![*c1, *c2]),
            Gate::Custom(_, nodes, _) => Some(nodes.to_vec()),
        }
    }
}

/// For identifying which gates are single, double etc.
#[derive(Debug, Clone)]
pub(crate) enum GateSize {
    Single,
    Double,
    Triple,
    Custom,
}

/// Bundles the gate and position together.
#[derive(Debug)]
pub(crate) struct GateInfo<'a> {
    pub name: Gate<'a>,
    pub position: usize,
    pub size: GateSize,
}
