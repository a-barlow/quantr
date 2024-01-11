/*
* Copyright (c) 2023 Andrew Rowan Barlow. Licensed under the EUPL-1.2
* or later. You may obtain a copy of the licence at
* https://joinup.ec.europa.eu/collection/eupl/eupl-text-eupl-12. A copy
* of the EUPL-1.2 licence in English is given in LICENCE.txt which is
* found in the root directory of this repository.
*
* Author: Andrew Rowan Barlow <a.barlow.dev@gmail.com>
*/

// Added only for silencing deprecated warnings for using public fields of `Circuit`.
#![allow(deprecated)]

use super::circuit::gate::{GateCategory, GateInfo};
use crate::states::{ProductState, SuperPosition};
use crate::{Gate, QuantrError};
use std::collections::HashMap;
use std::iter::zip;

pub mod gate;
pub mod printer;
mod simulation;
mod standard_gate_ops;
pub mod states;

pub(crate) type QResult<T> = Result<T, QuantrError>;

// The tolerance for declaring non-zero amplitudes.
const ZERO_MARGIN: f64 = 1e-7;

/// Distinguishes observable and non-observable quantities.
///
/// For example, this will distinguish the retrieval of a superposition (that cannot be measured
/// directly), and the state resulting from the collapse of a superposition upon measurement. See
/// [Circuit::get_superposition] and [Circuit::repeat_measurement] for examples.
pub enum Measurement<T> {
    Observable(T),
    NonObservable(T),
}

/// A quantum circuit where gates can be appended and then simulated to measure resulting
/// superpositions.
pub struct Circuit<'a> {
    #[deprecated(
        note = "This field will be made private to the user, where it will be given pub(crate) status in the next major update. Use Circuit::get_gates instead."
    )]
    // Change this to Vec<CategoryGate> in next major update.
    pub circuit_gates: Vec<Gate<'a>>,
    #[deprecated(
        note = "This field will be made private to the user, where it will be given pub(crate) status in the next major update. Use Circuit::get_num_qubits instead."
    )]
    pub num_qubits: usize,
    output_state: Option<SuperPosition>,
    register: Option<SuperPosition>,
    config_progress: bool,
}

impl<'a> Circuit<'a> {
    /// Initialises a new circuit.
    ///
    /// The lifetime is due to the slices of control qubits for [Gate::Custom]. That is, the slice
    /// argument must outlive the circuit.
    ///
    /// # Example
    /// ```
    /// use quantr::Circuit;
    ///
    /// // Initialises a 3 qubit circuit.
    /// let quantum_circuit: Circuit = Circuit::new(3).unwrap();
    /// ```
    pub fn new(num_qubits: usize) -> QResult<Circuit<'a>> {
        if num_qubits == 0 {
            return Err(QuantrError {
                message: String::from("The initiliased circuit must have at least one wire."),
            });
        }

