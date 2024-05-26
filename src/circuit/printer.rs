/*
* Copyright (c) 2024 Andrew Rowan Barlow. Licensed under the EUPL-1.2
* or later. You may obtain a copy of the licence at
* https://joinup.ec.europa.eu/collection/eupl/eupl-text-eupl-12. A copy
* of the EUPL-1.2 licence in English is given in LICENCE.txt which is
* found in the root directory of this repository.
*
* Author: Andrew Rowan Barlow <a.barlow.dev@gmail.com>
*/

use super::{Circuit, Gate};
use std::fs::File;
use std::io::Write;
use std::path::Path;

/// Constructs, displays and saves the circuit diagram as a UTF-8 string.
///
/// The user has the option to print the string to the terminal or a text file, where the text file
/// has the advantage of not wrapping the circuit within the terminal. The [Printer] will also
/// cache a copy of the diagram so subsequent prints will require no building of the diagram.
pub struct Printer<'a> {
    circuit: &'a Circuit,
    diagram: Option<String>,
}

struct DiagramSchema<'a> {
    longest_name_length: usize,
    gate_info_column: Vec<GatePrinterInfo<'a>>,
}

#[derive(Clone)]
struct RowSchematic {
    top: String,
    name: String,
    bottom: String,
    connection: String,
}

#[derive(Clone)]
struct GatePrinterInfo<'a> {
    gate_name: String,
    gate_name_length: usize,
    gate: &'a Gate,
}

#[derive(Debug)]
struct Extrema {
    pub max: usize,
    pub min: usize,
}

