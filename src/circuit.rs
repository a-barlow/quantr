/*
* Copyright (c) 2023 Andrew Rowan Barlow. Licensed under the EUPL-1.2
* or later. You may obtain a copy of the licence at
* https://joinup.ec.europa.eu/collection/eupl/eupl-text-eupl-12. A copy
* of the EUPL-1.2 licence in English is given in LICENCE.txt which is
* found in the root directory of this repository.
*
* Author: Andrew Rowan Barlow <a.barlow.dev@gmail.com>
*/

//! Construct, simulate and measure quantum circuits.
//!
//! Initialise a new quantum circuit by using [Circuit::new] where the argument defines the number
//! of qubits. Afterwards, various methods can be called to append gates onto the circuit in columns.
//! For instance, [Circuit::add_gate] will add a single gate, whilst
//! [Circuit::add_gates_with_positions] and [Circuit::add_repeating_gate] will add multiple.
//!
//! The circuit can then be simulated with [Circuit::simulate]. The progress of the simulation can
//! be printed to the terminal by calling [Circuit::toggle_simulation_progress] before simulating
//! the circuit.
//!
//! A bin count of states that are observed over a series of measurements can be performed with
//! [Circuit::repeat_measurement], where a new register is attached before each measurment. Or, the
//! explicit superpositon can be retreived using [Circuit::get_superposition].

use crate::complex::Complex;
use core::panic;
use rand::Rng;
use std::collections::HashMap;
use std::ops::{Add, Mul};

pub mod error;
pub mod printer;
mod standard_gate_ops;
pub mod states;
use self::error::QuantrError;
use crate::circuit::states::{ProductState, Qubit, SuperPosition};

/// Distinguishes observable and non-observable quantities.
///
/// For example, this will distinguish the retreival of a superposition (that cannot be measured
/// directly), and the state resulting from the collapse of a superposition upon measurement. See
/// [Circuit::get_superposition] and [Circuit::repeat_measurement] for examples.
pub enum Measurement<T> {
    Observable(T),
    NonObservable(T),
}

/// Gates that can be added to a [Circuit] struct.
///
/// Currently, this enum has the `#[non_exhaustive]` as it's
/// yet undecided what will be included as a standard gate. This will
/// lessen the impact of breaking changes in the future.
#[derive(Clone, PartialEq, Debug)]
#[non_exhaustive]
pub enum StandardGate<'a> {
    /// Identity.
    Id,
    /// Hadamard.
    H,
    /// Pauli-X
    X,
    /// Pauli-Y.
    Y,
    /// Pauli-Z.
    Z,
    /// Controlled Pauli-Z gate, with position of control node.
    CZ(usize),
    /// Controlled Pauli-Y gate, with position of control node.
    CY(usize),
    /// CNot, with position of control node.
    CNot(usize),
    /// Swap, with position of control node.
    Swap(usize),
    /// Toffoli, with position of control nodes.
    Toffoli(usize, usize),
    /// Defines a custom gate.
    ///
    /// The arguments define the mapping of the
    /// gate; the position of the qubit states that the gate acts on;
    /// and a name that will be displayed in the printed diagram
    /// respectively.
    ///
    /// # Example
    /// ```
    /// use quantr::circuit::{Circuit, StandardGate};
    /// use quantr::circuit::states::{SuperPosition, ProductState, Qubit};
    /// use quantr::complex::Complex;
    /// use quantr::complex_Re_array;
    ///
    /// // Defines a C-Not gate
    /// fn example_cnot(prod: ProductState) -> SuperPosition {
    ///    let input_register: [Qubit; 2] = [prod.state[0], prod.state[1]];
    ///    SuperPosition::new(2).set_amplitudes(match input_register {
    ///        [Qubit::Zero, Qubit::Zero] => &complex_Re_array!(1f64, 0f64, 0f64, 0f64),
    ///        [Qubit::Zero, Qubit::One]  => &complex_Re_array!(0f64, 1f64, 0f64, 0f64),
    ///        [Qubit::One, Qubit::Zero]  => &complex_Re_array!(0f64, 0f64, 0f64, 1f64),
    ///        [Qubit::One, Qubit::One]   => &complex_Re_array!(0f64, 0f64, 1f64, 0f64),
    ///    }).unwrap()
    /// }
    ///
    /// let mut quantum_circuit = Circuit::new(3);
    /// quantum_circuit.add_gate(StandardGate::Custom(example_cnot, &[2], String::from("X")), 1).unwrap();
    ///
    /// // This is equivalent to
    /// quantum_circuit.add_gate(StandardGate::CNot(2), 1).unwrap();
    ///
    /// ```
    Custom(fn(ProductState) -> SuperPosition, &'a [usize], String),
}