        let circuit_gates: Vec<Gate> = Vec::new();
        Ok(Circuit {
            circuit_gates,
            num_qubits,
            output_state: None,
            register: None,
            config_progress: false,
        })
    }

    /// Returns the number of qubits in the circuit.
    ///
    /// # Example
    /// ```
    /// use quantr::{Circuit, Gate};
    ///
    /// let quantum_circuit: Circuit = Circuit::new(3).unwrap();
    /// assert_eq!(quantum_circuit.get_num_qubits(), 3usize);
    /// ```
    pub fn get_num_qubits(self) -> usize {
        self.num_qubits
    }

    /// Returns the vector of gates that have been added to the circuit.
    ///
    /// It is a flattened vector which is buffered with identity gates.
    ///
    /// # Example
    /// ```
    /// use quantr::{Circuit, Gate};
    ///
    /// let mut quantum_circuit: Circuit = Circuit::new(3).unwrap();
    /// quantum_circuit.add_gate(Gate::X, 2).unwrap();
    ///
    /// assert_eq!(quantum_circuit.get_gates(), &[Gate::Id, Gate::Id, Gate::X]);
    /// ```
    pub fn get_gates(&self) -> &[Gate<'a>] {
        self.circuit_gates.as_slice()
    }

    /// Adds a single gate to the circuit.
    ///
    /// If wanting to add multiple gates, or a single gate repeatedly across multiple wires, see
    /// [Circuit::add_gates_with_positions] and [Circuit::add_repeating_gate] respectively.
    ///
    /// # Example
    /// ```
    /// use quantr::{Circuit, Gate};
    ///
    /// let mut quantum_circuit: Circuit = Circuit::new(3).unwrap();
    /// quantum_circuit.add_gate(Gate::X, 0).unwrap();
    ///
    /// // Produces the circuit:
    /// // -- X --
    /// // -------
    /// // -------
    /// ```
    pub fn add_gate(&mut self, gate: Gate<'a>, position: usize) -> QResult<&mut Circuit<'a>> {
        Self::add_gates_with_positions(self, HashMap::from([(position, gate)]))
    }

    /// Add a column of gates specifying the position for each gate.
    ///
    /// A `HashMap<usize, Gate>` is used to place gates onto their desired position.
    /// This is similar to [Circuit::add_gate], however not
    /// all wires have to be accounted for.
    ///
    /// # Example
    /// ```
    /// use quantr::{Circuit, Gate};
    /// use std::collections::HashMap;
    ///
    /// let mut quantum_circuit: Circuit = Circuit::new(3).unwrap();
    /// // Adds gates on wires 0 and 2, implicitly leaving wire 1 bare.
    /// quantum_circuit.add_gates_with_positions(
    ///     HashMap::from(
    ///         [(0, Gate::X), (2, Gate::H)]
    ///     )
    /// ).unwrap();
    ///
    /// // Produces the circuit:
    /// // -- X --
    /// // -------
    /// // -- H --
    /// ```
    pub fn add_gates_with_positions(
        &mut self,
        gates_with_positions: HashMap<usize, Gate<'a>>,
    ) -> QResult<&mut Circuit<'a>> {
        // If any keys are out of bounds, return an error.
        if let Some(out_of_bounds_key) =
            gates_with_positions.keys().find(|k| *k >= &self.num_qubits)
        {
            return Err(QuantrError {
                message: format!(
                    "The position, {}, is out of bounds for the circuit with {} qubits.",
                    out_of_bounds_key, self.num_qubits
                ),
            });
        }

        // Add gates from `gates_with_positions` based on their positions. For the lines that don't
        // have a gate, the identity is added.
        let mut gates_to_add: Vec<Gate> = Default::default();
        for row_num in 0..self.num_qubits {
            gates_to_add.push(
                gates_with_positions
                    .get(&row_num)
                    .unwrap_or(&Gate::Id)
                    .clone(),
            );
        }

        // No overlapping gates
        Self::has_overlapping_controls_and_target(&gates_to_add, self.num_qubits)?;

        // Push any multi-controlled gates to isolated columns
        Self::push_multi_gates(&mut gates_to_add)?;

        self.circuit_gates.extend(gates_to_add);
        Ok(self)
    }

    /// Add a column of gates.
    ///
    /// Expects the input vector to specify the gate that is added to *each* wire. That is, the
    /// length of the vector should equal the number of wires. To only add gates based on their
    /// positions, see [Circuit::add_gates_with_positions] and [Circuit::add_gate].
    ///
    /// # Example   
    /// ```
    /// use quantr::{Circuit, Gate};
    ///
    /// let mut quantum_circuit: Circuit = Circuit::new(3).unwrap();
    /// let gates_to_add = [Gate::H, Gate::X, Gate::Y];
    ///
    /// quantum_circuit.add_gates(&gates_to_add).unwrap();
    ///
    /// // Produces the circuit:
    /// // -- H --
    /// // -- X --
    /// // -- Y --
    /// ```
    pub fn add_gates(&mut self, gates: &[Gate<'a>]) -> QResult<&mut Circuit<'a>> {
        // Ensured we have a gate for every wire.
        if gates.len() != self.num_qubits {
            return Err(QuantrError {
                message: format!("The number of gates, {}, does not match the number of wires, {}. All wires must have gates added.", gates.len(), self.num_qubits)
            });
        }

        // Make sure there are no control nodes that overlap with it's other nodes.
        Self::has_overlapping_controls_and_target(gates, self.num_qubits)?;

        // Push n-gates to another line (double, triple, etc.)
        let mut gates_vec: Vec<Gate<'a>> = gates.to_vec();
        Self::push_multi_gates(&mut gates_vec)?;
        self.circuit_gates.extend(gates_vec);
        Ok(self)
    }

    // Pushes multi-controlled gates into their own column. Potentially expensive operation to
    // insert new elements at smaller positions into a long vector.
    fn push_multi_gates(gates: &mut Vec<Gate<'a>>) -> QResult<()> {
        let mut extended_vec: Vec<Gate> = Default::default();
        let mut multi_gate_positions: Vec<usize> = Default::default();

        // if its a column with only a multi-control gate, leave it
        let mut found_multi: bool = false;
        let mut found_second: bool = false;
        for gate in gates.iter() {
            if let Gate::Custom(_, _, name) = gate {
                if !name.is_ascii() {
                    return Err(QuantrError { message: format!("The custom function name, {}, does not only use ASCII chars. This could lead to problems in printing the circuit diagram. This warning will be promoted to an Error in the next major release.", name) } );
                }
            }
            if gate != &Gate::Id {
                if found_multi {
                    found_second = true;
                    break;
                }
                found_multi = true;
            }
        }
        if !found_second {
            gates.extend(extended_vec)
        } else {
            for (pos, gate) in gates.iter().enumerate() {
                if !gate.is_single_gate() {
                    let mut temp_vec = vec![Gate::Id; gates.len()];
                    temp_vec[pos] = gate.clone();
                    extended_vec.extend(temp_vec);
                    multi_gate_positions.push(pos);
                }
            }

            for i in multi_gate_positions {
                gates[i] = Gate::Id;
            }
            gates.extend(extended_vec);
        }

        Ok(())
    }

    // need to implement all other gates, in addition to checking that it's within circuit size!
    fn has_overlapping_controls_and_target(gates: &[Gate], circuit_size: usize) -> QResult<()> {
        for (pos, gate) in gates.iter().enumerate() {
            if let Some(nodes) = gate.get_nodes() {
                // check for overlapping control nodes.
                if Self::contains_repeating_values(circuit_size, &nodes) {
                    return Err(QuantrError {
                        message: format!("The gate, {:?}, has overlapping control nodes.", gate),
                    });
                }
                if nodes.contains(&pos) {
                    return Err(QuantrError { message: format!("The gate, {:?}, has a control node that equals the gate's position {}.", gate, pos) });
                }
                for &node in nodes.iter() {
                    if node >= circuit_size {
                        return Err(QuantrError { message: format!("The control node at position {:?}, is greater than the umnber of qubits {}.", node, circuit_size) });
                    }
                }
            }
        }

        Ok(())
    }

    // Find if there are any repeating values in array, O(n)
    // The initialisation of the circuit guarantees the max circuit size.
    fn contains_repeating_values(num_qubits: usize, array: &[usize]) -> bool {
        let mut counter: Vec<bool> = vec![false; num_qubits];
        for j in array {
            if counter[*j] {
                return true;
            };
            counter[*j] = true;
        }
        false
    }

    /// Place a single gate repeatedly onto multiple wires.
    ///
    /// For adding multiple different gates, refer to [Circuit::add_gates]
    /// and [Circuit::add_gates_with_positions].
    ///
    /// # Example
    /// ```
    /// use quantr::{Circuit, Gate};
    ///
    /// let mut quantum_circuit: Circuit = Circuit::new(3).unwrap();
    /// quantum_circuit.add_repeating_gate(Gate::H, &[1, 2]).unwrap();
    ///
    /// // Produces the circuit:
    /// // -------
    /// // -- H --
    /// // -- H --
    /// ```
    pub fn add_repeating_gate(
        &mut self,
        gate: Gate<'a>,
        positions: &[usize],
    ) -> QResult<&mut Circuit<'a>> {
        // Incase the user has attempted to place the gate twice on the same wire.
        if Self::contains_repeating_values(self.num_qubits, positions) {
            return Err(QuantrError {
                message: format!(
                    "Attempted to add more than one gate onto a single wire. The positions in {:?} must all differ.", positions 
                ),
            });
        }

        // Generates a list of identity gates, that are subsequently replaced by non-trivial gates
        // specified by the user.
        let list_of_identities: Vec<Gate> = vec![Gate::Id; self.num_qubits];
        let gates: Vec<Gate> = list_of_identities
            .iter()
            .enumerate()
            .map(|(pos, _)| {
                if positions.contains(&pos) {
                    gate.clone()
                } else {
                    Gate::Id
                }
            })
            .collect();

        self.add_gates(gates.as_slice())
    }

    /// Attaches the register, |0...0>, to the circuit resulting in a superposition that can be measured.
    ///
    /// See [Circuit::get_superposition] and [Circuit::repeat_measurement] for details on obtaining
    /// observables from the resulting superposition.
    ///
    /// # Example
    /// ```
    /// use quantr::{Circuit, Gate};
    ///
    /// let mut circuit = Circuit::new(3).unwrap();
    /// circuit.add_gate(Gate::H, 2).unwrap();
    ///
    /// circuit.simulate();
    ///
    /// // Simulates the circuit:
    /// // |0> -------
    /// // |0> -- H --
    /// // |0> -- H --
    /// ````
    pub fn simulate(&mut self) {
        // Form the initial state if the product space, that is |0...0>
        let mut register: SuperPosition = match &self.register {
            Some(custom_register) => custom_register.clone(),
            None => SuperPosition::new_unchecked(self.num_qubits),
        };
        let mut qubit_counter: usize = 0;
        let number_gates: usize = self.circuit_gates.len();

        // This will removed in next major update, as the circuit will directly store this. Instead
        // of what's happening now, in which the gates are being copied into another wapper.
        let mut categorised_gates: Vec<GateCategory> = Vec::with_capacity(number_gates);
        for gate in &self.circuit_gates {
            categorised_gates.push(Gate::linker(gate));
        }

        if self.config_progress {
            println!("Starting circuit simulation...");
        }

        // Loop through each gate of circuit from starting at top row to bottom, then moving onto the next.
        for (cat_gate, gate) in zip(categorised_gates, &self.circuit_gates) {
            if cat_gate == GateCategory::Identity {
                qubit_counter += 1;
                continue;
            }

            let gate_pos: usize = qubit_counter % self.num_qubits;

            if self.config_progress {
                Self::print_circuit_log(gate, &gate_pos, &qubit_counter, &number_gates);
            }

            let gate_to_apply: GateInfo = GateInfo {
                cat_gate,
                position: gate_pos,
            };
            Circuit::apply_gate(gate_to_apply, &mut register);

            qubit_counter += 1;
        }

        self.output_state = Some(register);
    }

    /// Returns the resulting superposition after the circuit has been simulated using
    /// [Circuit::simulate].
    ///
    /// This is a non-physical observable, as the superposition would reduce to a single state upon measurement.
    ///
    /// # Example
    /// ```
    /// use quantr::{states::SuperPosition, Circuit, Measurement::NonObservable, Gate};
    ///
    /// let mut circuit = Circuit::new(3).unwrap();
    ///
    /// circuit.add_gate(Gate::H, 2).unwrap();
    /// circuit.add_gate(Gate::Y, 2).unwrap();
    /// circuit.simulate();
    ///
    /// println!("State | Amplitude of State");
    /// if let Ok(NonObservable(super_pos)) = circuit.get_superposition() {
    ///     for (state, amplitude) in super_pos.into_iter() {
    ///         println!("|{}>   : {}", state.to_string(), amplitude);
    ///     }
    /// }
    ///
    /// // State | Amplitude of State    
    /// // |000> : 0 - 0.71...i     
    /// // |001> : 0 + 0.71...i
    /// ```
    pub fn get_superposition(&self) -> QResult<Measurement<&SuperPosition>> {
        match &self.output_state {
            Some(super_position) => Ok(Measurement::NonObservable(super_position)),
            None => {
                Err(QuantrError {
                    message: "The circuit has not been simulated. Call Circuit::simulate before calling this method, Circuit::get_superposition.".to_string(),
                })
            }
        }
    }

    /// Returns a `HashMap` that containes the number of times the corresponding state was observed over
    /// `n` measurements of the superpositions.
    ///
    /// Explicitly, this performs repeated measurements where a register is attached to the circuit,
    /// the resulting superposition measured, and then the reduced state recorded. If the HashMap does not
    /// include a product state, then it was not observed over the `n` measurements. This method
    /// requires that the circuit has already been simulated by calling [Circuit::simulate].
    ///
    /// # Example
    /// ```
    /// use quantr::{states::SuperPosition, Circuit, Measurement::Observable, Gate};
    ///
    /// let mut circuit = Circuit::new(3).unwrap();
    ///
    /// circuit.add_gate(Gate::H, 2).unwrap();
    /// circuit.simulate();
    ///
    /// // Measures 500 superpositions.
    /// println!("State | Number of Times Observed");
    /// if let Ok(Observable(bin_count)) = circuit.repeat_measurement(500) {
    ///     for (state, observed_count) in bin_count {
    ///         println!("|{}>   : {}", state.to_string(), observed_count);
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
    ) -> QResult<Measurement<HashMap<ProductState, usize>>> {
        match &self.output_state {
            Some(super_position) => {
                // Peform bin count of states
                let mut probabilities: HashMap<ProductState, f64> = Default::default();
                for (key, value) in super_position.to_hash_map() {
                    probabilities.insert(key, value.abs_square());
                }

                let mut bin_count: HashMap<ProductState, usize> = Default::default();

                for _ in 0..number_iterations {
                    let mut cummalitive: f64 = 0f64;
                    let dice_roll: f64 = fastrand::f64();
                    for (state_label, probability) in &probabilities {
                        cummalitive += probability;
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
                    message: "The circuit has not been simulated. Call Circuit::simulate before calling this method, Circuit::repeat_measurement.".to_string(),
                })
            },
        }
    }

    /// Changes the register which is applied to the circuit when [Circuit::simulate] is called.
    ///
    /// The default register is the |00..0> state. This method can be used before simulating the
    /// circuit to change the register. This is primarily helpful in defining custom functions, for
    /// example see `examples/qft.rs`.
    ///
    /// # Example
    /// ```
    /// use quantr::{Circuit, Gate};
    /// use quantr::states::{Qubit, ProductState, SuperPosition};
    ///
    /// let mut circuit = Circuit::new(2).unwrap();
    /// circuit.add_gate(Gate::X, 1).unwrap();
    ///
    /// let register: SuperPosition =
    ///     ProductState::new(&[Qubit::One, Qubit::Zero])
    ///         .unwrap()
    ///         .into();
    ///
    /// circuit.change_register(register).unwrap();
    /// circuit.simulate();
    ///
    /// // Simulates the circuit:
    /// // |1> -------
    /// // |0> -- X --
    /// ````
    pub fn change_register(&mut self, super_pos: SuperPosition) -> QResult<&mut Circuit<'a>> {
        if super_pos.product_dim != self.num_qubits {
            return Err(QuantrError {
                message: format!("The custom register has a product state dimension of {}, while the number of qubits is {}. These must equal each other.", super_pos.product_dim, self.num_qubits)
            });
        }

        self.register = Some(super_pos);

        Ok(self)
    }

    /// Toggles if the circuit should print the progress of simulating each gate.
    ///
    /// It will only show the application of non-identity gates. The toggle is set to `false` by
    /// default for a new quantum circuit.
    ///
    /// # Example
    /// ```
    /// use quantr::{Circuit, Gate};
    ///
    /// let mut circuit = Circuit::new(3).unwrap();
    /// circuit.add_gate(Gate::H, 2).unwrap();
    ///
    /// circuit.toggle_simulation_progress();
    ///
    /// circuit.simulate(); // Simulates and prints progress.
    /// ```
    pub fn toggle_simulation_progress(&mut self) {
        self.config_progress = !self.config_progress;
    }
}

