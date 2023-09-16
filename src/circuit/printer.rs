/*
* Copyright (c) 2023 Andrew Rowan Barlow. Licensed under the EUPL-1.2
* or later. You may obtain a copy of the licence at
* https://joinup.ec.europa.eu/collection/eupl/eupl-text-eupl-12. A copy
* of the EUPL-1.2 licence in English is given in LICENCE.txt which is
* found in the root directory of this repository.
*
* Author: Andrew Rowan Barlow <a.barlow.dev@gmail.com>
*/

//! Constructs, displays and saves the circuit diagram as a UTF-8 string.
//!
//! The user has the option to print the string to the terminal or a text file, where the text file
//! has the advantage of not wrapping the circuit within the terminal. The [Printer] will also
//! chache a copy of the diagram so subsequent prints will require no building of the diagram. This
//! cache can be removed with [Printer::flush] to force the [Printer] to construct the cicuit
//! diagram again.

//.!!! Developer Warning !!!
// This module is very messy, and will it's code will be cleared up in a near future update.

use crate::circuit::StandardGate;
use crate::circuit::{Circuit, GateInfo, GateSize};
use std::fs::File;
use std::io::Write;
use std::path::Path;

const GATE_WIDTH_MIN: usize = 3;

/// Handles the printing of the given circuit.
pub struct Printer<'a> {
    circuit: &'a Circuit<'a>,
    diagram: Option<String>,
}

struct PrintCache {
    top: String,
    name: String,
    bottom: String,
    connection: String,
}

struct PrintBuffer {
    wire: String,
    empty: String,
    boundary: String,
}

#[derive(Debug)]
struct Extrema {
    max: usize,
    min: usize,
}

