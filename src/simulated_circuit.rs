/*
* Copyright (c) 2024 Andrew Rowan Barlow. Licensed under the EUPL-1.2
* or later. You may obtain a copy of the licence at
* https://joinup.ec.europa.eu/collection/eupl/eupl-text-eupl-12. A copy
* of the EUPL-1.2 licence in English is given in LICENCE.txt which is
* found in the root directory of this repository.
*
* Author: Andrew Rowan Barlow <a.barlow.dev@gmail.com>
*/

use crate::Gate;
use crate::{
    states::{ProductState, SuperPosition},
    Measurement,
};
use std::collections::HashMap;

pub struct SimulatedCircuit {
    // Copy of Circuit struct but removed the wrapper around register.
    pub(crate) circuit_gates: Vec<Gate>,
    pub(crate) num_qubits: usize,
    pub(crate) register: SuperPosition,
    pub(crate) config_progress: bool,
    pub(super) disable_warnings: bool,
    pub(super) cache_register: bool,
}

impl SimulatedCircuit {
    /// Returns a `HashMap` that contains the number of times the corresponding state was observed over
    /// `n` measurements of the superpositions (shots).
    ///
    /// Explicitly, this performs repeated measurements where a register is attached to the circuit,
    /// the resulting superposition measured in the computational basis, and then the reduced state
    /// recorded. If the HashMap does not include a product state, then it was not observed over the
    /// `n` measurements.
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
    /// if let Observable(bin_count) = simulated_circuit.measure_all(500) {
    ///     for (state, observed_count) in bin_count {
    ///         println!("|{}>   : {}", state, observed_count);
    ///     }
    /// }
    ///
    /// // State | Number of Times Observed
    /// // |000> : 247
    /// // |001> : 253
    /// ```
    pub fn measure_all(&self, shots: usize) -> Measurement<HashMap<ProductState, usize>> {
        let mut bin_count: HashMap<ProductState, usize> = Default::default();
        if self.cache_register
            && self.circuit_gates.iter().any(|x| x.is_custom_gate())
            && !self.disable_warnings
        {
            eprintln!("\x1b[93m[Quantr Warning] Custom gates were detected in the circuit. Measurements will be taken from a cached register, and so if the Custom gate does NOT implement a unitary mapping, the measure all function will most likely lead to wrong results. To turn the cache off, see [SimulatedCircuit::toggle_cache].")
        }
        for _ in 0..shots {
            match self.register.measure() {
                Some(state) => {
                    bin_count
                        .entry(state)
                        .and_modify(|count| {
                            *count = *count + 1;
                        })
                        .or_insert(1);
                }
                None if !self.disable_warnings => {
                    eprintln!("\x1b[93m[Quantr Warning] The superposition failed to collapse to a state during repeat measurements. This is likely due to the use of Gate::Custom where the mapping is not unitary.\x1b[0m")
                }
                None => {}
            }
        }
        Measurement::Observable(bin_count)
    }

    /// Returns the resulting superposition after the circuit has been simulated using
    /// [super::Circuit::simulate].
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
    /// if let NonObservable(super_pos) = simulated_circuit.get_state() {
    ///     for (state, amplitude) in super_pos.into_iter() {
    ///         println!("|{}>   : {}", state.to_string(), amplitude);
    ///     }
    /// }
    ///
    /// // State | Amplitude of State
    /// // |000> : 0 - 0.71...i
    /// // |001> : 0 + 0.71...i
    /// ```
    pub fn get_state(&self) -> Measurement<&SuperPosition> {
        Measurement::NonObservable(&self.register)
    }

    /// Toggles if the printer should display warnings.
    pub fn toggle_warnings(&mut self) {
        self.disable_warnings = !self.disable_warnings;
    }

    pub fn toggle_cache(&mut self) {
        self.cache_register = !self.cache_register;
    }

    pub fn get_circuit_gates(&self) -> &Vec<Gate> {
        &self.circuit_gates
    }

    pub fn get_num_qubits(&self) -> usize {
        self.num_qubits
    }

    pub fn get_cache_status(&self) -> bool {
        self.cache_register
    }

    pub fn get_toggle_progress(&self) -> bool {
        self.config_progress
    }

    pub fn toggle_simulation_progress(&mut self) {
        self.config_progress = !self.config_progress;
    }

    pub fn take_state(self) -> Measurement<SuperPosition> {
        Measurement::NonObservable(self.register)
    }
}