impl Printer<'_> {
    /// Handle the printing of the given circuit.
    pub fn new<'circ>(circuit: &'circ Circuit) -> Printer<'circ> {
        Printer {
            circuit,
            diagram: None,
        }
    }

    /// Prints the circuit to the console in UTF-8.
    ///
    /// A warning is printed to the console if the circuit diagram is expected to exceed 72 chars.
    ///
    /// # Example
    /// ```
    /// use quantr::{Circuit, Gate, Printer};
    ///
    /// let mut qc: Circuit = Circuit::new(2).unwrap();
    /// qc.add_gate(Gate::CNot(0), 1).unwrap();
    ///
    /// let mut printer: Printer = Printer::new(&qc);
    /// printer.print_diagram();
    ///
    /// // The above prints:
    /// // ──█──
    /// //   │  
    /// //   │  
    /// // ┏━┷━┓
    /// // ┨ X ┠
    /// // ┗━━━┛
    /// ```
    pub fn print_diagram(&mut self) {
        if self.circuit.circuit_gates.len() / self.circuit.num_qubits > 14 {
            eprintln!("\x1b[93m[Quantr Warning] The string displaying the circuit diagram exceeds 72 chars, which could cause the circuit to render incorrectly in terminals (due to the wrapping). Instead, consider saving the string to a .txt file by using Printer::save_diagram.\x1b[0m");
        }
        println!("{}", self.get_or_make_diagram());
    }

    /// Saves the circuit diagram in UTF-8 chars to a text file.
    ///
    /// If the file already exists, it will overwrite it.
    ///
    /// # Example
    /// ```
    /// use quantr::{Circuit, Gate, Printer};
    ///
    /// let mut qc: Circuit = Circuit::new(2).unwrap();
    /// qc.add_gate(Gate::CNot(0), 1).unwrap();
    ///
    /// let mut printer: Printer = Printer::new(&qc);
    /// // printer.save_diagram("diagram.txt").unwrap();
    /// // Saves in directory of Cargo package.
    /// // (Commented so it doesn't create file during `cargo test`.)
    /// ```
    pub fn save_diagram(&mut self, file_path: &str) -> std::io::Result<()> {
        let path: &Path = Path::new(file_path);
        let mut file = File::create(path)?;
        file.write_all(self.get_or_make_diagram().as_bytes())
    }

    /// Prints the circuit diagram to the terminal and saves it to a text file in UTF-8.
    ///
    /// Essentially, this is a combination of [Printer::save_diagram] and [Printer::print_diagram].
    ///
    /// # Example
    /// ```
    /// use quantr::{Circuit, Gate, Printer};
    ///
    /// let mut qc: Circuit = Circuit::new(2).unwrap();
    /// qc.add_gate(Gate::CNot(0), 1).unwrap();
    ///
    /// let mut printer: Printer = Printer::new(&qc);
    /// // printer.print_and_save_diagram("diagram.txt").unwrap();
    /// // Saves in directory of cargo project, and prints to console.
    /// // (Commented so it doesn't create file during `cargo test`.)
    /// ```
    pub fn print_and_save_diagram(&mut self, file_path: &str) -> std::io::Result<()> {
        let diagram: String = self.get_or_make_diagram();

        println!("{}", diagram);

        let path = Path::new(file_path);
        let mut file = File::create(path)?;
        file.write_all(diagram.as_bytes())
    }

    /// Returns the circuit diagram that is made from UTF-8 chars.
    ///
    /// # Example
    /// ```
    /// use quantr::{Circuit, Gate, Printer};
    ///
    /// let mut qc: Circuit = Circuit::new(2).unwrap();
    /// qc.add_gate(Gate::CNot(0), 1).unwrap();
    ///
    /// let mut printer: Printer = Printer::new(&qc);
    /// println!("{}", printer.get_diagram()); // equivalent to Printer::print_diagram
    /// ```
    pub fn get_diagram(&mut self) -> String {
        self.get_or_make_diagram()
    }

    // Constructs the diagram, or returns the diagram previously built.
    fn get_or_make_diagram(&mut self) -> String {
        match &self.diagram {
            Some(diagram) => diagram.to_string(),
            None => self.make_diagram(),
        }
    }

    fn make_diagram(&mut self) -> String {
        // num qubits cannot be zero due to initialisation
        let number_of_columns: usize = self.circuit.circuit_gates.len() / self.circuit.num_qubits;
        let mut printed_diagram: Vec<String> =
            vec!["".to_string(); 4 * self.circuit.num_qubits + 1];

        for column_num in 0..number_of_columns {
            // Get a column of gates with all names and length of names
            let (gate_info_column, longest_name_length): (Vec<GatePrinterInfo>, usize) =
                Self::into_printer_gate_info(self.get_column_of_gates(column_num));

            let diagram_schematic = DiagramSchema {
                longest_name_length,
                gate_info_column,
            };

            if let Some((position, multi_gate_info)) =
                Self::get_multi_gate(&diagram_schematic.gate_info_column)
            {
                // Deals with column of single multi-gate
                Self::draw_multi_gates(
                    &mut printed_diagram,
                    multi_gate_info,
                    &self.circuit.num_qubits,
                    position,
                );
            } else {
                // Deals with single gates
                Self::draw_single_gates(printed_diagram.as_mut_slice(), diagram_schematic);
            }
        }

        // Collect all the strings to return a single string giving the diagram
        let final_diagram = printed_diagram
            .into_iter()
            .fold(String::from(""), |acc, line| acc + &line + "\n");

        self.diagram = Some(final_diagram.clone());

        final_diagram
    }

    fn get_column_of_gates(&self, column_num: usize) -> &[Gate] {
        &self.circuit.circuit_gates
            [column_num * self.circuit.num_qubits..(column_num + 1) * self.circuit.num_qubits]
    }

    fn into_printer_gate_info(gates_column: &[Gate]) -> (Vec<GatePrinterInfo>, usize) {
        let mut gates_infos: Vec<GatePrinterInfo> = Default::default();
        let mut longest_name_length: usize = 1usize;
        for gate in gates_column.iter() {
            let gate_name: String = gate.get_name();
            let gate_name_length: usize = gate_name.len();
            if gate_name_length > longest_name_length {
                longest_name_length = gate_name_length;
            }
            gates_infos.push(GatePrinterInfo {
                gate_name,
                gate_name_length,
                gate,
            })
        }
        (gates_infos, longest_name_length)
    }

    // Finds if there is a gate with one/multiple control nodes
    fn get_multi_gate<'gate>(
        gates: &[GatePrinterInfo<'gate>],
    ) -> Option<(usize, GatePrinterInfo<'gate>)> {
        for (pos, gate_info) in gates.iter().enumerate() {
            if !gate_info.gate.is_single_gate() {
                return Some((pos, gate_info.clone()));
            }
        }
        None
    }

    // Draw a column of single gates
    fn draw_single_gates(row_schematics: &mut [String], diagram_scheme: DiagramSchema) {
        for (pos, gate_info) in diagram_scheme.gate_info_column.iter().enumerate() {
            let padding: usize = diagram_scheme.longest_name_length - gate_info.gate_name_length;
            let cache: RowSchematic = match gate_info.gate {
                Gate::Id => RowSchematic {
                    top: " ".repeat(diagram_scheme.longest_name_length + 4),
                    name: "─".repeat(diagram_scheme.longest_name_length + 4),
                    bottom: " ".repeat(diagram_scheme.longest_name_length + 4),
                    connection: " ".repeat(diagram_scheme.longest_name_length + 4),
                },
                _ => RowSchematic {
                    top: "┏━".to_string()
                        + &"━".repeat(gate_info.gate_name_length)
                        + "━┓"
                        + &" ".repeat(padding),
                    name: "┨ ".to_string() + &gate_info.gate_name + " ┠" + &"─".repeat(padding),
                    bottom: "┗━".to_string()
                        + &"━".repeat(gate_info.gate_name_length)
                        + "━┛"
                        + &" ".repeat(padding),
                    connection: " ".repeat(diagram_scheme.longest_name_length + 4),
                },
            };
            Self::add_string_to_schematic(row_schematics, pos, cache)
        }
    }

    // Draw a single column containing a multigate function.
    fn draw_multi_gates(
        row_schematics: &mut [String],
        multi_gate_info: GatePrinterInfo<'_>,
        column_size: &usize,
        position: usize,
    ) {
        let mut control_nodes: Vec<usize> = multi_gate_info
            .gate
            .get_nodes()
            .expect("Single gate in drawing multi gate.");
        control_nodes.push(position);

        let (min, max): (usize, usize) = (
            *control_nodes.iter().min().unwrap(),
            *control_nodes.iter().max().unwrap(),
        );

        let extreme_nodes: Extrema = Extrema { max, min };

        for row in 0..*column_size {
            let cache: RowSchematic = if row == position {
                RowSchematic {
                    top: "┏━".to_string()
                        + if position > extreme_nodes.min {
                            "┷"
                        } else {
                            "━"
                        }
                        + &"━".repeat(multi_gate_info.gate_name_length - 1)
                        + "━┓",
                    name: "┨ ".to_string() + &multi_gate_info.gate_name + " ┠",
                    bottom: "┗━".to_string()
                        + if position < extreme_nodes.max {
                            "┯"
                        } else {
                            "━"
                        }
                        + &"━".repeat(multi_gate_info.gate_name_length - 1)
                        + "━┛",
                    connection: "  ".to_string()
                        + if position < extreme_nodes.max {
                            "│"
                        } else {
                            " "
                        }
                        + &" ".repeat(multi_gate_info.gate_name_length + 1),
                }
            } else if row == extreme_nodes.min {
                RowSchematic {
                    top: " ".repeat(multi_gate_info.gate_name_length + 4),
                    name: "──█──".to_string() + &"─".repeat(multi_gate_info.gate_name_length - 1),
                    bottom: "  │  ".to_string() + &" ".repeat(multi_gate_info.gate_name_length - 1),
                    connection: "  │  ".to_string()
                        + &" ".repeat(multi_gate_info.gate_name_length - 1),
                }
            } else if row == extreme_nodes.max {
                RowSchematic {
                    top: "  │  ".to_string() + &" ".repeat(multi_gate_info.gate_name_length - 1),
                    name: "──█──".to_string() + &"─".repeat(multi_gate_info.gate_name_length - 1),
                    bottom: " ".repeat(multi_gate_info.gate_name_length + 4),
                    connection: " ".repeat(multi_gate_info.gate_name_length + 4),
                }
            } else if control_nodes.contains(&row) {
                RowSchematic {
                    top: "  │  ".to_string() + &" ".repeat(multi_gate_info.gate_name_length - 1),
                    name: "──█──".to_string() + &"─".repeat(multi_gate_info.gate_name_length - 1),
                    bottom: "  │  ".to_string() + &" ".repeat(multi_gate_info.gate_name_length - 1),
                    connection: "  │  ".to_string()
                        + &" ".repeat(multi_gate_info.gate_name_length - 1),
                }
            } else if (extreme_nodes.min..=extreme_nodes.max).contains(&row) {
                RowSchematic {
                    top: "  │  ".to_string() + &" ".repeat(multi_gate_info.gate_name_length - 1),
                    name: "──┼──".to_string() + &"─".repeat(multi_gate_info.gate_name_length - 1),
                    bottom: "  │  ".to_string() + &" ".repeat(multi_gate_info.gate_name_length - 1),
                    connection: "  │  ".to_string()
                        + &" ".repeat(multi_gate_info.gate_name_length - 1),
                }
            } else {
                RowSchematic {
                    top: " ".repeat(multi_gate_info.gate_name_length + 4),
                    name: "─────".to_string() + &"─".repeat(multi_gate_info.gate_name_length - 1),
                    bottom: " ".to_string() + &" ".repeat(multi_gate_info.gate_name_length + 3),
                    connection: " ".to_string() + &" ".repeat(multi_gate_info.gate_name_length + 3),
                }
            };
            Self::add_string_to_schematic(row_schematics, row, cache)
        }
    }

    // Adds a gate to the vector of strings.
    fn add_string_to_schematic(
        schematic: &mut [String],
        row_schem_num: usize,
        cache: RowSchematic,
    ) {
        schematic[row_schem_num * 4].push_str(&cache.top);
        schematic[row_schem_num * 4 + 1].push_str(&cache.name);
        schematic[row_schem_num * 4 + 2].push_str(&cache.bottom);
        schematic[row_schem_num * 4 + 3].push_str(&cache.connection);
    }
}