#[rustfmt::skip]
#[cfg(test)]
mod tests {
    use crate::{complex_im, complex_re, complex_re_array, complex, COMPLEX_ZERO, Gate, Complex, Circuit};
    use crate::states::{SuperPosition, Qubit, ProductState};
    use super::HashMap;
    use std::f64::consts::{FRAC_1_SQRT_2, PI};
    use crate::Measurement::NonObservable;
    const ERROR_MARGIN: f64 = 0.000001f64; // For comparing floats due to floating point error.
    // Needed for testing
    fn compare_complex_lists_and_register(correct_list: &[Complex<f64>], register: &SuperPosition) {
        for (i, &comp_num) in register.amplitudes.iter().enumerate() { // Make sure that it turns up complex
            assert!(equal_within_error(comp_num.re, correct_list[i].re));
            assert!(equal_within_error(comp_num.im, correct_list[i].im));
        }
    }

    fn equal_within_error(num: f64, compare_num: f64) -> bool {
        num < compare_num + ERROR_MARGIN && num > compare_num - ERROR_MARGIN
    }

    fn compare_circuit(quantum_circuit: Circuit, correct_register: &[Complex<f64>]) {
        if let NonObservable(measured_register) = quantum_circuit.get_superposition().unwrap() {
            compare_complex_lists_and_register(correct_register, measured_register);
        }
    }