/// For identifying which gates are single, double etc.
#[derive(Debug)]
enum GateSize {
    Single,
    Double,
    Triple,
    Custom,
}

/// Bundles the gate and position together.
#[derive(Debug)]
struct GateInfo<'a> {
    name: StandardGate<'a>,
    position: usize,
    size: GateSize,
}

/// A quantum circuit where gates can be appended and then simulated to measure resulting
/// superpositions.
pub struct Circuit<'a> {
    pub circuit_gates: Vec<StandardGate<'a>>,
    pub num_qubits: usize,
    output_state: Option<SuperPosition>,
    config_progress: bool,
}

impl<'a> Circuit<'a> {
    /// Initialises a new circuit.
    ///
    /// The target qubits used in defining custom functions must out live the slice of target
    /// qubits given to the custom function.
    pub fn new(num_qubits: usize) -> Circuit<'a> {
        let circuit_gates: Vec<StandardGate> = Vec::new();
        Circuit {
            circuit_gates,
            num_qubits,
            output_state: None,
            config_progress: false,
        }
    }

    /// Toggles if the circuit should print the progress of simulating each gate.
    ///
    /// It will only show the application of non-identity gates. The toggle is set to `false`
    /// for a new quantum circuit.
    pub fn toggle_simulation_progress(&mut self) {
        self.config_progress = !self.config_progress;
    }

    /// Add a column of gates.
    ///
    /// Expects the input vector to specify the gate that is added to *each* wire. That is, the
    /// length of the vector should equal the number of wires. To only add gates based on their
    /// positions, see [Circuit::add_gates_with_positions] and [Circuit::add_gate]. An
    /// [error::QuantrError] is returned if all wires are not accounted for.
    ///
    /// # Example   
    /// ```
    /// use quantr::circuit::{Circuit, StandardGate};
    ///
    /// let mut quantum_circuit: Circuit = Circuit::new(3);
    /// let gates_to_add: Vec<StandardGate> = vec![StandardGate::H, StandardGate::X, StandardGate::Y];
    ///
    /// quantum_circuit.add_gates(gates_to_add).unwrap();
    ///
    /// // Produces the circuit:
    /// // |0> -- H --
    /// // |0> -- X --
    /// // |0> -- Y --
    /// ```
    pub fn add_gates(&mut self, gates: Vec<StandardGate<'a>>) -> Result<(), QuantrError> {
        // Ensured we have a gate for every wire.
        if gates.len() != self.num_qubits {
            return Err(QuantrError {
                message: format!("The number of gates, {}, does not match the number of wires, {}. All wires must have gates added.", gates.len(), self.num_qubits)
            });
        }

        // Add functionality to safely add double gates

        self.circuit_gates.extend(gates);
        Ok(())
    }

    /// Place a single gate repeatedly onto multiple wires.
    ///
    /// The top of the wire is in the 0th position. For adding multiple gates that are different,
    /// please refer to [Circuit::add_gates] and [Circuit::add_gates_with_positions]. An
    /// [error::QuantrError] will be returned if the vector contains positions that are equal.
    ///
    /// # Example
    /// ```
    /// use quantr::circuit::{Circuit, StandardGate};
    ///
    /// let mut quantum_circuit: Circuit = Circuit::new(3);
    /// quantum_circuit.add_repeating_gate(StandardGate::H, vec![1, 2]).unwrap();
    ///
    /// // Produces the circuit:
    /// // |0> -------
    /// // |0> -- H --
    /// // |0> -- H --
    /// ```
    pub fn add_repeating_gate(
        &mut self,
        gate: StandardGate<'a>,
        positions: Vec<usize>,
    ) -> Result<(), QuantrError> {
        // Incase the user has attempted to place the gate twice on the same wire.
        if let Some(wire_num) = Self::has_repeating_digits(positions.as_slice()) {
            return Err(QuantrError {
                message: format!(
                    "Attempted to add more than one gate onto wire {}.",
                    wire_num
                ),
            });
        }

        // Generates a list of identity gates, that are subsequently replaced by non-trivial gates
        // specified by the user.
        let list_of_identities: Vec<StandardGate> = vec![StandardGate::Id; self.num_qubits];
        let gates: Vec<StandardGate> = list_of_identities
            .iter()
            .enumerate()
            .map(|(pos, _)| {
                if positions.contains(&pos) {
                    gate.clone()
                } else {
                    StandardGate::Id
                }
            })
            .collect();
        self.add_gates(gates)
    }

    // Used to check if gates have been added to the same position more than once. Has O( n*log(n) )
    fn has_repeating_digits(array_to_check: &[usize]) -> Option<usize> {
        let length = array_to_check.len();
        for (i, element) in array_to_check.iter().enumerate() {
            for j in i + 1..length {
                if *element == array_to_check[j] {
                    return Some(i);
                }
            }
        }
        None
    }

    /// Add a column of gates based on their position on the wire.
    ///
    /// A HashMap is used to place gates onto their desired position; where the key is the position
    /// and the value is the [StandardGate]. This is similar to [Circuit::add_gate], however not
    /// all wires have to be accounted for. An [error::QuantrError] will be returned if there is a
    /// key that is greater than the size of the circuit.
    ///
    /// # Example
    /// ```
    /// use quantr::circuit::{Circuit, StandardGate};
    /// use std::collections::HashMap;
    ///
    /// let mut quantum_circuit: Circuit = Circuit::new(3);
    /// // Adds gates on wires 0 and 2, implicitly leaving wire 1 bare.
    /// quantum_circuit.add_gates_with_positions(
    ///     HashMap::from(
    ///         [(0, StandardGate::X), (2, StandardGate::H)]
    ///     )
    /// ).unwrap();
    ///
    /// // Produces the circuit:
    /// // |0> -- X --
    /// // |0> -------
    /// // |0> -- H --
    /// ```
    pub fn add_gates_with_positions(
        &mut self,
        gates_with_positions: HashMap<usize, StandardGate<'a>>,
    ) -> Result<(), QuantrError> {
        // If any keys are greater, return an error.
        if let Some(out_of_bounds_key) =
            gates_with_positions.keys().find(|k| *k >= &self.num_qubits)
        {
            return Err(QuantrError {
                message: format!(
                    "The position, {}, is a key in the hash map and is out of bounds 
                                 for the circuit with {} qubits.",
                    out_of_bounds_key, self.num_qubits
                ),
            });
        }

        // Add gates from `gates_with_positions` based on their positions. For the lines that don't have a gate, the identity is added.
        let mut gates_to_add: Vec<StandardGate> = Default::default();
        for row_num in 0..self.num_qubits {
            gates_to_add.push(
                gates_with_positions
                    .get(&row_num)
                    .unwrap_or(&StandardGate::Id)
                    .clone(),
            );
        }

        self.circuit_gates.extend(gates_to_add);
        Ok(())
    }

    /// Adds a single gate to the circuit.
    ///
    /// If wanting to add multiple gates, or a single gate repeatedly, across multiple wires, see
    /// [Circuit::add_repeating_gate] and [Circuit::add_gates_with_positions] respectively.
    ///
    /// # Example
    /// ```
    /// use quantr::circuit::{Circuit, StandardGate};
    ///
    /// let mut quantum_circuit: Circuit = Circuit::new(3);
    /// quantum_circuit.add_gate(StandardGate::X, 0).unwrap();
    ///
    /// // Produces the circuit:
    /// // |0> -- X --
    /// // |0> -------
    /// // |0> -------
    /// ```
    pub fn add_gate(&mut self, gate: StandardGate<'a>, position: usize) -> Result<(), QuantrError> {
        Self::add_gates_with_positions(self, HashMap::from([(position, gate)]))
    }

    /// Returns the resulting superposition after the circuit has been simulated.
    ///
    /// This is a non-physical observable, as the superposition would reduce to a single state upon measurement.
    /// Returnsan [error::QuantrError] if the circuit hasn't been simulated.
    ///
    /// # Example
    /// ```
    /// use quantr::circuit::{states::SuperPosition, Circuit, Measurement::NonObservable, StandardGate};
    ///
    /// let mut circuit = Circuit::new(3);
    ///
    /// circuit.add_gate(StandardGate::H, 2).unwrap();
    /// circuit.add_gate(StandardGate::Y, 2).unwrap();
    /// circuit.simulate();
    ///
    /// println!("State | Amplitude of State");
    /// if let NonObservable(super_pos) = circuit.get_superposition().unwrap() {
    ///     for (state, amplitude) in super_pos.into_iter() {
    ///         println!("|{}>   : {}", state.as_string(), amplitude);
    ///     }
    /// }
    ///
    /// // State | Amplitude of State    
    /// // |000> : 0 - 0.71...i     
    /// // |001> : 0 + 0.71...i
    /// ```
    pub fn get_superposition(&self) -> Result<Measurement<&SuperPosition>, QuantrError> {
        match &self.output_state {
            Some(super_position) => Ok(Measurement::NonObservable(super_position)),
            None => {
                Err(QuantrError {
                    message: format!("The circuit has not been simulated. Call Circuit::simulate before calling this method, Circuit::get_superposition.")
                })
            }
        }
    }

    /// Returns a HashMap that holds the number of times the corresponding state was observed over
    /// `n` measurments of the superposition.
    ///
    /// Peform repeated measurements where a register is attached to the circuit, the reuslting
    /// superposition measured, and then the reduced state recorded. If the HashMap does not
    /// include a product state, then it was not observed over the `n` measurements. Returns an
    /// [error::QuantrError] if the circuit hasn't been simulated.
    ///
    /// # Example
    /// ```
    /// use quantr::circuit::{states::SuperPosition, Circuit, Measurement::Observable, StandardGate};
    ///
    /// let mut circuit = Circuit::new(3);
    ///
    /// circuit.add_gate(StandardGate::H, 2).unwrap();
    /// circuit.simulate();
    ///
    /// // Measures 500 superpositions.
    /// println!("State | Number of Times Observed");
    /// if let Observable(bin_count) = circuit.repeat_measurement(500).unwrap() {
    ///     for (state, observed_count) in bin_count {
    ///         println!("|{}>   : {}", state.as_string(), observed_count);
    ///     }
    /// }
    ///
    /// // State | Number of Times Observed
    /// // |000> : 247
    /// // |001> : 253
    /// ```
    pub fn repeat_measurement(
        &self,
        number_iterations: usize,
    ) -> Result<Measurement<HashMap<ProductState, usize>>, QuantrError> {
        match &self.output_state {
            Some(super_position) => {
                // Peform bin count of states
                let mut probabilities: HashMap<ProductState, f64> = Default::default();
                for (key, value) in super_position.as_hash() {
                    probabilities.insert(key, value.abs_square());
                }

                let mut bin_count: HashMap<ProductState, usize> = Default::default();

                let mut rng = rand::thread_rng();
                for _ in 0..number_iterations {
                    let mut cummalitive: f64 = 0f64;
                    let dice_roll: f64 = rng.gen();
                    for (state_label, probability) in &probabilities {
                        cummalitive = cummalitive + probability;
                        if dice_roll < cummalitive {
                            match bin_count.get(state_label) {
                                Some(previous_count) => bin_count.insert(state_label.clone(), previous_count+1),
                                None => bin_count.insert(state_label.clone(), 1),
                            };
                            break;
                        }
                    }
                }
                Ok(Measurement::Observable(bin_count))
            },
            None => {
                Err(QuantrError {
                    message: format!("The circuit has not been simulated. Call Circuit::simulate before calling this method, Circuit::repeat_measurement.")
                })
            },
        }
    }

    /// Attaches the register, |0...0>, to the circuit resulting in a superposition that can be measured.
    ///
    /// See [Circuit::get_superposition] and [Circuit::repeat_measurement] for details on obtaining
    /// observables from the resulting superposition.
    pub fn simulate(&mut self) {
        // Form the initial state if the product space, that is |0...0>
        let mut register: SuperPosition = SuperPosition::new(self.num_qubits);

        let mut qubit_counter: usize = 0;
        let number_gates: usize = self.circuit_gates.len();

        if self.config_progress {
            println!("Starting circuit simulation...")
        }

        // Loop through each gate of circuit from starting at top row to bottom, then moving onto the next.
        for gate in &self.circuit_gates {
            let gate_pos: usize = qubit_counter % self.num_qubits;

            if self.config_progress {
                Self::print_circuit_log(gate, &gate_pos, &qubit_counter, &number_gates);
            }

            if *gate == StandardGate::Id {
                qubit_counter += 1;
                continue;
            }

            let gate_class: GateSize = Self::classify_gate_size(gate);
            let gate_to_apply: GateInfo = GateInfo {
                name: gate.clone(),
                position: gate_pos,
                size: gate_class,
            };
            register = Circuit::apply_gate(&gate_to_apply, &register);

            qubit_counter += 1;
        }

        self.output_state = Some(register);
    }

    // If the user toggles the log on, then prints the simulation of each circuit.
    fn print_circuit_log(
        gate: &StandardGate,
        gate_pos: &usize,
        qubit_counter: &usize,
        number_gates: &usize,
    ) {
        if gate != &StandardGate::Id {
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
    fn classify_gate_size(gate: &StandardGate) -> GateSize {
        match gate {
            StandardGate::Id
            | StandardGate::H
            | StandardGate::X
            | StandardGate::Y
            | StandardGate::Z => GateSize::Single,
            StandardGate::CNot(_)
            | StandardGate::Swap(_)
            | StandardGate::CZ(_)
            | StandardGate::CY(_) => GateSize::Double,
            StandardGate::Toffoli(_, _) => GateSize::Triple,
            StandardGate::Custom(_, _, _) => GateSize::Custom,
        }
    }

    // The main algorithm and impetus for this project.
    //
    // This takes linear mappings defined on how they act on the basis of their product space, to
    // then apply on an arbitary register. This algorithm is used instead of matrices, or sparse
    // matrices, in an effort to reduce memory. Cannot guarantee if this method is the fastest.
    fn apply_gate(gate: &GateInfo, register: &SuperPosition) -> SuperPosition {
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

            acting_positions.reverse(); // to fit the gate defintions to our convention
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
        register.set_amplitudes_from_states(&mapped_states).unwrap()
    }

    // The following functions compartementalise the algorithms for applying a gate to the
    // register.

    fn single_gate_on_wire(single_gate: &GateInfo, prod_state: &ProductState) -> SuperPosition {
        let operator: fn(Qubit) -> SuperPosition = match single_gate.name {
            StandardGate::Id => standard_gate_ops::identity,
            StandardGate::H => standard_gate_ops::hadamard,
            StandardGate::X => standard_gate_ops::pauli_x,
            StandardGate::Y => standard_gate_ops::pauli_y,
            StandardGate::Z => standard_gate_ops::pauli_z,
            _ => panic!("Non single gate was passed to single gate operation function."),
        };
        operator(prod_state.state[single_gate.position])
    }

    fn double_gate_on_wires(
        double_gate: &GateInfo,
        prod_state: &ProductState,
        positions: &mut Vec<usize>,
    ) -> SuperPosition {
        // operator: fn(ProductState) -> SuperPosition
        let control_node: usize;
        let operator = match double_gate.name {
            StandardGate::CNot(control) => {
                control_node = control;
                standard_gate_ops::cnot
            }
            StandardGate::CZ(control) => {
                control_node = control;
                standard_gate_ops::cz
            }
            StandardGate::CY(control) => {
                control_node = control;
                standard_gate_ops::cy
            }
            StandardGate::Swap(control) => {
                control_node = control;
                standard_gate_ops::swap
            }
            _ => panic!("Non double gate was passed to double gate operation function."),
        };

        positions.push(control_node);
        operator(
            prod_state
                .get(control_node)
                .join(prod_state.get(double_gate.position)),
        )
    }

    fn triple_gate_on_wires(
        triple_gate: &GateInfo,
        prod_state: &ProductState,
        positions: &mut Vec<usize>,
    ) -> SuperPosition {
        // operator: fn(ProductState) -> SuperPosition
        let (operator, control_node_one, control_node_two) = match triple_gate.name {
            StandardGate::Toffoli(control1, control2) => {
                (standard_gate_ops::toffoli, control1, control2)
            }
            _ => panic!("Non triple gate was passed to triple gate operation function"),
        };

        positions.push(control_node_two);
        positions.push(control_node_one);
        operator(
            prod_state
                .get(control_node_one)
                .join(prod_state.get(control_node_two))
                .join(prod_state.get(triple_gate.position)), // make qubit joiner for product states
        )
    }

    fn custom_gate_on_wires(
        custom_gate: &GateInfo,
        prod_state: &ProductState,
        positions: &mut Vec<usize>,
    ) -> SuperPosition {
        let (operator, controls) = match custom_gate.name {
            StandardGate::Custom(func, control_map, _) => (func, control_map),
            _ => panic!("Non custom gate was passed to custom gate operation function."),
        };

        let result_super: SuperPosition = if !controls.is_empty() {
            let mut concat_prodstate: ProductState = prod_state.get(controls[0]).as_state();

            for c in &controls[1..] {
                //converts product to larger product
                concat_prodstate = concat_prodstate.join(prod_state.get(*c));
            }
            concat_prodstate = concat_prodstate.join(prod_state.get(custom_gate.position));

            operator(concat_prodstate)
        } else {
            operator(prod_state.state[custom_gate.position].as_state())
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
            if state_amp.abs_square() < 0.01 {
                continue;
            }
            // Insert these image states back into a product space
            let swapped_state: ProductState =
                prod_state.insert_qubits(state.state.as_slice(), gate_positions);
            if mapped_states.contains_key(&swapped_state) {
                let existing_amp: Complex<f64> = *mapped_states.get(&swapped_state).unwrap();
                mapped_states.insert(swapped_state, existing_amp.add(state_amp.mul(amp)));
            } else {
                mapped_states.insert(swapped_state, state_amp.mul(amp));
            }
        }
    }
}

#[rustfmt::skip]
#[cfg(test)]
mod tests {
    use crate::{complex_Im, complex_Re, complex_Re_array, complex_zero};
    use std::f64::consts::FRAC_1_SQRT_2;
    use crate::circuit::Measurement::NonObservable;
    use super::*;
    const ERROR_MARGIN: f64 = 0.00000001f64; // For comparing floats due to floating point error.

    // Needed for testing
    fn compare_complex_lists_and_register(correct_list: &[Complex<f64>], register: &SuperPosition) {
        for (i, &comp_num) in register.amplitudes.iter().enumerate() { // Make sure that it turns up complex
            assert!(equal_within_error(comp_num.real, correct_list[i].real));
            assert!(equal_within_error(comp_num.imaginary, correct_list[i].imaginary));
        }
    }

    fn equal_within_error(num: f64, compare_num: f64) -> bool {
        if num < compare_num + ERROR_MARGIN && num > compare_num - ERROR_MARGIN {
            true
        } else {
            false
        }
    }

    fn compare_circuit(quantum_circuit: Circuit, correct_register: &[Complex<f64>]) {
        if let NonObservable(measured_register) = quantum_circuit.get_superposition().unwrap() {
            compare_complex_lists_and_register(correct_register, measured_register);
        }
    }

    fn example_cnot(prod: ProductState) -> SuperPosition {
        let input_register: [Qubit; 2] = [prod.state[0], prod.state[1]];
        SuperPosition::new(2).set_amplitudes(match input_register {
            [Qubit::Zero, Qubit::Zero] => &complex_Re_array!(1f64, 0f64, 0f64, 0f64),
            [Qubit::Zero, Qubit::One]  => &complex_Re_array!(0f64, 1f64, 0f64, 0f64),
            [Qubit::One, Qubit::Zero]  => &complex_Re_array!(0f64, 0f64, 0f64, 1f64),
            [Qubit::One, Qubit::One]   => &complex_Re_array!(0f64, 0f64, 1f64, 0f64),
        }).unwrap()
    }

    // All tests below were calculated by hand.

    #[test]
    fn custom_gates() {
        let mut quantum_circuit = Circuit::new(3);
        quantum_circuit.add_gate(StandardGate::H, 2).unwrap();
        quantum_circuit.add_gate(StandardGate::Custom(example_cnot, &[2], String::from("cNot")), 1).unwrap();
        quantum_circuit.simulate();
        
        let correct_register: [Complex<f64>; 8] = [
            complex_Re!(FRAC_1_SQRT_2), complex_zero!(),
            complex_zero!(), complex_Re!(FRAC_1_SQRT_2),
            complex_zero!(), complex_zero!(),
            complex_zero!(), complex_zero!()];

        compare_circuit(quantum_circuit, &correct_register);
    }

    #[test]
    fn toffoli_gates() {
        let mut quantum_circuit = Circuit::new(4);
        quantum_circuit.add_gate(StandardGate::X, 0).unwrap();
        quantum_circuit.add_gate(StandardGate::H, 3).unwrap();
        quantum_circuit.add_gate(StandardGate::Y, 3).unwrap();
        quantum_circuit.add_gate(StandardGate::Toffoli(3, 0), 1).unwrap();
        quantum_circuit.simulate();
        let correct_register = [
            complex_zero!(), complex_zero!(), complex_zero!(), complex_zero!(),
            complex_zero!(), complex_zero!(), complex_zero!(), complex_zero!(),
            complex_Im!(-FRAC_1_SQRT_2), complex_zero!(), complex_zero!(), complex_zero!(),
            complex_zero!(), complex_Im!(FRAC_1_SQRT_2), complex_zero!(), complex_zero!()
        ];
        compare_circuit(quantum_circuit, &correct_register);
    }

    #[test]
    fn add_gates_to_circuit_with_vec() {
        let mut quantum_circuit = Circuit::new(2);
        quantum_circuit.add_gates(vec!(StandardGate::Id, StandardGate::X)).unwrap();

        assert!(vec!(StandardGate::Id, StandardGate::X).iter().all(|item| quantum_circuit.circuit_gates.contains(item)));
    }

    #[test]
    fn add_repeating_gates_to_circuits() {
        let mut circuit = Circuit::new(5);
        circuit.add_repeating_gate(StandardGate::H,vec![0, 1, 2, 3, 4]).unwrap();

        assert!(vec![StandardGate::H; 5].iter().all(|item| circuit.circuit_gates.contains(item)));
    }

    #[test]
    fn add_gates_to_circuit_with_positions() {
        let mut quantum_circuit = Circuit::new(3);
        quantum_circuit.add_gates_with_positions(HashMap::from([(0, StandardGate::X), (2, StandardGate::H)])).unwrap();
        
        assert!(vec!(StandardGate::X, StandardGate::Id, StandardGate::H)
                .iter().all(|item| quantum_circuit.circuit_gates.contains(item)));
    }

    

    #[test]
    fn runs_three_pauli_gates_with_hadamard() {
        let mut circuit = Circuit::new(4);
        circuit.add_gates(vec![StandardGate::Z, StandardGate::Y, StandardGate::H, StandardGate::X]).unwrap();
        circuit.simulate();

        let correct_register = [
            complex_zero!(), complex_zero!(), complex_zero!(), complex_zero!(),
            complex_zero!(), complex_Im!(FRAC_1_SQRT_2), complex_zero!(),complex_Im!(FRAC_1_SQRT_2),
            complex_zero!(), complex_zero!(), complex_zero!(), complex_zero!(),
            complex_zero!(), complex_zero!(), complex_zero!(), complex_zero!()
        ];
        compare_circuit(circuit, &correct_register);
    }

    #[test]
    fn hash_map_with_two_gates() {
        let mut circuit = Circuit::new(3);
        circuit.add_gates_with_positions(HashMap::from([(0, StandardGate::X), (2, StandardGate::H)])).unwrap();
        circuit.simulate();
        let correct_register: [Complex<f64>; 8] = [
            complex_zero!(), complex_zero!(),
            complex_zero!(), complex_zero!(),
            complex_Re!(FRAC_1_SQRT_2), complex_Re!(FRAC_1_SQRT_2),
            complex_zero!(), complex_zero!()];
        compare_circuit(circuit, &correct_register);
    }

    #[test]
    #[should_panic]
    fn catches_repeating_positions() {
        let mut circuit = Circuit::new(4);
        circuit.add_repeating_gate(StandardGate::X, vec![0, 1, 1, 3]).unwrap();
    }

    #[test]
    fn two_hadamard_gates_work() {
        let mut circuit = Circuit::new(2);
        circuit.add_gates(vec!(StandardGate::H, StandardGate::H)).unwrap();
        circuit.simulate();

        let correct_register: [Complex<f64>; 4] = [
            complex_Re!(0.5f64), complex_Re!(0.5f64),
            complex_Re!(0.5f64), complex_Re!(0.5f64)];
        compare_circuit(circuit, &correct_register);
    }

    #[test]
    fn add_two_rows_single_gates() {
        let mut circuit = Circuit::new(4);

        circuit.add_gates_with_positions(HashMap::from([(0, StandardGate::X)])).unwrap();
        circuit.add_gates_with_positions(HashMap::from([(3, StandardGate::X), (2, StandardGate::H)])).unwrap();

        circuit.simulate();

        let correct_register = [
            complex_zero!(), complex_zero!(), complex_zero!(), complex_zero!(),
            complex_zero!(), complex_zero!(), complex_zero!(), complex_zero!(),
            complex_zero!(), complex_Re!(FRAC_1_SQRT_2), complex_zero!(), complex_Re!(FRAC_1_SQRT_2),
            complex_zero!(), complex_zero!(), complex_zero!(), complex_zero!()
        ];
        
        compare_circuit(circuit, &correct_register);
    }

    #[test]
    fn cy_and_swap_gates_work() {
        let mut circuit = Circuit::new(4);

        circuit.add_repeating_gate(StandardGate::X, vec![1,2]).unwrap();
        circuit.add_gate(StandardGate::CY(2), 0).unwrap();
        circuit.add_gate(StandardGate::Swap(3), 2).unwrap();
        circuit.add_gate(StandardGate::CY(0), 3).unwrap();

        circuit.simulate();

        let correct_register = [
            complex_zero!(), complex_zero!(), complex_zero!(), complex_zero!(),
            complex_zero!(), complex_zero!(), complex_zero!(), complex_zero!(),
            complex_zero!(), complex_zero!(), complex_zero!(), complex_zero!(),
            complex_Re!(1f64), complex_zero!(), complex_zero!(), complex_zero!()
        ];
        
        compare_circuit(circuit, &correct_register);

    }

    #[test]
    fn cz_and_swap_gates_work() {
        let mut circuit = Circuit::new(3);

        circuit.add_repeating_gate(StandardGate::X, vec![0,2]).unwrap();
        circuit.add_gate(StandardGate::Swap(1), 2).unwrap();
        circuit.add_gate(StandardGate::CZ(1), 0).unwrap();

        circuit.simulate();

        let correct_register = [
            complex_zero!(), complex_zero!(), complex_zero!(), complex_zero!(),
            complex_zero!(), complex_zero!(), complex_Re!(-1f64), complex_zero!(),
            complex_zero!(), complex_zero!(), complex_zero!(), complex_zero!(),
            complex_zero!(), complex_zero!(), complex_zero!(), complex_zero!()
        ];
        
        compare_circuit(circuit, &correct_register);
    }

    #[test]
    fn cnot_gate_simply_use_works() {
        let mut circuit = Circuit::new(2);

        circuit.add_gate(StandardGate::H, 0).unwrap();
        circuit.add_gate(StandardGate::CNot(1), 0).unwrap();

        circuit.simulate();

        let correct_register: [Complex<f64>; 4] = [
            complex_Re!(FRAC_1_SQRT_2), complex_zero!(),
            complex_Re!(FRAC_1_SQRT_2), complex_zero!()
        ];
        
        compare_circuit(circuit, &correct_register);

    }

    #[test]
    fn cnot_gate_simply_flipped() {
        let mut circuit = Circuit::new(2);

        circuit.add_gate(StandardGate::H, 0).unwrap();
        circuit.add_gate(StandardGate::CNot(0), 1).unwrap();

        circuit.simulate();

        let correct_register: [Complex<f64>; 4] = [
            complex_Re!(FRAC_1_SQRT_2), complex_zero!(),
            complex_zero!(), complex_Re!(FRAC_1_SQRT_2)
        ];

        compare_circuit(circuit, &correct_register);

    }

    #[test]
    fn cnot_gate_extended_control_works_asymmetric() {
        let mut circuit = Circuit::new(4);

        circuit.add_gate(StandardGate::H, 1).unwrap();
        circuit.add_gate(StandardGate::CNot(1), 3).unwrap(); 
        circuit.add_gate(StandardGate::Y, 1).unwrap();

        circuit.simulate();

        let correct_register = [
            complex_zero!(), complex_Im!(-FRAC_1_SQRT_2), complex_zero!(), complex_zero!(),
            complex_Im!(FRAC_1_SQRT_2), complex_zero!(), complex_zero!(), complex_zero!(),
            complex_zero!(), complex_zero!(), complex_zero!(), complex_zero!(),
            complex_zero!(), complex_zero!(), complex_zero!(), complex_zero!()
        ];

        compare_circuit(circuit, &correct_register);

    }
}
