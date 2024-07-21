/*
* Copyright (c) 2024 Andrew Rowan Barlow. Licensed under the EUPL-1.2
* or later. You may obtain a copy of the licence at
* https://joinup.ec.europa.eu/collection/eupl/eupl-text-eupl-12. A copy
* of the EUPL-1.2 licence in English is given in LICENCE.txt which is
* found in the root directory of this repository.
*
* Author: Andrew Rowan Barlow <a.barlow.dev@gmail.com>
*/

use super::circuit::gate::GateInfo;
use crate::error::QuantrError;
use crate::states::SuperPosition;
use crate::{Gate, SimulatedCircuit};
use std::collections::HashMap;

pub mod gate;
pub mod measurement;
pub mod printer;
mod simulation;
mod standard_gate_ops;
pub mod states;

pub(crate) type QResult<T> = Result<T, QuantrError>;

/// A quantum circuit where gates can be appended and then simulated to measure resulting
/// superpositions.
pub struct Circuit {
    pub(crate) circuit_gates: Vec<Gate>,
    pub(crate) num_qubits: usize,
    pub(crate) register: Option<SuperPosition>,
    pub(crate) config_progress: bool,
}

// The tolerance for declaring non-zero amplitudes.
const ZERO_MARGIN: f64 = 1e-7;
impl Circuit {
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
    pub fn new(num_qubits: usize) -> QResult<Circuit> {
        if num_qubits == 0 {
            return Err(QuantrError {
                message: String::from("The initialised circuit must have at least one wire."),
            });
        }

        let circuit_gates: Vec<Gate> = Vec::new();
        Ok(Circuit {
            circuit_gates,
            num_qubits,
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
    pub fn get_num_qubits(&self) -> usize {
        self.num_qubits
    }

    /// Returns if the circuit will print explicit simulatin output set
    /// by [Circuit::toggle_simulation_progress].
    ///
    /// # Example
    /// ```
    /// use quantr::{Circuit};
    ///
    /// let mut quantum_circuit = Circuit::new(2).unwrap();
    /// quantum_circuit.set_print_progress(true);
    /// ```
    pub fn set_print_progress(&mut self, progress: bool) {
        self.config_progress = progress;
    }

    /// Returns the slice of gates that have been added to the circuit.
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
    pub fn get_gates(&self) -> &[Gate] {
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
    pub fn add_gate(&mut self, gate: Gate, position: usize) -> QResult<&mut Circuit> {
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
        gates_with_positions: HashMap<usize, Gate>,
    ) -> QResult<&mut Circuit> {
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
    pub fn add_gates(&mut self, gates: &[Gate]) -> QResult<&mut Circuit> {
        // Ensured we have a gate for every wire.
        if gates.len() != self.num_qubits {
            return Err(QuantrError {
                message: format!("The number of gates, {}, does not match the number of wires, {}. All wires must have gates added.", gates.len(), self.num_qubits)
            });
        }

        // Make sure there are no control nodes that overlap with it's other nodes.
        Self::has_overlapping_controls_and_target(gates, self.num_qubits)?;

        // Push n-gates to another line (double, triple, etc.)
        let mut gates_vec: Vec<Gate> = gates.to_vec();
        Self::push_multi_gates(&mut gates_vec)?;
        self.circuit_gates.extend(gates_vec);
        Ok(self)
    }

    // Pushes multi-controlled gates into their own column. Potentially expensive operation to
    // insert new elements at smaller positions into a long vector.
    fn push_multi_gates(gates: &mut Vec<Gate>) -> QResult<()> {
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
    pub fn add_repeating_gate(&mut self, gate: Gate, positions: &[usize]) -> QResult<&mut Circuit> {
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
        let mut gates: Vec<Gate> = vec![Gate::Id; self.num_qubits];
        for &pos in positions {
            gates[pos] = gate.clone();
        }
        self.add_gates(gates.as_slice())
    }

    /// Attaches the register, |0...0>, to the circuit resulting in a superposition that can be measured.
    ///
    /// See [SimulatedCircuit::get_state] and [SimulatedCircuit::measure_all] for details on obtaining
    /// observables from the resulting superposition.
    ///
    /// If you are not wanting the circuit to be consumed, please refer to [Circuit::clone_and_simulate].
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
    /// // |0> -------
    /// // |0> -- H --
    /// ````
    pub fn simulate(mut self) -> SimulatedCircuit {
        match self.register.take() {
            Some(mut prepared_register) => {
                self.simulate_with_register(&mut prepared_register);
                SimulatedCircuit {
                    circuit_gates: self.circuit_gates,
                    num_qubits: self.num_qubits,
                    register: prepared_register,
                    config_progress: self.config_progress,
                    disable_warnings: false,
                }
            }
            None => {
                let mut zero_register = SuperPosition::new_unchecked(self.num_qubits);
                self.simulate_with_register(&mut zero_register);
                SimulatedCircuit {
                    circuit_gates: self.circuit_gates,
                    num_qubits: self.num_qubits,
                    register: zero_register,
                    config_progress: self.config_progress,
                    disable_warnings: false,
                }
            }
        }
    }

    /// Attaches the register, |0...0>, to the circuit resulting in a superposition that can be measured,
    /// and will clone the contents of the register. This will duplicate the register, and so could
    /// lead to large memeory consumption for circuits with many qubits.
    ///
    /// See [SimulatedCircuit::get_state] and [SimulatedCircuit::measure_all] for details on obtaining
    /// observables from the resulting superposition.
    ///
    /// If you are wanting the circuit to be consumed, please refer to [Circuit::simulate].
    ///
    /// # Example
    /// ```
    /// use quantr::{Circuit, Gate};
    ///
    /// let mut circuit = Circuit::new(3).unwrap();
    /// circuit.add_gate(Gate::H, 2).unwrap();
    ///
    /// let simulated_with_H = circuit.clone_and_simulate();
    ///
    /// // Below would be impossible if Circuit::simulate was used instead
    /// let simulated_with_H_and_X = circuit.add_gate(Gate::X, 1);
    /// ````
    pub fn clone_and_simulate(&self) -> SimulatedCircuit {
        match self.register.clone() {
            Some(mut prepared_register) => {
                self.simulate_with_register(&mut prepared_register);
                SimulatedCircuit {
                    circuit_gates: self.circuit_gates.clone(),
                    num_qubits: self.num_qubits,
                    register: prepared_register,
                    config_progress: self.config_progress,
                    disable_warnings: false,
                }
            }
            None => {
                let mut zero_register = SuperPosition::new_unchecked(self.num_qubits);
                self.simulate_with_register(&mut zero_register);
                SimulatedCircuit {
                    circuit_gates: self.circuit_gates.clone(),
                    num_qubits: self.num_qubits,
                    register: zero_register,
                    config_progress: self.config_progress,
                    disable_warnings: false,
                }
            }
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
    pub fn change_register(&mut self, super_pos: SuperPosition) -> QResult<&mut Circuit> {
        if super_pos.product_dim != self.num_qubits {
            return Err(QuantrError {
                message: format!("The custom register has a product state dimension of {}, while the number of qubits is {}. These must equal each other.", super_pos.product_dim, self.num_qubits)
            });
        }

        self.register = Some(super_pos);

        Ok(self)
    }
}

#[rustfmt::skip]
#[cfg(test)]
mod tests {
    use crate::{complex_im, complex_re, complex_re_array, Circuit, Gate};
    use num_complex::{Complex64, c64};
    use crate::states::{SuperPosition, Qubit, ProductState};
    use super::HashMap;
    use std::f64::consts::{FRAC_1_SQRT_2, PI};
    use crate::Measurement::NonObservable;
    const ERROR_MARGIN: f64 = 0.000001f64; // For comparing floats due to floating point error.
    // Needed for testing
    fn compare_complex_lists_and_register(correct_list: &[Complex64], register: &SuperPosition) {
        for (i, &comp_num) in register.amplitudes.iter().enumerate() { // Make sure that it turns up complex
            assert!(equal_within_error(comp_num.re, correct_list[i].re));
            assert!(equal_within_error(comp_num.im, correct_list[i].im));
        }
    }

    fn equal_within_error(num: f64, compare_num: f64) -> bool {
        num < compare_num + ERROR_MARGIN && num > compare_num - ERROR_MARGIN
    }

    fn compare_circuit(circuit: Circuit, correct_register: &[Complex64]) {
        if let NonObservable(measured_register) = circuit.simulate().get_state() {
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
            .add_gates(&[Gate::Id, Gate::Custom(example_cnot, vec!(1), "X".to_string()), Gate::Id])
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
            .add_gates(&[Gate::S, Gate::Sdag]).unwrap();

        let correct_register: [Complex64; 4] = [
            complex_re!(0.5f64), complex_im!(-0.5f64),
            complex_im!(0.5f64), complex_re!(0.5f64)];
        compare_circuit(circuit, &correct_register);
    }

    #[test]
    fn t_and_conjugate_gates() {
        let mut circuit = Circuit::new(2).unwrap();
        circuit.add_gates(&[Gate::H, Gate::H]).unwrap()
               .add_gates(&[Gate::T, Gate::Tdag]).unwrap();

        let correct_register: [Complex64; 4] = [
            complex_re!(0.5f64), c64(0.5f64*FRAC_1_SQRT_2, -0.5f64*FRAC_1_SQRT_2),
            c64(0.5f64*FRAC_1_SQRT_2, 0.5f64*FRAC_1_SQRT_2), complex_re!(0.5f64)];
        compare_circuit(circuit, &correct_register);
    }


    #[test]
    fn custom_gates() {
        let mut quantum_circuit = Circuit::new(3).unwrap();
        quantum_circuit.add_gate(Gate::H, 2).unwrap()
            .add_gate(Gate::Custom(example_cnot, vec!(2), String::from("cNot")), 1).unwrap();

        let correct_register: [Complex64; 8] = [
            complex_re!(FRAC_1_SQRT_2), num_complex::Complex64::ZERO,
            num_complex::Complex64::ZERO, complex_re!(FRAC_1_SQRT_2),
            num_complex::Complex64::ZERO, num_complex::Complex64::ZERO,
            num_complex::Complex64::ZERO, num_complex::Complex64::ZERO];

        compare_circuit(quantum_circuit, &correct_register);
    }

    #[test]
    fn toffoli_gates() {
        let mut quantum_circuit = Circuit::new(4).unwrap();
        quantum_circuit.add_gate(Gate::X, 0).unwrap()
            .add_gate(Gate::H, 3).unwrap()
            .add_gate(Gate::Y, 3).unwrap()
            .add_gate(Gate::Toffoli(3, 0), 1).unwrap();

        let correct_register = [
            num_complex::Complex64::ZERO, num_complex::Complex64::ZERO, num_complex::Complex64::ZERO, num_complex::Complex64::ZERO,
            num_complex::Complex64::ZERO, num_complex::Complex64::ZERO, num_complex::Complex64::ZERO, num_complex::Complex64::ZERO,
            complex_im!(-FRAC_1_SQRT_2), num_complex::Complex64::ZERO, num_complex::Complex64::ZERO, num_complex::Complex64::ZERO,
            num_complex::Complex64::ZERO, complex_im!(FRAC_1_SQRT_2), num_complex::Complex64::ZERO, num_complex::Complex64::ZERO
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
            .add_gates(&[Gate::Z, Gate::Y, Gate::H, Gate::X]).unwrap();

        let correct_register = [
            num_complex::Complex64::ZERO, num_complex::Complex64::ZERO, num_complex::Complex64::ZERO, num_complex::Complex64::ZERO,
            num_complex::Complex64::ZERO, complex_im!(FRAC_1_SQRT_2), num_complex::Complex64::ZERO, complex_im!(FRAC_1_SQRT_2),
            num_complex::Complex64::ZERO, num_complex::Complex64::ZERO, num_complex::Complex64::ZERO, num_complex::Complex64::ZERO,
            num_complex::Complex64::ZERO, num_complex::Complex64::ZERO, num_complex::Complex64::ZERO, num_complex::Complex64::ZERO
        ];
        compare_circuit(circuit, &correct_register);
    }

    #[test]
    fn hash_map_with_two_gates() {
        let mut circuit = Circuit::new(3).unwrap();
        circuit.add_gates_with_positions(HashMap::from([(0, Gate::X), (2, Gate::H)])).unwrap();
        let correct_register: [Complex64; 8] = [
            num_complex::Complex64::ZERO, num_complex::Complex64::ZERO,
            num_complex::Complex64::ZERO, num_complex::Complex64::ZERO,
            complex_re!(FRAC_1_SQRT_2), complex_re!(FRAC_1_SQRT_2),
            num_complex::Complex64::ZERO, num_complex::Complex64::ZERO];
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
        circuit.add_gates(&[Gate::H, Gate::H]).unwrap();

        let correct_register: [Complex64; 4] = [
            complex_re!(0.5f64), complex_re!(0.5f64),
            complex_re!(0.5f64), complex_re!(0.5f64)];
        compare_circuit(circuit, &correct_register);
    }

    #[test]
    fn add_two_rows_single_gates() {
        let mut circuit = Circuit::new(4).unwrap();

        circuit.add_gates_with_positions(HashMap::from([(0, Gate::X)])).unwrap()
                .add_gates_with_positions(HashMap::from([(3, Gate::X), (2, Gate::H)])).unwrap();

        let correct_register = [
            num_complex::Complex64::ZERO, num_complex::Complex64::ZERO, num_complex::Complex64::ZERO, num_complex::Complex64::ZERO,
            num_complex::Complex64::ZERO, num_complex::Complex64::ZERO, num_complex::Complex64::ZERO, num_complex::Complex64::ZERO,
            num_complex::Complex64::ZERO, complex_re!(FRAC_1_SQRT_2), num_complex::Complex64::ZERO, complex_re!(FRAC_1_SQRT_2),
            num_complex::Complex64::ZERO, num_complex::Complex64::ZERO, num_complex::Complex64::ZERO, num_complex::Complex64::ZERO
        ];
        
        compare_circuit(circuit, &correct_register);
    }

    #[test]
    fn cy_and_swap_gates_work() {
        let mut circuit = Circuit::new(4).unwrap();

        circuit.add_repeating_gate(Gate::X, &[1,2]).unwrap()
            .add_gate(Gate::CY(2), 0).unwrap()
            .add_gate(Gate::Swap(3), 2).unwrap()
            .add_gate(Gate::CY(0), 3).unwrap();

        let correct_register = [
            num_complex::Complex64::ZERO, num_complex::Complex64::ZERO, num_complex::Complex64::ZERO, num_complex::Complex64::ZERO,
            num_complex::Complex64::ZERO, num_complex::Complex64::ZERO, num_complex::Complex64::ZERO, num_complex::Complex64::ZERO,
            num_complex::Complex64::ZERO, num_complex::Complex64::ZERO, num_complex::Complex64::ZERO, num_complex::Complex64::ZERO,
            complex_re!(1f64), num_complex::Complex64::ZERO, num_complex::Complex64::ZERO, num_complex::Complex64::ZERO
        ];
        
        compare_circuit(circuit, &correct_register);

    }

    #[test]
    fn cz_and_swap_gates_work() {
        let mut circuit = Circuit::new(3).unwrap();

        circuit.add_repeating_gate(Gate::X, &[0,2]).unwrap()
            .add_gate(Gate::Swap(1), 2).unwrap()
            .add_gate(Gate::CZ(1), 0).unwrap();

        let correct_register = [
            num_complex::Complex64::ZERO, num_complex::Complex64::ZERO, num_complex::Complex64::ZERO, num_complex::Complex64::ZERO,
            num_complex::Complex64::ZERO, num_complex::Complex64::ZERO, complex_re!(-1f64), num_complex::Complex64::ZERO,
            num_complex::Complex64::ZERO, num_complex::Complex64::ZERO, num_complex::Complex64::ZERO, num_complex::Complex64::ZERO,
            num_complex::Complex64::ZERO, num_complex::Complex64::ZERO, num_complex::Complex64::ZERO, num_complex::Complex64::ZERO
        ];
        
        compare_circuit(circuit, &correct_register);
    }

    #[test]
    fn cnot_gate_simply_use_works() {
        let mut circuit = Circuit::new(2).unwrap();

        circuit.add_gate(Gate::H, 0).unwrap()
            .add_gate(Gate::CNot(1), 0).unwrap();

        let correct_register: [Complex64; 4] = [
            complex_re!(FRAC_1_SQRT_2), num_complex::Complex64::ZERO,
            complex_re!(FRAC_1_SQRT_2), num_complex::Complex64::ZERO
        ];
        
        compare_circuit(circuit, &correct_register);

    }

    #[test]
    fn cnot_gate_simply_flipped() {
        let mut circuit = Circuit::new(2).unwrap();

        circuit.add_gate(Gate::H, 0).unwrap()
            .add_gate(Gate::CNot(0), 1).unwrap();

        let correct_register: [Complex64; 4] = [
            complex_re!(FRAC_1_SQRT_2), num_complex::Complex64::ZERO,
            num_complex::Complex64::ZERO, complex_re!(FRAC_1_SQRT_2)
        ];

        compare_circuit(circuit, &correct_register);

    }

    #[test]
    fn cnot_gate_extended_control_works_asymmetric() {
        let mut circuit = Circuit::new(4).unwrap();

        circuit.add_gate(Gate::H, 1).unwrap()
            .add_gate(Gate::CNot(1), 3).unwrap()
            .add_gate(Gate::Y, 1).unwrap();

        let correct_register = [
            num_complex::Complex64::ZERO, complex_im!(-FRAC_1_SQRT_2), num_complex::Complex64::ZERO, num_complex::Complex64::ZERO,
            complex_im!(FRAC_1_SQRT_2), num_complex::Complex64::ZERO, num_complex::Complex64::ZERO, num_complex::Complex64::ZERO,
            num_complex::Complex64::ZERO, num_complex::Complex64::ZERO, num_complex::Complex64::ZERO, num_complex::Complex64::ZERO,
            num_complex::Complex64::ZERO, num_complex::Complex64::ZERO, num_complex::Complex64::ZERO, num_complex::Complex64::ZERO
        ];

        compare_circuit(circuit, &correct_register);

    }
    
    #[test]
    #[should_panic]
    fn custom_non_ascii_name() {
        let mut circuit = Circuit::new(3).unwrap();

        circuit.add_gate(Gate::Custom(example_cnot, vec!(0), "NonAscii†".to_string()), 1).unwrap();
    }

    #[test]
    fn rx_gate() {
        let mut circuit = Circuit::new(2).unwrap();

        circuit.add_gates(&[Gate::H, Gate::H]).unwrap()
            .add_gate(Gate::Rx(PI), 0).unwrap();

        let correct_register: [Complex64; 4] = [
            complex_im!(-0.5f64), complex_im!(-0.5f64),
            complex_im!(-0.5f64), complex_im!(-0.5f64)
        ];

        compare_circuit(circuit, &correct_register);
    }

    #[test]
    fn ry_gate() {
        let mut circuit = Circuit::new(2).unwrap();

        circuit.add_gates(&[Gate::H, Gate::H]).unwrap()
            .add_gate(Gate::Ry(PI), 0).unwrap();

        let correct_register: [Complex64; 4] = [
            complex_re!(-0.5f64), complex_re!(-0.5f64),
            complex_re!(0.5f64), complex_re!(0.5f64)
        ];

        compare_circuit(circuit, &correct_register);
    }

    #[test]
    fn rz_gate() {
        let mut circuit = Circuit::new(2).unwrap();

        circuit.add_gates(&[Gate::H, Gate::H]).unwrap()
            .add_gate(Gate::Rz(PI), 0).unwrap();

        let correct_register: [Complex64; 4] = [
            complex_im!(-0.5f64), complex_im!(-0.5f64),
            complex_im!(0.5f64), complex_im!(0.5f64)
        ];

        compare_circuit(circuit, &correct_register);
    }

    #[test]
    fn global_gate() {
        let mut circuit = Circuit::new(2).unwrap();

        circuit.add_gates(&[Gate::H, Gate::H]).unwrap()
            .add_gate(Gate::Phase(PI), 0).unwrap();

        let correct_register: [Complex64; 4] = [
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
            .add_gate(Gate::X90, 1).unwrap();

        let correct_register: [Complex64; 4] = [
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
            .add_gate(Gate::Y90, 1).unwrap();

        let correct_register: [Complex64; 4] = [
            complex_re!(-0.5f64), complex_re!(0.5f64),
            complex_re!(0.5f64), complex_re!(-0.5f64)
        ];

        compare_circuit(circuit, &correct_register);
    }

    #[test]
    fn cr_gate() {
        let mut circuit = Circuit::new(3).unwrap();

        circuit.add_gates(&[Gate::X, Gate::X, Gate::X]).unwrap()
            .add_gate(Gate::CR(-PI*0.5f64, 2), 1).unwrap();

        let correct_register = [
            num_complex::Complex64::ZERO, num_complex::Complex64::ZERO, num_complex::Complex64::ZERO, num_complex::Complex64::ZERO,
            num_complex::Complex64::ZERO, num_complex::Complex64::ZERO, num_complex::Complex64::ZERO, complex_im!(-1f64)
        ];
       
        compare_circuit(circuit, &correct_register);
    }

    #[test]
    fn crk_gate() {
        let mut circuit = Circuit::new(3).unwrap();

        circuit.add_gates(&[Gate::X, Gate::X, Gate::X]).unwrap()
            .add_gate(Gate::CRk(2i32, 2), 1).unwrap();

        let correct_register = [
            num_complex::Complex64::ZERO, num_complex::Complex64::ZERO, num_complex::Complex64::ZERO, num_complex::Complex64::ZERO,
            num_complex::Complex64::ZERO, num_complex::Complex64::ZERO, num_complex::Complex64::ZERO, complex_im!(1f64)
        ];
        
        compare_circuit(circuit, &correct_register);
    }

    #[test]
    fn custom_register() {
        let mut circuit = Circuit::new(3).unwrap();
        let register: SuperPosition = ProductState::new_unchecked(&[Qubit::One, Qubit::Zero, Qubit::One]).into();
        circuit.add_gate(Gate::X, 1).unwrap()
            .change_register(register).unwrap();

        let correct_register = [
            num_complex::Complex64::ZERO, num_complex::Complex64::ZERO, num_complex::Complex64::ZERO, num_complex::Complex64::ZERO,
            num_complex::Complex64::ZERO, num_complex::Complex64::ZERO, num_complex::Complex64::ZERO, complex_re!(1f64)
        ];
        
        compare_circuit(circuit, &correct_register);
    }

    #[test]
    #[should_panic]
    fn custom_register_wrong_dimension() {
        let mut circuit = Circuit::new(3).unwrap();
        let register: SuperPosition = ProductState::new_unchecked(&[Qubit::One, Qubit::Zero]).into();
        circuit.add_gate(Gate::X, 1).unwrap()
            .change_register(register).unwrap();
    }
}