#[rustfmt::skip]
#[cfg(test)]
mod tests {
    
    use crate::{
        Printer, Circuit, Gate, states::{Qubit, ProductState, SuperPosition},
    };
    use crate::Complex;
    use crate::complex_re_array;
    // These are primarily tested by making sure they print correctly to
    // the terminal, and then copy the output for the assert_eq! macro.

    fn example_cnot(prod: ProductState) -> Option<SuperPosition> {
        let input_register: [Qubit; 2] = [prod.qubits[0], prod.qubits[1]];
        Some(SuperPosition::new_with_amplitudes(match input_register {
                [Qubit::Zero, Qubit::Zero] => return None,
                [Qubit::Zero, Qubit::One] => return None, 
                [Qubit::One, Qubit::Zero] => &complex_re_array!(0f64, 0f64, 0f64, 1f64),
                [Qubit::One, Qubit::One] => &complex_re_array!(0f64, 0f64, 1f64, 0f64),
            })
            .unwrap())
    }

    #[test]
    fn producing_string_circuit() {
        let mut quantum_circuit = Circuit::new(4).unwrap();
        quantum_circuit.add_gate(Gate::H, 3).unwrap()
            .add_repeating_gate(Gate::Y, &[0, 1]).unwrap()
            .add_gate(Gate::Toffoli(0, 3), 1).unwrap()
            .add_gate(Gate::CNot(1), 3).unwrap()
            .add_gate(Gate::CNot(2), 0).unwrap()
            .add_gate(Gate::CNot(2), 1).unwrap();

        let mut circuit_printer: Printer = Printer::new(&quantum_circuit);

        circuit_printer.print_diagram();

        assert_eq!(circuit_printer.get_diagram(), "     ┏━━━┓          ┏━━━┓     \n─────┨ Y ┠──█───────┨ X ┠─────\n     ┗━━━┛  │       ┗━┯━┛     \n            │         │       \n     ┏━━━┓┏━┷━┓       │  ┏━━━┓\n─────┨ Y ┠┨ X ┠──█────┼──┨ X ┠\n     ┗━━━┛┗━┯━┛  │    │  ┗━┯━┛\n            │    │    │    │  \n            │    │    │    │  \n────────────┼────┼────█────█──\n            │    │            \n            │    │            \n┏━━━┓       │  ┏━┷━┓          \n┨ H ┠───────█──┨ X ┠──────────\n┗━━━┛          ┗━━━┛          \n                              \n\n".to_string());
    }

