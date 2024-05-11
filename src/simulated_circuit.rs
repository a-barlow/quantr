/*
* Copyright (c) 2024 Andrew Rowan Barlow. Licensed under the EUPL-1.2
* or later. You may obtain a copy of the licence at
* https://joinup.ec.europa.eu/collection/eupl/eupl-text-eupl-12. A copy
* of the EUPL-1.2 licence in English is given in LICENCE.txt which is
* found in the root directory of this repository.
*
* Author: Andrew Rowan Barlow <a.barlow.dev@gmail.com>
*/

use crate::circuit::QResult;
use crate::Gate;
use crate::{
    circuit::Circuit,
    states::{ProductState, SuperPosition},
    Measurement, QuantrError,
};
use std::collections::HashMap;

pub struct SimulatedCircuit {
    pub(crate) circuit: Circuit,
    pub(crate) partially_simulated: bool,
}

impl SimulatedCircuit {
    pub fn get_circuit(&self) -> &Circuit {
        &self.circuit
    }

    pub fn get_circuit_gates(&self) -> &[Gate] {
        self.circuit.get_gates()
    }

    pub fn get_partial_status(&self) -> bool {
        self.partially_simulated
    }

    pub fn get_toggle_progress(&self) -> bool {
        self.circuit.get_toggle_progress()
    }

    pub fn toggle_simulation_progress(&mut self) {
        self.circuit.toggle_simulation_progress()
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
    /// let simulated_circuit = circuit.simulate();
    ///
    /// println!("State | Amplitude of State");
    /// if let Ok(NonObservable(super_pos)) = simulated_circuit.get_superposition() {
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
        // TODO change name; and output type as None reflects partially simulated
        match &self.circuit.output_state {
            Some(super_position) => Ok(Measurement::NonObservable(super_position)),
            None => {
                Err(QuantrError{
                    message: String::from("The circuit has not been simulated. Call Circuit::simulate before calling this method, Circuit::get_superposition."),
                })
            }
        }
    }

    /// Returns a `HashMap` that contains the number of times the corresponding state was observed over
    /// `n` measurements of the superpositions (shots).
    ///
    /// Explicitly, this performs repeated measurements where a register is attached to the circuit,
    /// the resulting superposition measured in the computational basis, and then the reduced state
    /// recorded. If the HashMap does not include a product state, then it was not observed over the
    /// `n` measurements. This method requires that the circuit has already been simulated by calling
    /// [Circuit::simulate].
    ///
    /// # Example
    /// ```
    /// use quantr::{states::SuperPosition, Circuit, Measurement::Observable, Gate};
    ///
    /// let mut circuit = Circuit::new(3).unwrap();
    ///
    /// circuit.add_gate(Gate::H, 2).unwrap();
    /// let simulated_circuit = circuit.simulate();
    ///
    /// // Measures 500 superpositions.
    /// println!("State | Number of Times Observed");
    /// if let Ok(Observable(bin_count)) = simulated_circuit.repeat_measurement(500) {
    ///     for (state, observed_count) in bin_count {
    ///         println!("|{}>   : {}", state, observed_count);
    ///     }
    /// }
    ///
    /// // State | Number of Times Observed
    /// // |000> : 247
    /// // |001> : 253
    /// ```
    pub fn repeat_measurement(
        &self,
        shots: usize,
    ) -> QResult<Measurement<HashMap<ProductState, usize>>> {
        match &self.circuit.output_state {
            Some(super_position) => {
                // Perform bin count of states
                let mut probabilities: HashMap<ProductState, f64> = Default::default();
                for (key, value) in super_position.to_hash_map() {
                    probabilities.insert(key, value.abs_square());
                }

                let mut bin_count: HashMap<ProductState, usize> = Default::default();

                for _ in 0..shots {
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
                Err(QuantrError{
                    message: String::from("The circuit has not been simulated. Call Circuit::simulate before calling this method, Circuit::repeat_measurement."),
                })
            },
        }
    }
}