    fn example_cnot(prod: ProductState) -> Option<SuperPosition> {
        let input_register: [Qubit; 2] = [prod.qubits[0], prod.qubits[1]];
        Some(SuperPosition::new_with_amplitudes(match input_register {
            [Qubit::Zero, Qubit::Zero] => return None,
            [Qubit::Zero, Qubit::One]  => return None,
            [Qubit::One, Qubit::Zero]  => &complex_re_array!(0f64, 0f64, 0f64, 1f64),
            [Qubit::One, Qubit::One]   => &complex_re_array!(0f64, 0f64, 1f64, 0f64),
        }).unwrap())
    }

    // No expected panic message as the eample_cnot function is an address in memory, that will
    // change everytime.
    #[test]
    #[should_panic]
    fn catches_overlapping_nodes_custom_gate() {
        let mut quantum_circuit = Circuit::new(3).unwrap();
        quantum_circuit
            .add_gates(&[Gate::Id, Gate::Custom(example_cnot, &[1], "X".to_string()), Gate::Id])
            .unwrap();
    }
    
    #[test]
    #[should_panic]
    fn catches_overlapping_control_nodes() {
        let mut quantum_circuit = Circuit::new(3).unwrap();
        quantum_circuit
            .add_gates(&[Gate::CNot(0), Gate::Id, Gate::Id])
            .unwrap();
    }

