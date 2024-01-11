/*
* Copyright (c) 2023 Andrew Rowan Barlow. Licensed under the EUPL-1.2
* or later. You may obtain a copy of the licence at
* https://joinup.ec.europa.eu/collection/eupl/eupl-text-eupl-12. A copy
* of the EUPL-1.2 licence in English is given in LICENCE.txt which is
* found in the root directory of this repository.
*
* Author: Andrew Rowan Barlow <a.barlow.dev@gmail.com>
*/

use super::gate::GateCategory;
use super::{GateInfo, ZERO_MARGIN};
use crate::states::{ProductState, SuperPosition};
use crate::{Circuit, Complex, Gate};
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
        println!(
            "Applying {:?} on wire {} # {}/{} ",
            gate,
            gate_pos,
            qubit_counter + 1,
            number_gates
        );

        if *qubit_counter + 1 == *number_gates {
            println!("Finished circuit simulation.")
        }
    }

    // The main algorithm and impetus for this project.
    //
    // This takes linear mappings defined on how they act on the basis of their product space, to
    // then apply on an arbitrary register. This algorithm is used instead of matrices, or sparse
    // matrices, in an effort to reduce memory. Cannot guarantee if this method is the fastest.
    pub(super) fn apply_gate(gate: GateInfo, register: &mut SuperPosition) {
        // the sum of states that are required to be added to the register
        let mut mapped_states: HashMap<ProductState, Complex<f64>> = Default::default();

        for (prod_state, amp) in register.into_iter() {
            //Looping through super position of register

            // Obtain superposition from applying gate from a specified wire onto the product state, and add control nodes if necersary
            let mut acting_positions: Vec<usize> = Vec::<usize>::with_capacity(3); // change to array for increased speed?

            let wrapped_super_pos: Option<SuperPosition> = match gate.cat_gate {
                GateCategory::Identity => None,
                GateCategory::Single(func) => Some(func(prod_state.get_qubits()[gate.position])),
                GateCategory::SingleArg(arg, func) => {
                    Some(func(prod_state.get_qubits()[gate.position], arg))
                }
                GateCategory::Double(c, func) => {
                    acting_positions.push(c);
                    let qubits = prod_state.get_qubits();
                    Some(func(qubits[c], qubits[gate.position]))
                }
                GateCategory::DoubleArg(arg, c, func) => {
                    acting_positions.push(c);
                    let qubits = prod_state.get_qubits();
                    Some(func(qubits[c], qubits[gate.position], arg))
                }
                GateCategory::DoubleArgInt(arg_int, c, func) => {
                    acting_positions.push(c);
                    let qubits = prod_state.get_qubits();
                    Some(func(qubits[c], qubits[gate.position], arg_int))
                }
                GateCategory::Triple(c1, c2, func) => {
                    acting_positions.push(c2);
                    acting_positions.push(c1);
                    let qubits = prod_state.get_qubits();
                    Some(func(qubits[c1], qubits[c2], qubits[gate.position]))
                }
                GateCategory::Custom(func, controls) => {
                    acting_positions.extend(controls.iter().rev());
                    Self::custom_gate_on_wires(func, controls, gate.position, &prod_state)
                }
            };

            if let Some(super_pos) = wrapped_super_pos {
                if !acting_positions.is_empty() {
                    acting_positions.reverse()
                };
                acting_positions.push(gate.position);
                Self::insert_gate_image_into_product_state(
                    super_pos,
                    acting_positions,
                    prod_state,
                    amp,
                    &mut mapped_states,
                );
            }
        }
        // All states in register considers, and can create new super position
        register.set_amplitudes_from_states_unchecked(mapped_states);
    }

    fn custom_gate_on_wires(
        operator: fn(ProductState) -> Option<SuperPosition>,
        controls: &[usize],
        position: usize,
        prod_state: &ProductState,
    ) -> Option<SuperPosition> {
        if !controls.is_empty() {
            let mut concat_prodstate: ProductState = prod_state.get_unchecked(controls[0]).into();

            for c in &controls[1..] {
                //converts product to larger product
                concat_prodstate = concat_prodstate.kronecker_prod(prod_state.get_unchecked(*c));
            }
            concat_prodstate = concat_prodstate.kronecker_prod(prod_state.get_unchecked(position));

            operator(concat_prodstate)
        } else {
            operator(ProductState::from(prod_state.qubits[position]))
        }
    }

    fn insert_gate_image_into_product_state(
        gate_image: SuperPosition,
        gate_positions: Vec<usize>,
        prod_state: ProductState,
        amp: Complex<f64>,
        mapped_states: &mut HashMap<ProductState, Complex<f64>>,
    ) {
        for (state, state_amp) in gate_image.into_iter() {
            if state_amp.re.abs() < ZERO_MARGIN && state_amp.im.abs() < ZERO_MARGIN {
                continue;
            }
            // Insert these image states back into a product space
            let mut swapped_state: ProductState = prod_state.clone();
            swapped_state.insert_qubits(state.qubits.as_slice(), gate_positions.as_slice());

            if let Some(existing_amp) = mapped_states.get(&swapped_state) {
                mapped_states.insert(swapped_state, existing_amp.add(state_amp.mul(amp)));
            } else {
                mapped_states.insert(swapped_state, state_amp.mul(amp));
            }
        }
    }
}
