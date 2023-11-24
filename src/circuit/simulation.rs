/*
* Copyright (c) 2023 Andrew Rowan Barlow. Licensed under the EUPL-1.2
* or later. You may obtain a copy of the licence at
* https://joinup.ec.europa.eu/collection/eupl/eupl-text-eupl-12. A copy
* of the EUPL-1.2 licence in English is given in LICENCE.txt which is
* found in the root directory of this repository.
*
* Author: Andrew Rowan Barlow <a.barlow.dev@gmail.com>
*/

use super::{standard_gate_ops, GateInfo, GateSize, ZERO_MARGIN};
use crate::states::{ProductState, Qubit, SuperPosition};
use crate::{Circuit, Complex, Gate};
use core::panic;
use std::collections::HashMap;
use std::ops::{Add, Mul};

impl<'a> Circuit<'a> {
    // If the user toggles the log on, then prints the simulation of each circuit.
    pub(super) fn print_circuit_log(
        gate: &Gate,
        gate_pos: &usize,
        qubit_counter: &usize,
        number_gates: &usize,
    ) {
        if gate != &Gate::Id {
            println!(
                "Applying {:?} on wire {} # {}/{} ",
                gate,
                gate_pos,
                qubit_counter + 1,
                number_gates
            );
        }

        if *qubit_counter + 1 == *number_gates {
            println!("Finished circuit simulation.")
        }
    }

    // Helps in constructing a bundle. This ultimately makes the match statements more concise.
    // Maybe best to see if this can be hardcoded in before hand; that is the bundles are added to
    // the circuit instead?
    pub(crate) fn classify_gate_size(gate: &Gate) -> GateSize {
        match gate {
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
            | Gate::MY90 => GateSize::Single,
            Gate::CNot(_)
            | Gate::Swap(_)
            | Gate::CZ(_)
            | Gate::CY(_)
            | Gate::CR(_, _)
            | Gate::CRk(_, _) => GateSize::Double,
            Gate::Toffoli(_, _) => GateSize::Triple,
            Gate::Custom(_, _, _) => GateSize::Custom,
        }
    }

    // The main algorithm and impetus for this project.
    //
    // This takes linear mappings defined on how they act on the basis of their product space, to
    // then apply on an arbitrary register. This algorithm is used instead of matrices, or sparse
    // matrices, in an effort to reduce memory. Cannot guarantee if this method is the fastest.
    pub(super) fn apply_gate(gate: &GateInfo, register: &SuperPosition) -> SuperPosition {
        // the sum of states that are required to be added to the register
        let mut mapped_states: HashMap<ProductState, Complex<f64>> = Default::default();

        for (prod_state, amp) in (&register).into_iter() {
            //Looping through super position of register

            // Obtain superposition from applying gate from a specified wire onto the product state, and add control nodes if necersary
            let mut acting_positions: Vec<usize> = vec![gate.position]; // change to array for increased speed?

            let super_pos: SuperPosition = match gate.size {
                GateSize::Single => Self::single_gate_on_wire(gate, &prod_state),
                GateSize::Double => {
                    Self::double_gate_on_wires(gate, &prod_state, &mut acting_positions)
                }
                GateSize::Triple => {
                    Self::triple_gate_on_wires(gate, &prod_state, &mut acting_positions)
                }
                GateSize::Custom => {
                    Self::custom_gate_on_wires(gate, &prod_state, &mut acting_positions)
                }
            };

            acting_positions.reverse(); // to fit the gate definitions to our convention
            Self::insert_gate_image_into_product_state(
                super_pos,
                acting_positions.as_slice(),
                &prod_state,
                amp,
                &mut mapped_states,
            );
        }
        // All states in register considers, and can create new super position
        // GET RID OF RETURNING SUPERPOSITION, INSTEAD JUST PASS REGISTER BY REFERENCE, &mut register
        register.set_amplitudes_from_states_unchecked(&mapped_states)
    }

    // The following functions compartmentalise the algorithms for applying a gate to the
    // register.
    fn single_gate_on_wire(single_gate: &GateInfo, prod_state: &ProductState) -> SuperPosition {
        if let Gate::Rx(angle) = single_gate.name {
            standard_gate_ops::rx(prod_state.qubits[single_gate.position], angle)
        } else if let Gate::Ry(angle) = single_gate.name {
            standard_gate_ops::ry(prod_state.qubits[single_gate.position], angle)
        } else if let Gate::Rz(angle) = single_gate.name {
            standard_gate_ops::rz(prod_state.qubits[single_gate.position], angle)
        } else if let Gate::Phase(angle) = single_gate.name {
            standard_gate_ops::global_phase(prod_state.qubits[single_gate.position], angle)
        } else {
            let operator: fn(Qubit) -> SuperPosition = match single_gate.name {
                Gate::Id => standard_gate_ops::identity,
                Gate::H => standard_gate_ops::hadamard,
                Gate::S => standard_gate_ops::phase,
                Gate::Sdag => standard_gate_ops::phasedag,
                Gate::T => standard_gate_ops::tgate,
                Gate::Tdag => standard_gate_ops::tgatedag,
                Gate::X => standard_gate_ops::pauli_x,
                Gate::Y => standard_gate_ops::pauli_y,
                Gate::Z => standard_gate_ops::pauli_z,
                Gate::X90 => standard_gate_ops::x90,
                Gate::Y90 => standard_gate_ops::y90,
                Gate::MX90 => standard_gate_ops::mx90,
                Gate::MY90 => standard_gate_ops::my90,
                _ => panic!("Non single gate was passed to single gate operation function."),
            };
            operator(prod_state.qubits[single_gate.position])
        }
    }