    #[test]
    fn pushes_multi_gates() {
        let mut quantum_circuit = Circuit::new(3).unwrap();
        quantum_circuit
            .add_gates(&[Gate::CNot(2), Gate::CNot(0), Gate::H]).unwrap()
            .add_gates(&[Gate::Toffoli(1, 2), Gate::H, Gate::CNot(0)]).unwrap();
    
        let correct_circuit_layout: Vec<Gate> = vec![
            Gate::Id, Gate::Id, Gate::H,
            Gate::CNot(2), Gate::Id, Gate::Id,
            Gate::Id, Gate::CNot(0), Gate::Id,
            Gate::Id, Gate::H, Gate::Id,
            Gate::Toffoli(1, 2), Gate::Id, Gate::Id,
            Gate::Id, Gate::Id, Gate::CNot(0)];

        assert_eq!(correct_circuit_layout, quantum_circuit.circuit_gates);
    }

    #[test]
    fn pushes_multi_gates_using_vec() {
        let mut quantum_circuit = Circuit::new(3).unwrap();
        quantum_circuit.add_gates_with_positions(HashMap::from([
            (2, Gate::H),
            (0, Gate::CNot(2)),
            (1, Gate::CNot(0))
        ])).unwrap()
        .add_gates_with_positions(HashMap::from([
            (2, Gate::CNot(0)),
            (0, Gate::Toffoli(1, 2)),
            (1, Gate::H)
        ])).unwrap();
    
        let correct_circuit_layout: Vec<Gate> = vec![
            Gate::Id, Gate::Id, Gate::H,
            Gate::CNot(2), Gate::Id, Gate::Id,
            Gate::Id, Gate::CNot(0), Gate::Id,
            Gate::Id, Gate::H, Gate::Id,
            Gate::Toffoli(1, 2), Gate::Id, Gate::Id,
            Gate::Id, Gate::Id, Gate::CNot(0)];

        assert_eq!(correct_circuit_layout, quantum_circuit.circuit_gates);
    }

    #[test]
    #[should_panic]
    fn catches_overlapping_control_nodes_using_vec() {
        let mut quantum_circuit = Circuit::new(3).unwrap();
        quantum_circuit.add_gates_with_positions(HashMap::from([
            (2, Gate::H),
            (0, Gate::CNot(0)),
            (1, Gate::CNot(0))
        ])).unwrap();
    }

    #[test]
    #[should_panic]
    fn control_node_greater_than_circuit_size() {
        let mut quantum_circuit = Circuit::new(3).unwrap();
        quantum_circuit.add_gates_with_positions(HashMap::from([
            (2, Gate::H),
            (0, Gate::CNot(2)),
            (1, Gate::CNot(3))
        ])).unwrap();
    }

    //
    // All circuit tests were calculated by hand.
    //
    
    #[test]
    fn swap_and_conjugate_gates() {
        let mut circuit = Circuit::new(2).unwrap();
        circuit.add_gates(&[Gate::H, Gate::H]).unwrap()
            .add_gates(&[Gate::S, Gate::Sdag]).unwrap()
            .simulate();

        let correct_register: [Complex<f64>; 4] = [
            complex_re!(0.5f64), complex_im!(-0.5f64),
            complex_im!(0.5f64), complex_re!(0.5f64)];
        compare_circuit(circuit, &correct_register);
    }

    #[test]
    fn t_and_conjugate_gates() {
        let mut circuit = Circuit::new(2).unwrap();
        circuit.add_gates(&[Gate::H, Gate::H]).unwrap()
               .add_gates(&[Gate::T, Gate::Tdag]).unwrap()
               .simulate();

        let correct_register: [Complex<f64>; 4] = [
            complex_re!(0.5f64), complex!(0.5f64*FRAC_1_SQRT_2, -0.5f64*FRAC_1_SQRT_2),
            complex!(0.5f64*FRAC_1_SQRT_2, 0.5f64*FRAC_1_SQRT_2), complex_re!(0.5f64)];
        compare_circuit(circuit, &correct_register);
    }


