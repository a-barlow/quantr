/*
* Copyright (c) 2024 Andrew Rowan Barlow. Licensed under the EUPL-1.2
* or later. You may obtain a copy of the licence at
* https://joinup.ec.europa.eu/collection/eupl/eupl-text-eupl-12. A copy
* of the EUPL-1.2 licence in English is given in LICENCE.txt which is
* found in the root directory of this repository.
*
* Author: Andrew Rowan Barlow <a.barlow.dev@gmail.com>
*/

use crate::{
    complex_re,
    states::{ProductState, SuperPosition},
    Measurement,
};
use crate::{Circuit, Gate};
use std::collections::HashMap;

/// Contains the resulting state vector produced from the simulation of a circuit.
pub struct SimulatedCircuit {
    // Copy of Circuit struct but removed the wrapper around register.
    pub(crate) circuit_gates: Vec<Gate>,
    pub(crate) num_qubits: usize,
    pub(crate) register: SuperPosition,
    pub(crate) config_progress: bool,
    pub(super) disable_warnings: bool,
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
    /// For efficiency, this will use the cached register from the simulated circuit. If your
    /// circuit contains mixed states, then most likely the circuit will have to be simulated again
    /// for each shot. To achieve this, use [SimulatedCircuit::measure_all_without_cache].
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
        if self.circuit_gates.iter().any(|x| x.is_custom_gate()) && !self.disable_warnings {
            eprintln!("\x1b[93m[Quantr Warning] Custom gates were detected in the circuit. Measurements will be taken from a cached register in memory, and so if the Custom gate does NOT implement a unitary mapping, the measure_all method will most likely lead to wrong results. To simulate a circuit without cache, see SimulatedCircuit::measure_all_without_cache.\x1b[0m")
        }

        for _ in 0..shots {
            self.add_to_bin(&mut bin_count);
        }
        Measurement::Observable(bin_count)
    }

    /// Similar to [SimulatedCircuit::measure_all], however for every shot it will simulate the
    /// circuit, where the input register is reset to the zero state.
    ///
    /// This _potentially_ allows for mixed states to be simulated, through the implementation of
    /// [Gate::Custom]. In doing so will dramatically increase the simulation time, as a new
    /// circuit will be simulated for each shot.
    pub fn measure_all_without_cache(
        self,
        shots: usize,
    ) -> Measurement<HashMap<ProductState, usize>> {
        let mut bin_count: HashMap<ProductState, usize> = Default::default();
        let mut simulated_circ = self;
        simulated_circ.add_to_bin(&mut bin_count);
        if simulated_circ.config_progress {
            println!("Measured state # 1/{}", shots);
        }
        for i in 0..shots - 1 {
            // reset to |0> register
            simulated_circ
                .register
                .amplitudes
                .fill(num_complex::Complex64::ZERO);
            simulated_circ.register.amplitudes[0] = complex_re!(1f64);
            if simulated_circ.config_progress {
                println!("Register reset to zero state")
            }
            let circuit = Circuit {
                circuit_gates: simulated_circ.circuit_gates,
                num_qubits: simulated_circ.num_qubits,
                register: Some(simulated_circ.register),
                config_progress: simulated_circ.config_progress,
            };
            simulated_circ = circuit.simulate();
            simulated_circ.add_to_bin(&mut bin_count);
            if simulated_circ.config_progress {
                println!("Measured state # {}/{}", i + 2, shots);
            }
        }
        Measurement::Observable(bin_count)
    }

    fn add_to_bin(&self, bin: &mut HashMap<ProductState, usize>) {
        match self.register.measure() {
            Some(state) => {
                bin.entry(state)
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

    /// Sets if the printer should display warnings.
    pub fn print_warnings(&mut self, printing: bool) {
        self.disable_warnings = printing;
    }

    /// The slice of gates that composed the circuit, equivalent to [Circuit::get_gates].
    pub fn get_circuit_gates(&self) -> &Vec<Gate> {
        &self.circuit_gates
    }

    /// The number of qubits that composed the circuit, equivalent to [Circuit::get_num_qubits].
    pub fn get_num_qubits(&self) -> usize {
        self.num_qubits
    }

    /// Sets whether the simulation progress of the circuit will be printed to the terminal. This
    /// value is inherited from the circuit this struct was derived from.
    pub fn set_print_progress(&mut self, printing: bool) {
        self.config_progress = printing;
    }

    /// Takes ownership of the state that the `SimulatedCircuit` wraps around, that is the state
    /// that resulted from a circuit simulation.
    pub fn take_state(self) -> Measurement<SuperPosition> {
        Measurement::NonObservable(self.register)
    }
}