    fn double_gate_on_wires(
        double_gate: &GateInfo,
        prod_state: &ProductState,
        positions: &mut Vec<usize>,
    ) -> SuperPosition {
        // operator: fn(ProductState) -> SuperPosition
        if let Gate::CR(angle, control) = double_gate.name {
            positions.push(control);
            standard_gate_ops::cr(
                prod_state
                    .get(control)
                    .kronecker_prod(prod_state.get(double_gate.position)),
                angle,
            )
        } else if let Gate::CRk(k, control) = double_gate.name {
            positions.push(control);
            standard_gate_ops::crk(
                prod_state
                    .get(control)
                    .kronecker_prod(prod_state.get(double_gate.position)),
                k,
            )
        } else {
            let control_node: usize;
            let operator = match double_gate.name {
                Gate::CNot(control) => {
                    control_node = control;
                    standard_gate_ops::cnot
                }
                Gate::CZ(control) => {
                    control_node = control;
                    standard_gate_ops::cz
                }
                Gate::CY(control) => {
                    control_node = control;
                    standard_gate_ops::cy
                }
                Gate::Swap(control) => {
                    control_node = control;
                    standard_gate_ops::swap
                }
                _ => panic!("Non double gate was passed to double gate operation function."),
            };

            positions.push(control_node);
            operator(
                prod_state
                    .get(control_node)
                    .kronecker_prod(prod_state.get(double_gate.position)),
            )
        }
    }

    fn triple_gate_on_wires(
        triple_gate: &GateInfo,
        prod_state: &ProductState,
        positions: &mut Vec<usize>,
    ) -> SuperPosition {
        // operator: fn(ProductState) -> SuperPosition
        let (operator, control_node_one, control_node_two) = match triple_gate.name {
            Gate::Toffoli(control1, control2) => (standard_gate_ops::toffoli, control1, control2),
            _ => panic!("Non triple gate was passed to triple gate operation function"),
        };

        positions.push(control_node_two);
        positions.push(control_node_one);
        operator(
            prod_state
                .get(control_node_one)
                .kronecker_prod(prod_state.get(control_node_two))
                .kronecker_prod(prod_state.get(triple_gate.position)), // make qubit joiner for product states
        )
    }

    fn custom_gate_on_wires(
        custom_gate: &GateInfo,
        prod_state: &ProductState,
        positions: &mut Vec<usize>,
    ) -> SuperPosition {
        let (operator, controls) = match custom_gate.name {
            Gate::Custom(func, control_map, _) => (func, control_map),
            _ => panic!("Non custom gate was passed to custom gate operation function."),
        };

        let result_super: SuperPosition = if !controls.is_empty() {
            let mut concat_prodstate: ProductState = prod_state.get(controls[0]).into_state();

            for c in &controls[1..] {
                //converts product to larger product
                concat_prodstate = concat_prodstate.kronecker_prod(prod_state.get(*c));
            }
            concat_prodstate =
                concat_prodstate.kronecker_prod(prod_state.get(custom_gate.position));

            operator(concat_prodstate)
        } else {
            operator(prod_state.qubits[custom_gate.position].into_state())
        };

        positions.extend(controls.iter().rev());

        result_super
    }

    fn insert_gate_image_into_product_state(
        gate_image: SuperPosition,
        gate_positions: &[usize],
        prod_state: &ProductState,
        amp: Complex<f64>,
        mapped_states: &mut HashMap<ProductState, Complex<f64>>,
    ) {
        for (state, state_amp) in gate_image.into_iter() {
            if state_amp.abs_square() < ZERO_MARGIN {
                continue;
            }
            // Insert these image states back into a product space
            let swapped_state: ProductState =
                prod_state.insert_qubits(state.qubits.as_slice(), gate_positions);
            if mapped_states.contains_key(&swapped_state) {
                let existing_amp: Complex<f64> = *mapped_states.get(&swapped_state).unwrap();
                mapped_states.insert(swapped_state, existing_amp.add(state_amp.mul(amp)));
            } else {
                mapped_states.insert(swapped_state, state_amp.mul(amp));
            }
        }
    }
}