    #[test]
    fn custom_gates() {
        let mut quantum_circuit = Circuit::new(3).unwrap();
        quantum_circuit.add_gate(Gate::H, 2).unwrap()
            .add_gate(Gate::Custom(example_cnot, &[2], String::from("cNot")), 1).unwrap()
            .simulate();

        let correct_register: [Complex<f64>; 8] = [
            complex_re!(FRAC_1_SQRT_2), COMPLEX_ZERO,
            COMPLEX_ZERO, complex_re!(FRAC_1_SQRT_2),
            COMPLEX_ZERO, COMPLEX_ZERO,
            COMPLEX_ZERO, COMPLEX_ZERO];

        compare_circuit(quantum_circuit, &correct_register);
    }

    #[test]
    fn toffoli_gates() {
        let mut quantum_circuit = Circuit::new(4).unwrap();
        quantum_circuit.add_gate(Gate::X, 0).unwrap()
            .add_gate(Gate::H, 3).unwrap()
            .add_gate(Gate::Y, 3).unwrap()
            .add_gate(Gate::Toffoli(3, 0), 1).unwrap()
            .simulate();

        let correct_register = [
            COMPLEX_ZERO, COMPLEX_ZERO, COMPLEX_ZERO, COMPLEX_ZERO,
            COMPLEX_ZERO, COMPLEX_ZERO, COMPLEX_ZERO, COMPLEX_ZERO,
            complex_im!(-FRAC_1_SQRT_2), COMPLEX_ZERO, COMPLEX_ZERO, COMPLEX_ZERO,
            COMPLEX_ZERO, complex_im!(FRAC_1_SQRT_2), COMPLEX_ZERO, COMPLEX_ZERO
        ];
        compare_circuit(quantum_circuit, &correct_register);
    }

    #[test]
    fn add_gates_to_circuit_with_vec() {
        let mut quantum_circuit = Circuit::new(2).unwrap();
        quantum_circuit
            .add_gates(&[Gate::Id, Gate::X]).unwrap();

        assert!(vec!(Gate::Id, Gate::X).iter().all(|item| quantum_circuit.circuit_gates.contains(item)));
    }

    #[test]
    fn add_repeating_gates_to_circuits() {
        let mut circuit = Circuit::new(5).unwrap();
        circuit
            .add_repeating_gate(Gate::H, &[0, 1, 2, 3, 4]).unwrap();

        assert!(vec![Gate::H; 5].iter().all(|item| circuit.circuit_gates.contains(item)));
    }

    #[test]
    fn add_gates_to_circuit_with_positions() {
        let mut quantum_circuit = Circuit::new(3).unwrap();
        quantum_circuit
            .add_gates_with_positions(HashMap::from([(0, Gate::X), (2, Gate::H)])).unwrap();
        
        assert!(vec!(Gate::X, Gate::Id, Gate::H)
                .iter().all(|item| quantum_circuit.circuit_gates.contains(item)));
    }

    #[test]
    fn runs_three_pauli_gates_with_hadamard() {
        let mut circuit: Circuit = Circuit::new(4).unwrap();
        circuit
            .add_gates(&[Gate::Z, Gate::Y, Gate::H, Gate::X]).unwrap()
            .simulate();

        let correct_register = [
            COMPLEX_ZERO, COMPLEX_ZERO, COMPLEX_ZERO, COMPLEX_ZERO,
            COMPLEX_ZERO, complex_im!(FRAC_1_SQRT_2), COMPLEX_ZERO, complex_im!(FRAC_1_SQRT_2),
            COMPLEX_ZERO, COMPLEX_ZERO, COMPLEX_ZERO, COMPLEX_ZERO,
            COMPLEX_ZERO, COMPLEX_ZERO, COMPLEX_ZERO, COMPLEX_ZERO
        ];
        compare_circuit(circuit, &correct_register);
    }

    #[test]
    fn hash_map_with_two_gates() {
        let mut circuit = Circuit::new(3).unwrap();
        circuit.add_gates_with_positions(HashMap::from([(0, Gate::X), (2, Gate::H)])).unwrap().simulate();
        let correct_register: [Complex<f64>; 8] = [
            COMPLEX_ZERO, COMPLEX_ZERO,
            COMPLEX_ZERO, COMPLEX_ZERO,
            complex_re!(FRAC_1_SQRT_2), complex_re!(FRAC_1_SQRT_2),
            COMPLEX_ZERO, COMPLEX_ZERO];
        compare_circuit(circuit, &correct_register);
    }

    #[test]
    #[should_panic]
    fn catches_repeating_positions() {
        let mut circuit = Circuit::new(4).unwrap();
        circuit.add_repeating_gate(Gate::X, &[0, 1, 1, 3]).unwrap();
    }

    #[test]
    fn two_hadamard_gates_work() {
        let mut circuit = Circuit::new(2).unwrap();
        circuit.add_gates(&[Gate::H, Gate::H]).unwrap().simulate();

        let correct_register: [Complex<f64>; 4] = [
            complex_re!(0.5f64), complex_re!(0.5f64),
            complex_re!(0.5f64), complex_re!(0.5f64)];
        compare_circuit(circuit, &correct_register);
    }

