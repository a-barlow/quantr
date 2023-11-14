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

use crate::Complex;
use core::panic;
use rand::Rng;
use std::collections::HashMap;
use std::ops::{Add, Mul};

use crate::states::{ProductState, Qubit, SuperPosition};
use crate::QuantrError;

pub mod printer;
mod standard_gate_ops;
pub mod states;

// The tolerance for declaring non-zero amplitudes.
const ZERO_MARGIN: f64 = 0.01;

// Maximum qubits for any circuit.
const CIRCUIT_MAX_QUBITS: usize = 50;

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
/// Matrix representations of these gates can be found at
/// <https://www.quantum-inspire.com/kbase/cqasm-qubit-gate-operations/>.
///
/// Currently, this enum has the `#[non_exhaustive]` as it's
/// yet undecided what will be included as a standard gate. This will
/// lessen the impact of breaking changes in the future.
#[derive(Clone, PartialEq, Debug)]
pub enum Gate<'a> {
    /// Identity.
    Id,
    /// Hadamard.
    H,
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
    /// Phase, rotation of +π/2 around the z-axis.
    S,
    /// Phase dagger, rotation of -π/2 around the z-axis.
    Sdag,
    /// T.
    T,
    /// T dagger.
    Tdag,
    /// Pauli-X.
    X,
    /// Pauli-Y.
    Y,
    /// Pauli-Z.
    Z,
    /// Controlled phase shift, with rotation and position of control node respectively.
    CR(f64, usize),
    /// Controlled phase shift for Quantum Fourier Transforms, with rotation and position of control node respectively.
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
    /// The arguments define the mapping of the gate; the position of the qubit states
    /// that the gate acts on; and a name that will be displayed in the printed diagram
    /// respectively. The name of the custom gate should be in ASCII for it to render properly
    /// when printing the circuit diagram.
    ///
    /// # Example
    /// ```
    /// use quantr::{Circuit, Gate};
    /// use quantr::states::{SuperPosition, ProductState, Qubit};
    /// use quantr::{Complex, complex_Re_array};
    ///
    /// // Defines a C-Not gate
    /// fn example_cnot(prod: ProductState) -> SuperPosition {
    ///    let input_register: [Qubit; 2] = [prod.qubits[0], prod.qubits[1]];
    ///    SuperPosition::new(2).set_amplitudes(match input_register {
    ///        [Qubit::Zero, Qubit::Zero] => &complex_Re_array!(1f64, 0f64, 0f64, 0f64),
    ///        [Qubit::Zero, Qubit::One]  => &complex_Re_array!(0f64, 1f64, 0f64, 0f64),
    ///        [Qubit::One, Qubit::Zero]  => &complex_Re_array!(0f64, 0f64, 0f64, 1f64),
    ///        [Qubit::One, Qubit::One]   => &complex_Re_array!(0f64, 0f64, 1f64, 0f64),
    ///    }).unwrap()
    /// }
    ///
    /// let mut quantum_circuit = Circuit::new(3).unwrap();
    /// quantum_circuit.add_gate(Gate::Custom(example_cnot, &[2], String::from("X")), 1).unwrap();
    ///
    /// // This is equivalent to
    /// quantum_circuit.add_gate(Gate::CNot(2), 1).unwrap();
    ///
    /// ```
    Custom(fn(ProductState) -> SuperPosition, &'a [usize], String),
}

impl<'a> Gate<'a> {
    // Retrieves the list of nodes within a gate.
    fn get_nodes(&self) -> Option<Vec<usize>> {
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
struct GateInfo<'a> {
    name: Gate<'a>,
    position: usize,
    size: GateSize,
}

/// A quantum circuit where gates can be appended and then simulated to measure resulting
/// superpositions.
pub struct Circuit<'a> {
    pub circuit_gates: Vec<Gate<'a>>,
    pub num_qubits: usize,
    output_state: Option<SuperPosition>,
    config_progress: bool,
}