    #[test]
    fn producing_string_circuit_custom() {
        let mut quantum_circuit = Circuit::new(4).unwrap();
        quantum_circuit.add_gate(Gate::H, 3).unwrap();
        quantum_circuit
            .add_gates(&[
                Gate::H,
                Gate::Custom(example_cnot, vec!(3), "Custom CNot".to_string()),
                Gate::Id,
                Gate::X,
            ]).unwrap()
            .add_repeating_gate(Gate::Y, &[0, 1]).unwrap()
            .add_gate(Gate::Toffoli(0, 3), 1).unwrap()
            .add_gate(Gate::CNot(1), 3).unwrap()
            .add_gate(Gate::CNot(2), 0).unwrap()
            .add_gate(Gate::CNot(2), 1).unwrap();

        let mut circuit_printer: Printer = Printer::new(&quantum_circuit);

        circuit_printer.print_diagram();

        assert_eq!(circuit_printer.get_diagram(), "     ┏━━━┓               ┏━━━┓          ┏━━━┓     \n─────┨ H ┠───────────────┨ Y ┠──█───────┨ X ┠─────\n     ┗━━━┛               ┗━━━┛  │       ┗━┯━┛     \n                                │         │       \n          ┏━━━━━━━━━━━━━┓┏━━━┓┏━┷━┓       │  ┏━━━┓\n──────────┨ Custom CNot ┠┨ Y ┠┨ X ┠──█────┼──┨ X ┠\n          ┗━┯━━━━━━━━━━━┛┗━━━┛┗━┯━┛  │    │  ┗━┯━┛\n            │                   │    │    │    │  \n            │                   │    │    │    │  \n────────────┼───────────────────┼────┼────█────█──\n            │                   │    │            \n            │                   │    │            \n┏━━━┓┏━━━┓  │                   │  ┏━┷━┓          \n┨ H ┠┨ X ┠──█───────────────────█──┨ X ┠──────────\n┗━━━┛┗━━━┛                         ┗━━━┛          \n                                                  \n\n".to_string());
    }
}