    #[test]
    fn add_two_rows_single_gates() {
        let mut circuit = Circuit::new(4).unwrap();

        circuit.add_gates_with_positions(HashMap::from([(0, Gate::X)])).unwrap()
                .add_gates_with_positions(HashMap::from([(3, Gate::X), (2, Gate::H)])).unwrap()
                .simulate();

        let correct_register = [
            COMPLEX_ZERO, COMPLEX_ZERO, COMPLEX_ZERO, COMPLEX_ZERO,
            COMPLEX_ZERO, COMPLEX_ZERO, COMPLEX_ZERO, COMPLEX_ZERO,
            COMPLEX_ZERO, complex_re!(FRAC_1_SQRT_2), COMPLEX_ZERO, complex_re!(FRAC_1_SQRT_2),
            COMPLEX_ZERO, COMPLEX_ZERO, COMPLEX_ZERO, COMPLEX_ZERO
        ];
        
        compare_circuit(circuit, &correct_register);
    }

    #[test]
    fn cy_and_swap_gates_work() {
        let mut circuit = Circuit::new(4).unwrap();

        circuit.add_repeating_gate(Gate::X, &[1,2]).unwrap()
            .add_gate(Gate::CY(2), 0).unwrap()
            .add_gate(Gate::Swap(3), 2).unwrap()
            .add_gate(Gate::CY(0), 3).unwrap()
            .simulate();

        let correct_register = [
            COMPLEX_ZERO, COMPLEX_ZERO, COMPLEX_ZERO, COMPLEX_ZERO,
            COMPLEX_ZERO, COMPLEX_ZERO, COMPLEX_ZERO, COMPLEX_ZERO,
            COMPLEX_ZERO, COMPLEX_ZERO, COMPLEX_ZERO, COMPLEX_ZERO,
            complex_re!(1f64), COMPLEX_ZERO, COMPLEX_ZERO, COMPLEX_ZERO
        ];
        
        compare_circuit(circuit, &correct_register);

    }

    #[test]
    fn cz_and_swap_gates_work() {
        let mut circuit = Circuit::new(3).unwrap();

        circuit.add_repeating_gate(Gate::X, &[0,2]).unwrap()
            .add_gate(Gate::Swap(1), 2).unwrap()
            .add_gate(Gate::CZ(1), 0).unwrap()
            .simulate();

        let correct_register = [
            COMPLEX_ZERO, COMPLEX_ZERO, COMPLEX_ZERO, COMPLEX_ZERO,
            COMPLEX_ZERO, COMPLEX_ZERO, complex_re!(-1f64), COMPLEX_ZERO,
            COMPLEX_ZERO, COMPLEX_ZERO, COMPLEX_ZERO, COMPLEX_ZERO,
            COMPLEX_ZERO, COMPLEX_ZERO, COMPLEX_ZERO, COMPLEX_ZERO
        ];
        
        compare_circuit(circuit, &correct_register);
    }

    #[test]
    fn cnot_gate_simply_use_works() {
        let mut circuit = Circuit::new(2).unwrap();

        circuit.add_gate(Gate::H, 0).unwrap()
            .add_gate(Gate::CNot(1), 0).unwrap()
            .simulate();

        let correct_register: [Complex<f64>; 4] = [
            complex_re!(FRAC_1_SQRT_2), COMPLEX_ZERO,
            complex_re!(FRAC_1_SQRT_2), COMPLEX_ZERO
        ];
        
        compare_circuit(circuit, &correct_register);

    }

    #[test]
    fn cnot_gate_simply_flipped() {
        let mut circuit = Circuit::new(2).unwrap();

        circuit.add_gate(Gate::H, 0).unwrap()
            .add_gate(Gate::CNot(0), 1).unwrap()
            .simulate();

        let correct_register: [Complex<f64>; 4] = [
            complex_re!(FRAC_1_SQRT_2), COMPLEX_ZERO,
            COMPLEX_ZERO, complex_re!(FRAC_1_SQRT_2)
        ];

        compare_circuit(circuit, &correct_register);

    }

    #[test]
    fn cnot_gate_extended_control_works_asymmetric() {
        let mut circuit = Circuit::new(4).unwrap();

        circuit.add_gate(Gate::H, 1).unwrap()
            .add_gate(Gate::CNot(1), 3).unwrap()
            .add_gate(Gate::Y, 1).unwrap()
            .simulate();

        let correct_register = [
            COMPLEX_ZERO, complex_im!(-FRAC_1_SQRT_2), COMPLEX_ZERO, COMPLEX_ZERO,
            complex_im!(FRAC_1_SQRT_2), COMPLEX_ZERO, COMPLEX_ZERO, COMPLEX_ZERO,
            COMPLEX_ZERO, COMPLEX_ZERO, COMPLEX_ZERO, COMPLEX_ZERO,
            COMPLEX_ZERO, COMPLEX_ZERO, COMPLEX_ZERO, COMPLEX_ZERO
        ];

        compare_circuit(circuit, &correct_register);

    }
    
    #[test]
    #[should_panic]
    fn custom_non_ascii_name() {
        let mut circuit = Circuit::new(3).unwrap();

        circuit.add_gate(Gate::Custom(example_cnot, &[0], "NonAscii†".to_string()), 1).unwrap();
    }