impl<'a> Circuit<'a> {
    /// Initialises a new circuit.
    ///
    /// The target qubits used in defining custom functions must out live the slice of target
    /// qubits given to the custom function.
    ///
    /// # Example
    /// ```
    /// use quantr::Circuit;
    ///
    /// // Initialises a 3 qubit circuit.
    /// let quantum_circuit: Circuit = Circuit::new(3).unwrap();
    /// ```
    pub fn new(num_qubits: usize) -> Result<Circuit<'a>, QuantrError> {
        if num_qubits > CIRCUIT_MAX_QUBITS {
            return Err(QuantrError {
                message: String::from("The initialised circuit must have 50 or less qubits."),
            });
        } else if num_qubits == 0 {
            return Err(QuantrError {
                message: String::from("The initiliased circuit must have at least one wire."),
            });
        }

        let circuit_gates: Vec<Gate> = Vec::new();
        Ok(Circuit {
            circuit_gates,
            num_qubits,
            output_state: None,
            config_progress: false,
        })
    }

    /// Adds a single gate to the circuit.
    ///
    /// If wanting to add multiple gates, or a single gate repeatedly, across multiple wires, see
    /// [Circuit::add_repeating_gate] and [Circuit::add_gates_with_positions] respectively.
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
    pub fn add_gate(
        &mut self,
        gate: Gate<'a>,
        position: usize,
    ) -> Result<&mut Circuit<'a>, QuantrError> {
        Self::add_gates_with_positions(self, HashMap::from([(position, gate)]))
    }

    /// Add a column of gates based on their position on the wire.
    ///
    /// A HashMap is used to place gates onto their desired position; where the key is the position
    /// and the value is the [Gate]. This is similar to [Circuit::add_gate], however not
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
    ) -> Result<&mut Circuit<'a>, QuantrError> {
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
        Self::has_overlapping_controls_and_target(&gates_to_add, self.num_qubits.clone())?;

        // Push any multi-controlled gates to isolated columns
        Self::push_multi_gates(&mut gates_to_add);

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
    pub fn add_gates(
        &mut self,
        gates: &[Gate<'a>],
    ) -> Result<&mut Circuit<'a>, QuantrError> {
        // Ensured we have a gate for every wire.
        if gates.len() != self.num_qubits {
            return Err(QuantrError {
                message: format!("The number of gates, {}, does not match the number of wires, {}. All wires must have gates added.", gates.len(), self.num_qubits)
            });
        }

        // Make sure there are no control nodes that overlap with it's other nodes.
        Self::has_overlapping_controls_and_target(&gates, self.num_qubits.clone())?;

        // Push n-gates to another line (double, triple, etc.)
        let mut gates_vec: Vec<Gate<'a>> = gates.to_vec();
        Self::push_multi_gates(&mut gates_vec);
        self.circuit_gates.extend(gates_vec);

        Ok(self)
    }

    // Pushes multi-controlled gates into their own column. Potentially expensive operation to
    // insert new elements at smaller positions into a long vector.
    fn push_multi_gates(gates: &mut Vec<Gate<'a>>) {
        let mut extended_vec: Vec<Gate> = Default::default();
        let mut multi_gate_positions: Vec<usize> = Default::default();

        // if its a column with only a multi-control gate, leave it
        let mut found_multi: bool = false;
        let mut found_second: bool = false;
        for gate in gates.iter() {
            if let Gate::Custom(_, _, name) = gate {
                if !name.is_ascii() {
                    println!("\x1b[93m[Quantr Warning] The custom function name, {}, does not only use ASCII chars. This could lead to problems in printing the circuit diagram. This warning will be promoted to an Error in the next major release.\x1b[0m", name);
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
                match Self::classify_gate_size(gate) {
                    GateSize::Double | GateSize::Triple | GateSize::Custom => {
                        let mut temp_vec = vec![Gate::Id; gates.len()];
                        temp_vec[pos] = gate.clone();
                        extended_vec.extend(temp_vec);
                        multi_gate_positions.push(pos);
                    }
                    _ => {}
                }
            }

            for i in multi_gate_positions {
                gates[i] = Gate::Id;
            }
            gates.extend(extended_vec);
        }
    }

    // need to implement all other gates, in addition to checking that it's within circuit size!
    fn has_overlapping_controls_and_target(gates: &[Gate], circuit_size: usize) -> Result<(), QuantrError> {
        for (pos, gate) in gates.iter().enumerate() {
            match gate.get_nodes() {
                Some(nodes) => { // check for overlapping control nodes.
                    if Self::contains_repeating_values(&nodes) {
                        return Err(QuantrError { message: format!("The gate, {:?}, has overlapping control nodes.", gate) });
                    }
                    if nodes.contains(&pos) {
                        return Err(QuantrError { message: format!("The gate, {:?}, has its position overlapping with a control node at position {}.", gate, pos) });
                    }
                    for &node in nodes.iter() {
                        if node >= circuit_size {
                            return Err(QuantrError { message: format!("The control node at position {:?}, is greater than the umnber of qubits {}.", node, circuit_size) });
                        }
                    }
                },
                None => {}
            } 
        }

        Ok(())
    }

    // Find if there are any repating values in array, O(n)
    // The initialisation of the circuit guarantees the max circuit size.
    fn contains_repeating_values(array: &[usize]) -> bool {
        let mut counter: [bool; CIRCUIT_MAX_QUBITS] = [false; CIRCUIT_MAX_QUBITS];
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
    /// The top of the wire is in the 0th position. For adding multiple gates that are different,
    /// please refer to [Circuit::add_gates] and [Circuit::add_gates_with_positions].
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
    ) -> Result<&mut Circuit<'a>, QuantrError> {
        // Incase the user has attempted to place the gate twice on the same wire.
        if Self::contains_repeating_values(positions) {
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
    /// if let NonObservable(super_pos) = circuit.get_superposition().unwrap() {
    ///     for (state, amplitude) in super_pos.into_iter() {
    ///         println!("|{}>   : {}", state.to_string(), amplitude);
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
    /// if let Observable(bin_count) = circuit.repeat_measurement(500).unwrap() {
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
    ) -> Result<Measurement<HashMap<ProductState, usize>>, QuantrError> {
        match &self.output_state {
            Some(super_position) => {
                // Peform bin count of states
                let mut probabilities: HashMap<ProductState, f64> = Default::default();
                for (key, value) in super_position.to_hash_map() {
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

            if *gate == Gate::Id {
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

    /// Attaches a custom register to the circuit resulting in a superposition that can be measured.
    ///
    /// See [Circuit::get_superposition] and [Circuit::repeat_measurement] for details on obtaining
    /// observables from the resulting superposition.
    ///
    /// # Example
    /// ```
    /// use quantr::{Circuit, Gate};
    /// use quantr::states::{Qubit, ProductState, SuperPosition};
    ///
    /// let mut circuit = Circuit::new(2).unwrap();
    /// circuit.add_gate(Gate::X, 1).unwrap();
    ///
    /// let register: SuperPosition = ProductState::new(&[Qubit::One, Qubit::Zero]).into_super_position();
    ///
    /// circuit.simulate_with_register(register);
    ///
    /// // Simulates the circuit:
    /// // |1> -------
    /// // |0> -- X --
    /// ````
    pub fn simulate_with_register(
        &mut self,
        mut register: SuperPosition,
    ) -> Result<(), QuantrError> {
        if register.product_dim != self.num_qubits {
            return Err(QuantrError {
                message: format!("The custom register has a product state dimension of {}, while the number of qubits is {}. These must equal each other.", register.product_dim, self.num_qubits)
            });
        }

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

            if *gate == Gate::Id {
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
        Ok(())
    }

    // If the user toggles the log on, then prints the simulation of each circuit.
    fn print_circuit_log(
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
            Gate::Toffoli(control1, control2) => {
                (standard_gate_ops::toffoli, control1, control2)
            }
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

    /// Toggles if the circuit should print the progress of simulating each gate.
    ///
    /// It will only show the application of non-identity gates. The toggle is set to `false`
    /// for a new quantum circuit.
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
    use crate::{complex_Im, complex_Re, complex_Re_array, COMPLEX_ZERO, complex};
    use std::f64::consts::{FRAC_1_SQRT_2, PI};
    use crate::Measurement::NonObservable;
    use super::*;
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

    fn example_cnot(prod: ProductState) -> SuperPosition {
        let input_register: [Qubit; 2] = [prod.qubits[0], prod.qubits[1]];
        SuperPosition::new(2).set_amplitudes(match input_register {
            [Qubit::Zero, Qubit::Zero] => &complex_Re_array!(1f64, 0f64, 0f64, 0f64),
            [Qubit::Zero, Qubit::One]  => &complex_Re_array!(0f64, 1f64, 0f64, 0f64),
            [Qubit::One, Qubit::Zero]  => &complex_Re_array!(0f64, 0f64, 0f64, 1f64),
            [Qubit::One, Qubit::One]   => &complex_Re_array!(0f64, 0f64, 1f64, 0f64),
        }).unwrap()
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
            complex_Re!(0.5f64), complex_Im!(-0.5f64),
            complex_Im!(0.5f64), complex_Re!(0.5f64)];
        compare_circuit(circuit, &correct_register);
    }

    #[test]
    fn t_and_conjugate_gates() {
        let mut circuit = Circuit::new(2).unwrap();
        circuit.add_gates(&[Gate::H, Gate::H]).unwrap()
               .add_gates(&[Gate::T, Gate::Tdag]).unwrap()
               .simulate();

        let correct_register: [Complex<f64>; 4] = [
            complex_Re!(0.5f64), complex!(0.5f64*FRAC_1_SQRT_2, -0.5f64*FRAC_1_SQRT_2),
            complex!(0.5f64*FRAC_1_SQRT_2, 0.5f64*FRAC_1_SQRT_2), complex_Re!(0.5f64)];
        compare_circuit(circuit, &correct_register);
    }


    #[test]
    fn custom_gates() {
        let mut quantum_circuit = Circuit::new(3).unwrap();
        quantum_circuit.add_gate(Gate::H, 2).unwrap()
            .add_gate(Gate::Custom(example_cnot, &[2], String::from("cNot")), 1).unwrap()
            .simulate();
        
        let correct_register: [Complex<f64>; 8] = [
            complex_Re!(FRAC_1_SQRT_2), COMPLEX_ZERO,
            COMPLEX_ZERO, complex_Re!(FRAC_1_SQRT_2),
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
            complex_Im!(-FRAC_1_SQRT_2), COMPLEX_ZERO, COMPLEX_ZERO, COMPLEX_ZERO,
            COMPLEX_ZERO, complex_Im!(FRAC_1_SQRT_2), COMPLEX_ZERO, COMPLEX_ZERO
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
            COMPLEX_ZERO, complex_Im!(FRAC_1_SQRT_2), COMPLEX_ZERO,complex_Im!(FRAC_1_SQRT_2),
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
            complex_Re!(FRAC_1_SQRT_2), complex_Re!(FRAC_1_SQRT_2),
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
            complex_Re!(0.5f64), complex_Re!(0.5f64),
            complex_Re!(0.5f64), complex_Re!(0.5f64)];
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
            COMPLEX_ZERO, complex_Re!(FRAC_1_SQRT_2), COMPLEX_ZERO, complex_Re!(FRAC_1_SQRT_2),
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
            complex_Re!(1f64), COMPLEX_ZERO, COMPLEX_ZERO, COMPLEX_ZERO
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
            COMPLEX_ZERO, COMPLEX_ZERO, complex_Re!(-1f64), COMPLEX_ZERO,
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
            complex_Re!(FRAC_1_SQRT_2), COMPLEX_ZERO,
            complex_Re!(FRAC_1_SQRT_2), COMPLEX_ZERO
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
            complex_Re!(FRAC_1_SQRT_2), COMPLEX_ZERO,
            COMPLEX_ZERO, complex_Re!(FRAC_1_SQRT_2)
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
            COMPLEX_ZERO, complex_Im!(-FRAC_1_SQRT_2), COMPLEX_ZERO, COMPLEX_ZERO,
            complex_Im!(FRAC_1_SQRT_2), COMPLEX_ZERO, COMPLEX_ZERO, COMPLEX_ZERO,
            COMPLEX_ZERO, COMPLEX_ZERO, COMPLEX_ZERO, COMPLEX_ZERO,
            COMPLEX_ZERO, COMPLEX_ZERO, COMPLEX_ZERO, COMPLEX_ZERO
        ];

        compare_circuit(circuit, &correct_register);

    }
    
    #[test]
    fn custom_non_ascii_name() {
        let mut circuit = Circuit::new(3).unwrap();

        circuit.add_gate(Gate::Custom(example_cnot, &[0], "NonAscii†".to_string()), 1).unwrap();
        // in future, this should panic. For now, this is a warning message.
    }

    #[test]
    fn rx_gate() {
        let mut circuit = Circuit::new(2).unwrap();

        circuit.add_gates(&[Gate::H, Gate::H]).unwrap()
            .add_gate(Gate::Rx(PI), 0).unwrap()
            .simulate();

        let correct_register: [Complex<f64>; 4] = [
            complex_Im!(-0.5f64), complex_Im!(-0.5f64),
            complex_Im!(-0.5f64), complex_Im!(-0.5f64)
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
            complex_Re!(-0.5f64), complex_Re!(-0.5f64),
            complex_Re!(0.5f64), complex_Re!(0.5f64)
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
            complex_Im!(-0.5f64), complex_Im!(-0.5f64),
            complex_Im!(0.5f64), complex_Im!(0.5f64)
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
            complex_Im!(0.5f64), complex_Im!(0.5f64),
            complex_Im!(0.5f64), complex_Im!(0.5f64)
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
            complex_Re!(0.5f64), complex_Re!(0.5f64),
            complex_Re!(0.5f64), complex_Re!(0.5f64)
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
            complex_Re!(-0.5f64), complex_Re!(0.5f64),
            complex_Re!(0.5f64), complex_Re!(-0.5f64)
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
            COMPLEX_ZERO, COMPLEX_ZERO, COMPLEX_ZERO, complex_Im!(-1f64)
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
            COMPLEX_ZERO, COMPLEX_ZERO, COMPLEX_ZERO, complex_Im!(1f64)
        ];
        
        compare_circuit(circuit, &correct_register);
    }

    #[test]
    fn custom_register() {
        let mut circuit = Circuit::new(3).unwrap();
        let register: SuperPosition = ProductState::new(&[Qubit::One, Qubit::Zero, Qubit::One]).into_super_position();
        circuit.add_gate(Gate::X, 1).unwrap()
            .simulate_with_register(register).unwrap();

        let correct_register = [
            COMPLEX_ZERO, COMPLEX_ZERO, COMPLEX_ZERO, COMPLEX_ZERO,
            COMPLEX_ZERO, COMPLEX_ZERO, COMPLEX_ZERO, complex_Re!(1f64)
        ];
        
        compare_circuit(circuit, &correct_register);
    }

    #[test]
    #[should_panic]
    fn custom_register_wrong_dimension() {
        let mut circuit = Circuit::new(3).unwrap();
        let register: SuperPosition = ProductState::new(&[Qubit::One, Qubit::Zero]).into_super_position();
        circuit.add_gate(Gate::X, 1).unwrap()
            .simulate_with_register(register).unwrap();
    }
}
