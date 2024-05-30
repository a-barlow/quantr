/*
* Copyright (c) 2024 Andrew Rowan Barlow. Licensed under the EUPL-1.2
* or later. You may obtain a copy of the licence at
* https://joinup.ec.europa.eu/collection/eupl/eupl-text-eupl-12. A copy
* of the EUPL-1.2 licence in English is given in LICENCE.txt which is
* found in the root directory of this repository.
*
* Author: Andrew Rowan Barlow <a.barlow.dev@gmail.com>
*/

use crate::circuit::standard_gate_ops;
use crate::states::{ProductState, Qubit, SuperPosition};

/// Gates that can be added to a [crate::Circuit] struct.
///
/// Matrix representations of these gates can be found at
/// <https://www.quantum-inspire.com/kbase/cqasm-qubit-gate-operations/>.
#[derive(Clone, PartialEq, Debug)]
pub enum Gate {
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
    ///    let input_register: [Qubit; 2] = [prod.get_qubits()[0], prod.get_qubits()[1]];
    ///    Some(SuperPosition::new_with_amplitudes(match input_register {
    ///        [Qubit::Zero, Qubit::Zero] => return None,
    ///        [Qubit::Zero, Qubit::One]  => return None,
    ///        [Qubit::One, Qubit::Zero]  => &complex_re_array!(0f64, 0f64, 0f64, 1f64),
    ///        [Qubit::One, Qubit::One]   => &complex_re_array!(0f64, 0f64, 1f64, 0f64),
    ///    }).unwrap())
    /// }
    ///
    /// let mut quantum_circuit = Circuit::new(3).unwrap();
    /// quantum_circuit.add_gate(Gate::Custom(example_cnot, vec![2], String::from("X")), 1).unwrap();
    ///
    /// // This is equivalent to
    /// quantum_circuit.add_gate(Gate::CNot(2), 1).unwrap();
    ///
    /// ```
    Custom(
        fn(ProductState) -> Option<SuperPosition>,
        Vec<usize>,
        String,
    ),
}

impl Gate {
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

    pub(crate) fn linker(&self) -> GateCategory {
        match self {
            Gate::Id => GateCategory::Identity,
            Gate::H => GateCategory::Single(standard_gate_ops::hadamard),
            Gate::S => GateCategory::Single(standard_gate_ops::phase),
            Gate::Sdag => GateCategory::Single(standard_gate_ops::phasedag),
            Gate::T => GateCategory::Single(standard_gate_ops::tgate),
            Gate::Tdag => GateCategory::Single(standard_gate_ops::tgatedag),
            Gate::X => GateCategory::Single(standard_gate_ops::pauli_x),
            Gate::Y => GateCategory::Single(standard_gate_ops::pauli_y),
            Gate::Z => GateCategory::Single(standard_gate_ops::pauli_z),
            Gate::X90 => GateCategory::Single(standard_gate_ops::x90),
            Gate::Y90 => GateCategory::Single(standard_gate_ops::y90),
            Gate::MX90 => GateCategory::Single(standard_gate_ops::mx90),
            Gate::MY90 => GateCategory::Single(standard_gate_ops::my90),
            Gate::Rx(arg) => GateCategory::SingleArg(*arg, standard_gate_ops::rx),
            Gate::Ry(arg) => GateCategory::SingleArg(*arg, standard_gate_ops::ry),
            Gate::Rz(arg) => GateCategory::SingleArg(*arg, standard_gate_ops::rz),
            Gate::Phase(arg) => GateCategory::SingleArg(*arg, standard_gate_ops::global_phase),
            Gate::CNot(c) => GateCategory::Double(*c, standard_gate_ops::cnot),
            Gate::Swap(c) => GateCategory::Double(*c, standard_gate_ops::swap),
            Gate::CZ(c) => GateCategory::Double(*c, standard_gate_ops::cz),
            Gate::CY(c) => GateCategory::Double(*c, standard_gate_ops::cy),
            Gate::CR(arg, c) => GateCategory::DoubleArg(*arg, *c, standard_gate_ops::cr),
            Gate::CRk(arg, c) => GateCategory::DoubleArgInt(*arg, *c, standard_gate_ops::crk),
            Gate::Toffoli(c1, c2) => GateCategory::Triple(*c1, *c2, standard_gate_ops::toffoli),
            Gate::Custom(func, controls, _) => GateCategory::Custom(*func, controls),
        }
    }

    // Helps in constructing a bundle. This ultimately makes the match statements more concise.
    // Maybe best to see if this can be hardcoded in before hand; that is the bundles are added to
    // the circuit instead?
    pub(crate) fn is_single_gate(&self) -> bool {
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
            | Gate::MY90 => true,
            Gate::CNot(_)
            | Gate::Swap(_)
            | Gate::CZ(_)
            | Gate::CY(_)
            | Gate::CR(_, _)
            | Gate::CRk(_, _)
            | Gate::Toffoli(_, _)
            | Gate::Custom(_, _, _) => false,
        }
    }

    pub(crate) fn is_custom_gate(&self) -> bool {
        match self {
            Gate::Custom(_, _, _) => true,
            _ => false,
        }
    }

    pub(crate) fn get_name(&self) -> String {
        match self {
            Gate::Id => "".to_string(),
            Gate::X => "X".to_string(),
            Gate::H => "H".to_string(),
            Gate::S => "S".to_string(),
            Gate::Sdag => "S*".to_string(),
            Gate::T => "T".to_string(),
            Gate::Tdag => "T*".to_string(),
            Gate::Y => "Y".to_string(),
            Gate::Z => "Z".to_string(),
            Gate::Rx(_) => "Rx".to_string(),
            Gate::Ry(_) => "Ry".to_string(),
            Gate::Rz(_) => "Rz".to_string(),
            Gate::Phase(_) => "P".to_string(),
            Gate::X90 => "X90".to_string(),
            Gate::Y90 => "Y90".to_string(),
            Gate::MX90 => "X90*".to_string(),
            Gate::MY90 => "Y90*".to_string(),
            Gate::CR(_, _) => "CR".to_string(),
            Gate::CRk(_, _) => "CRk".to_string(),
            Gate::Swap(_) => "Sw".to_string(),
            Gate::CZ(_) => "Z".to_string(),
            Gate::CY(_) => "Y".to_string(),
            Gate::CNot(_) => "X".to_string(),
            Gate::Toffoli(_, _) => "X".to_string(),
            Gate::Custom(_, _, name) => name.to_string(),
        }
    }
}

// Contain second variant that references the function in standard_gate_ops.rs
#[derive(PartialEq, Debug)]
pub(crate) enum GateCategory<'a> {
    Identity,
    Single(fn(Qubit) -> SuperPosition),
    SingleArg(f64, fn(Qubit, f64) -> SuperPosition),
    Double(usize, fn(Qubit, Qubit) -> SuperPosition),
    DoubleArg(f64, usize, fn(Qubit, Qubit, f64) -> SuperPosition),
    DoubleArgInt(i32, usize, fn(Qubit, Qubit, i32) -> SuperPosition),
    Triple(usize, usize, fn(Qubit, Qubit, Qubit) -> SuperPosition),
    Custom(fn(ProductState) -> Option<SuperPosition>, &'a [usize]),
}

/// Bundles the gate and position together.
#[derive(Debug)]
pub(crate) struct GateInfo<'a> {
    pub cat_gate: GateCategory<'a>,
    pub position: usize,
}