impl Printer<'_> {
    /// Handles the printing of the circuit.
    pub fn new<'a>(circuit: &'a Circuit) -> Printer<'a> {
        Printer {
            circuit,
            diagram: None,
        }
    }

    /// Prints the circuit to the console in UTF-8.
    pub fn print_diagram(&mut self) {
        if self.circuit.circuit_gates.len() / self.circuit.num_qubits > 14 {
            println!("\x1b[93m[Quantr Warning] The string displaying the circuit diagram exceeds 72 chars, causing the circuit to render incorrectly in terminals (due to the wrapping). Instead, consider saving the string to a .txt file by using Printer::save_diagram.\x1b[0m");
        }
        println!("{}", self.get_or_make_diagram());
    }

    /// Saves the circuit diagram to a text file in UTF-8 chars.
    ///
    /// If the file already exists, it will overwrite it.
    pub fn save_diagram(&mut self, file_path: &str) -> std::io::Result<()> {
        let path: &Path = Path::new(file_path);
        let mut file = File::create(&path)?;
        file.write_all(self.get_or_make_diagram().as_bytes())
    }

    /// Prints the circuit diagram to the terminal and saves it to a text file in UTF-8.
    ///
    /// Essentially, this is a combination of [Printer::save_diagram] and [Printer::print_diagram].
    pub fn print_and_save_diagram(&mut self, file_path: &str) -> std::io::Result<()> {
        let diagram: String = self.get_or_make_diagram();

        println!("{}", diagram);

        let path = Path::new(file_path);
        let mut file = File::create(&path)?;
        file.write_all(diagram.as_bytes())
    }

    /// Returns the circuit diagram that is made from UTF8-chars.
    pub fn get_diagram(&mut self) -> String {
        self.get_or_make_diagram()
    }

    /// Removes the cache of the circuit diagram.
    ///
    /// Future calls to print the diagram will have to build the diagram from scratch. Can be used
    /// if the circuit has been updated, and the printer needs to rebuild the same circuit.
    pub fn flush(&mut self) {
        self.diagram = None;
    }

    // Constructs the diagram, or returns the diagram previously built.
    fn get_or_make_diagram(&mut self) -> String {
        match &self.diagram {
            Some(diagram) => diagram.to_string(),
            None => {
                self.make_diagram();
                self.diagram.clone().unwrap()
            }
        }
    }

    // Change name to get UTF-8 representation after testing. This requires the asumption that
    // there is only one node per row
    fn make_diagram(&mut self) {
        // For every row, there is fourspaces to control.
        let mut schematic: Vec<String> = vec![String::from(""); 4 * self.circuit.num_qubits + 1];

        let wire: String = (0..GATE_WIDTH_MIN + 2)
            .into_iter()
            .map(|_| "─")
            .collect::<String>();
        let empty: String = (0..GATE_WIDTH_MIN + 2)
            .into_iter()
            .map(|_| " ")
            .collect::<String>();
        let boundary: String = (0..GATE_WIDTH_MIN)
            .into_iter()
            .map(|_| "━")
            .collect::<String>();
        let buffer: PrintBuffer = PrintBuffer {
            wire,
            empty,
            boundary,
        };

        for column_num in 0..(self.circuit.circuit_gates.len() / self.circuit.num_qubits) {
            //check if there exists nodes
            let row_gates: &[StandardGate] = &self.circuit.circuit_gates
                [column_num * self.circuit.num_qubits..(column_num + 1) * self.circuit.num_qubits];

            if let Some((name, position)) = Self::has_double_gate(row_gates) {
                let double_gate: GateInfo = GateInfo {
                    name,
                    position,
                    size: GateSize::Double,
                };
                Self::draw_double_gate(&mut schematic, double_gate, row_gates, &buffer);
            } else {
                // single gates
                Self::draw_single_gate(&mut schematic, row_gates, &buffer);
            }
        }

        self.diagram = Some(
            schematic
                .into_iter()
                .fold(String::from(""), |acc, line| acc + &line + &"\n"),
        );
    }

    fn get_gate_name(gate: &StandardGate) -> String {
        match gate {
            StandardGate::X => String::from("X"),
            StandardGate::H => String::from("H"),
            StandardGate::Y => String::from("Y"),
            StandardGate::Z => String::from("Z"),
            StandardGate::Swap(_) => String::from("Z"),
            StandardGate::CY(_) => String::from("Y"),
            StandardGate::CNot(_) => String::from("X"),
            StandardGate::Toffoli(_, _) => String::from("T"),
            _ => String::from("#"),
        }
    }

    fn has_double_gate<'a>(gates: &[StandardGate<'a>]) -> Option<(StandardGate<'a>, usize)> {
        for (i, gate) in gates.iter().enumerate() {
            match gate {
                StandardGate::CNot(_) | StandardGate::Toffoli(_, _) => {
                    return Some((gate.clone(), i))
                }
                _ => (),
            }
        }
        None
    }

    fn add_to_schematic(schematic: &mut Vec<String>, row_schem_num: &usize, cache: PrintCache) {
        schematic[*row_schem_num].push_str(&cache.top);
        schematic[*row_schem_num + 1].push_str(&cache.name);
        schematic[*row_schem_num + 2].push_str(&cache.bottom);
        schematic[*row_schem_num + 3].push_str(&cache.connection);
    }

    fn draw_connected_gate(
        printing_info: &mut PrintCache,
        extreme: &Extrema,
        row_num: &usize,
        buff: &PrintBuffer,
    ) {
        if extreme.max > *row_num && *row_num > extreme.min {
            printing_info.top = String::from("┏━┷━┓");
            printing_info.bottom = String::from("┗━┯━┛");
            printing_info.connection = String::from("  │  ");
        } else if extreme.max > *row_num {
            printing_info.top = String::from("┏") + &buff.boundary.clone() + "┓";
            printing_info.bottom = String::from("┗━┯━┛");
            printing_info.connection = String::from("  │  ");
        } else {
            printing_info.top = String::from("┏━┷━┓");
            printing_info.bottom = String::from("┗━━━┛");
        }
    }

    fn draw_double_gate<'a>(
        schematic: &mut Vec<String>,
        double_gate: GateInfo,
        row_gates: &[StandardGate],
        buff: &PrintBuffer,
    ) {
        let mut targets: Vec<usize> = Default::default();
        let max_name: usize = row_gates
            .iter()
            .map(Self::get_gate_name)
            .map(|s| s.len())
            .max()
            .unwrap();

        // Not exhaustive, as only double gates will be considered.
        if let StandardGate::CNot(t1) = double_gate.name {
            targets.push(t1);
        } else if let StandardGate::Toffoli(t1, t2) = double_gate.name {
            targets.push(t1);
            targets.push(t2);
        }

        targets.push(double_gate.position);
        let target_extrema = Extrema {
            max: *targets.iter().max().unwrap(),
            min: *targets.iter().min().unwrap(),
        };

        let extended: String = Self::repeat('─', max_name - 1);
        let extended_empty: String = Self::repeat(' ', max_name - 1);

        for (row_num, gate) in row_gates.iter().enumerate() {
            let mut printing_info: PrintCache = PrintCache {
                top: buff.empty.clone() + &extended_empty,
                name: buff.wire.clone() + &extended,
                bottom: buff.empty.clone() + &extended_empty,
                connection: buff.empty.clone() + &extended_empty,
            };

            if *gate != StandardGate::Id {
                // This only works from assuming that a
                // double gate or greater only for one
                // column.
                printing_info.name = String::from("┨ ") + &Self::get_gate_name(&gate) + &" ┠";
                Self::draw_connected_gate(&mut printing_info, &target_extrema, &row_num, buff);
            } else if target_extrema.max == row_num {
                printing_info.top = String::from("  │  ");
                printing_info.name = String::from("──█──");
            } else if target_extrema.min == row_num {
                printing_info.name = String::from("──█──");
                printing_info.bottom = String::from("  │  ");
                printing_info.connection = String::from("  │  ");
            } else if targets.contains(&row_num) {
                printing_info.top = String::from("  │  ");
                printing_info.name = String::from("──█──");
                printing_info.bottom = String::from("  │  ");
                printing_info.connection = String::from("  │  ");
            } else if target_extrema.max > row_num && row_num > target_extrema.min {
                printing_info.top = String::from("  │  ");
                printing_info.name = String::from("─────");
                printing_info.bottom = String::from("  │  ");
                printing_info.connection = String::from("  │  ");
            }

            Self::add_to_schematic(schematic, &(row_num * 4), printing_info);
        }
    }

    fn repeat(c: char, n: usize) -> String {
        (0..n).map(|_| c).collect::<String>()
    }

    fn draw_single_gate<'a>(
        schematic: &mut Vec<String>,
        row_gates: &[StandardGate],
        buff: &PrintBuffer,
    ) {
        let max_name: usize = row_gates
            .iter()
            .map(Self::get_gate_name)
            .map(|s| s.len())
            .max()
            .unwrap();

        for (i, gate) in row_gates.iter().enumerate() {
            let mut printing_info: PrintCache = PrintCache {
                top: String::from(""),
                name: String::from(""),
                bottom: String::from(""),
                connection: String::from(""),
            };

            match gate {
                StandardGate::Id => {
                    let extended: String = Self::repeat('─', max_name - 1);
                    let extended_empty: String = Self::repeat(' ', max_name - 1);

                    printing_info.top = buff.empty.clone() + &extended_empty;
                    printing_info.name = buff.wire.clone() + &extended;
                    printing_info.bottom = buff.empty.clone() + &extended_empty;
                    printing_info.connection = buff.empty.clone() + &extended_empty;
                }
                _ => {
                    let name_width: String = (0..max_name - Self::get_gate_name(&gate).len())
                        .map(|_| "─")
                        .collect::<String>();
                    let empty_width: String = (0..max_name - Self::get_gate_name(&gate).len())
                        .map(|_| " ")
                        .collect::<String>();
                    let extended: String = (0..Self::get_gate_name(gate).len() + 2)
                        .map(|_| "━")
                        .collect::<String>();
                    let extended_empty: String = (0..Self::get_gate_name(gate).len() - 1)
                        .map(|_| " ")
                        .collect::<String>();

                    printing_info.top = String::from("┏") + &extended + "┓" + &empty_width;
                    printing_info.name =
                        String::from("┨ ") + &Self::get_gate_name(&gate) + &" ┠" + &name_width; //GATE_WIDTH_MIN
                    printing_info.bottom = String::from("┗") + &extended + "┛" + &empty_width;
                    printing_info.connection = String::from("     ") + &extended_empty;
                }
            }

            Self::add_to_schematic(schematic, &(i * 4), printing_info);
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::circuit::{printer::Printer, Circuit, StandardGate};
    // These are primarly tested by making sure they print correctly to
    // the terminal, and then copy the output for the assert_eq! macro.

    #[test]
    fn producing_string_circuit() {
        let mut quantum_circuit = Circuit::new(4);
        quantum_circuit.add_gate(StandardGate::H, 3).unwrap();
        quantum_circuit
            .add_repeating_gate(StandardGate::Y, vec![0, 1])
            .unwrap();
        quantum_circuit
            .add_gate(StandardGate::Toffoli(0, 3), 1)
            .unwrap();
        quantum_circuit.add_gate(StandardGate::CNot(1), 3).unwrap();

        let mut circuit_printer: Printer = Printer::new(&quantum_circuit);

        // circuit_printer.print_diagram();

        assert_eq!(circuit_printer.get_diagram(), String::from("     ┏━━━┓          \n─────┨ Y ┠──█───────\n     ┗━━━┛  │       \n            │       \n     ┏━━━┓┏━┷━┓     \n─────┨ Y ┠┨ T ┠──█──\n     ┗━━━┛┗━┯━┛  │  \n            │    │  \n            │    │  \n────────────────────\n            │    │  \n            │    │  \n┏━━━┓       │  ┏━┷━┓\n┨ H ┠───────█──┨ X ┠\n┗━━━┛          ┗━━━┛\n                    \n\n"));
    }
}