    #[test]
    fn rx_gate() {
        let mut circuit = Circuit::new(2).unwrap();

        circuit.add_gates(&[Gate::H, Gate::H]).unwrap()
            .add_gate(Gate::Rx(PI), 0).unwrap()
            .simulate();

        let correct_register: [Complex<f64>; 4] = [
            complex_im!(-0.5f64), complex_im!(-0.5f64),
            complex_im!(-0.5f64), complex_im!(-0.5f64)
        ];

        compare_circuit(circuit, &correct_register);
    }

    #[test]
    fn ry_gate() {
        let mut circuit = Circuit::new(2).unwrap();

        circuit.add_gates(&[Gate::H, Gate::H]).unwrap()
            .add_gate(Gate::Ry(PI), 0).unwrap()
            .simulate();

        let correct_register: [Complex<f64>; 4] = [
            complex_re!(-0.5f64), complex_re!(-0.5f64),
            complex_re!(0.5f64), complex_re!(0.5f64)
        ];

        compare_circuit(circuit, &correct_register);
    }

    #[test]
    fn rz_gate() {
        let mut circuit = Circuit::new(2).unwrap();

        circuit.add_gates(&[Gate::H, Gate::H]).unwrap()
            .add_gate(Gate::Rz(PI), 0).unwrap()
            .simulate();

        let correct_register: [Complex<f64>; 4] = [
            complex_im!(-0.5f64), complex_im!(-0.5f64),
            complex_im!(0.5f64), complex_im!(0.5f64)
        ];

        compare_circuit(circuit, &correct_register);
    }

    #[test]
    fn global_gate() {
        let mut circuit = Circuit::new(2).unwrap();

        circuit.add_gates(&[Gate::H, Gate::H]).unwrap()
            .add_gate(Gate::Phase(PI), 0).unwrap()
            .simulate();

        let correct_register: [Complex<f64>; 4] = [
            complex_im!(0.5f64), complex_im!(0.5f64),
            complex_im!(0.5f64), complex_im!(0.5f64)
        ];

        compare_circuit(circuit, &correct_register);
    }

    #[test]
    fn x90_and_mx90_gate() {
        let mut circuit = Circuit::new(2).unwrap();

        circuit.add_gates(&[Gate::H, Gate::H]).unwrap()
            .add_gate(Gate::MX90, 0).unwrap()
            .add_gate(Gate::X90, 1).unwrap()
            .simulate();

        let correct_register: [Complex<f64>; 4] = [
            complex_re!(0.5f64), complex_re!(0.5f64),
            complex_re!(0.5f64), complex_re!(0.5f64)
        ];

        compare_circuit(circuit, &correct_register);
    }

    #[test]
    fn y90_and_my90_gate() {
        let mut circuit = Circuit::new(2).unwrap();

        circuit.add_gates(&[Gate::H, Gate::H]).unwrap()
            .add_gate(Gate::MY90, 0).unwrap()
            .add_gate(Gate::Y90, 1).unwrap()
            .simulate();

        let correct_register: [Complex<f64>; 4] = [
            complex_re!(-0.5f64), complex_re!(0.5f64),
            complex_re!(0.5f64), complex_re!(-0.5f64)
        ];

        compare_circuit(circuit, &correct_register);
    }

    #[test]
    fn cr_gate() {
        let mut circuit = Circuit::new(3).unwrap();

        circuit.add_gates(&[Gate::X, Gate::X, Gate::X]).unwrap()
            .add_gate(Gate::CR(-PI*0.5f64, 2), 1).unwrap()
            .simulate();

        let correct_register = [
            COMPLEX_ZERO, COMPLEX_ZERO, COMPLEX_ZERO, COMPLEX_ZERO,
            COMPLEX_ZERO, COMPLEX_ZERO, COMPLEX_ZERO, complex_im!(-1f64)
        ];
       
        compare_circuit(circuit, &correct_register);
    }

    #[test]
    fn crk_gate() {
        let mut circuit = Circuit::new(3).unwrap();

        circuit.add_gates(&[Gate::X, Gate::X, Gate::X]).unwrap()
            .add_gate(Gate::CRk(2i32, 2), 1).unwrap()
            .simulate();

        let correct_register = [
            COMPLEX_ZERO, COMPLEX_ZERO, COMPLEX_ZERO, COMPLEX_ZERO,
            COMPLEX_ZERO, COMPLEX_ZERO, COMPLEX_ZERO, complex_im!(1f64)
        ];
        
        compare_circuit(circuit, &correct_register);
    }

    #[test]
    fn custom_register() {
        let mut circuit = Circuit::new(3).unwrap();
        let register: SuperPosition = ProductState::new_unchecked(&[Qubit::One, Qubit::Zero, Qubit::One]).into();
        circuit.add_gate(Gate::X, 1).unwrap()
            .change_register(register).unwrap()
            .simulate();

        let correct_register = [
            COMPLEX_ZERO, COMPLEX_ZERO, COMPLEX_ZERO, COMPLEX_ZERO,
            COMPLEX_ZERO, COMPLEX_ZERO, COMPLEX_ZERO, complex_re!(1f64)
        ];
        
        compare_circuit(circuit, &correct_register);
    }

    #[test]
    #[should_panic]
    fn custom_register_wrong_dimension() {
        let mut circuit = Circuit::new(3).unwrap();
        let register: SuperPosition = ProductState::new_unchecked(&[Qubit::One, Qubit::Zero]).into();
        circuit.add_gate(Gate::X, 1).unwrap()
            .change_register(register).unwrap()
            .simulate();
    }
}
