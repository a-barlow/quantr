# Changelog

This file logs the versions of quantr.

## 0.3.0 - UNTITLED

This major update overhauls the structure of quantr, and the naming of
many methods. The aim is to increase simplicity in using the library,
in turn producing more readable and efficient code. The re-naming of
methods is meant to be more inkeeping with the Rust standard library,
that is `to` represents a pass by reference, while `into` moves the value
into the method.

Moreover, some examples have been added showcasing custom functions and
printing the circuits in a variety of ways. 

Features:

- Removed `Printer::flush` as it cannot be used due to borrowing rules.
- Renamed the enum `StandardGate` to `Gate`.
- The `complex_zero!` macro has been replaced with a `Complex<f64>`
  constant `quantr::COMPLEX_ZERO`. 
- Changed method names:
    - `Qubit::join` -> `Qubit::kronecker_prod`
    - `Qubit::as_state` -> `Qubit::into_state`
    - `ProductState::join` -> `ProductState::kronecker_prod`
    - `ProductState::as_string` -> `ProductState::to_string`
    - `SuperPosition::as_hash_map` -> `SuperPosition::to_hash_map`
    - `ProductState::to_super_position` ->
      `ProductState::into_super_position`
- The field of `ProductState` called `state` -> `qubits`.
- The `QuantrError` struct has been made public for the user (this was
  available in versions < 0.2.0). This allows for succint error handling
  with `?` when creating circuits when the main function is allowed to
  return `Result<(), QuantrError>`.
- Re-structured access of structs and module paths. Now, every struct is
  accessed through `quantr::...` except for those that control states,
  which are accessed through the module `quantr::states::...`.
- Changed the input type of two methods in `Circuit`:
    - `add_gates(Vec<Gate>)` -> `add_gates(&[Gate])`
    - `add_repeating_gate(Gate, Vec<usize>)` ->
      `add_repeating_gate(Gate, &[usize])`.
- `Circuit` methods that add gates now output a mutable reference to the
  mutated circuit. This allows for a 'chain of method calls' to be made.

Examples:

All examples print the circuit to the console, along with data of the
resulting superpositions from simulating the circuit.

- A custom function implementing a Quantum Fourier Transform in
  `examples/qft.rs` which is designed to be used in other circuits. This
  also showcases the idea of running a circuit within a custom function,
  in a way sub-divding components of the larger circuit into smaller
  ones.
- The custom function itself, implementing a CCC-not gate, is shown in
  `examples/custom_gates.rs`. Within this function, the product states
  are directly manipulated to produce a CCC-not gate. This example also
  prints the progress of the simulation.

Tests:

- All tests and examples have been updated to reflect this major change.
  Now answers had to be changed, only the interfaces with quantr.
- Boundary test to catch if a control node is greater than the size of
  the circuit.

## 0.2.5 - Complex exponential, ASCII warnings and gates

Features:

- `SuperPosition` now has a new publicly available field that specifies
  the product dimension of the superposition (the number of qubits in
  each computational state).
- Added `Circuit::simulate_with_register`, which allows to attach a
  custom register defined by a `SuperPosition`.
- All gates from the cQASM instruction set have now been added. The
  gates that were added to complete this set are:
  - Rx (Rotation around x-axis)
  - Ry (Rotation around y-axis)
  - Rz (Rotation around z-axis)
  - X90 (90 degree rotation around x-axis)
  - MX90 (conjugate of above)
  - Y90 (90 degree rotation around y-axis)
  - MY90 (conjugate of above)
  - Phase (implements a global phase change on single qubit)
  - CR (controlled rotation)
  - CRk (controlled rotation for QFT implementation)
- `Complex` now has an `expi` function, which implements the complex
  exponential raised to a real number. This returns a `Complex<f64>` or
  `Complex<f32>`.

Fixes:

- T and S conjugate gates now have ASCII names when printed in a circuit
  diagram. Before hand, this would have potentially ruined circuit
  diagrams.
- A warning has now been added when ASCII strings are used to label
  custom functions.

Optimisations:

- A new method in `SuperPosition` was added to bypass checks on
  conservation of probability for standard gates (that have been checked
  manually).

## 0.2.4 - Add S (Phase) and T gates

Features:

- Phase gate and conjugate has been added.
- T gate and conjugate has been added.

Tests:

- Unit tests for Phase and T gates, with their conjugates, are tested in
  2 qubit circuits.

## 0.2.3 - Extra functionality for `ProductState` and added examples

Features:

- [circuit::states::ProductStates] has two new methods:
    - [ProductStates::invert_digit] will invert the qubit digit that
      represents the state.
    - [ProductStates::to_super_position] transforms the product state
      into a superposition with one amplitude.
- Additional examples added to circuits and printer methods.
- A Grovers algorithm example has been added to the cargo. This example
  is the completed code from the
  [quantr-book](https://a-barlow.github.io/quantr-book/), and can be
  run with `cargo run --example grovers`. 

Fixes:

- The labelling of the Toffoli gate by the printer has changed from 'To'
  to 'X'.
- The labelling of the swap gate by the printer has changed from 'Swap'
  to 'Sw'.
- The printer will now print vertical lines overlapping the horizontal
  wires as connected lines; they are no longer spaced. This was decided
  as for large diagrams, the 'unconnected' wires strained the eyes. It
  was also noted that the 'scintillating grid illusion' occurs for big
  diagrams when there are many nodes.

Deprecated:

- `Printer::flush` is deprecated and will be removed next major update,
  as it cannot be used as the quantum circuit struct it borrows is
  mutable, and thus cannot mutate the circuit after printing it. This
  method makes no sense to exist then.

Tests:

- New grovers test that implements a 3x3 single row of sudoku solver.
- Add unit test of inverting binary digits labelling computational basis
  using `ProductStates::invert_digit`.
- Add unit test of `Product::to_super_position`.

## 0.2.2 - Fixing the `Printer` and pushing of custom functions 

Features:

- A usage section in the README.md; displaying a small snippet of quantr
  code.

Fixes:

- Custom gates added to columns were not automatically pushed so that
  they were isolated.
- The `Printer` struct now prints double gates, triple gates and custom
  gates correctly, in addition to adjusting to the circuit diagram for
  variable length of the gate names.

## 0.2.1 - Fixed `Circuit::repeat_measurement`

Fix:

- The method `Circuit::repeat_measurement` now works as expected. In
  release 0.2.0, this method only returned an empty hash map.

## 0.2.0 - Input validation and isolating multi-controlled gates

Features:

- Gates with multiple control nodes (multi-control gates) are now
  automatically pushed to columns so that they are isolated. This
  automatic pushing occurs when the user attempts to add several
  multi-control gates at the same time via the methods that append gates
  to the circuit. 
- The documentation has been re-shuffled for the `circuit` module, where
  the methods are now in order of when they would be used in a
  simulation of a circuit. 
- Compiled with Rust 1.72.1.

Breaking changes:

- `Circuit::new` now returns `Result<Circuit,QuantrError>`, instead of
  just a `Circuit` struct.
- A circuit now has an upper bound of 50 qubits (although, much less is
  recommended due to incomplete optimisations).
- The following methods have now been made unavailable to the user:
    - `ProductStates::insert_qubits`
    - `ProductStates::num_qubits`
    - `ProductStates::get`
    - `ProductStates::comp_basis`
    - `ProductStates::binary_basis`
- The following methods have been removed:
    - `SuperPosition::as_hash_string`
- The following methods have been renamed:
    - `SuperPosition::as_hash` -> `States::as_hash_map`
    - `SuperPosition::get_amp_from_pos` -> `States::get_amplitude`
    - `SuperPosition::get_amp_from_state` ->
      `States::get_amplitude_from_state`
- `QuantrError` is no longer accessible to the user.

Tests:

- Unit tests for adding multiple multi-control gates.
- Unit tests for user validation of various methods.

## 0.1.2 - CZ and Swap gate confusion

Fixes:

- There was confusion in thinking that CZ and Swap gates were the same
  (probably due to the similar notation in circuit diagrams). This has
  now been corrected in the documentation of the code and quick start
  guide.
- The swap gate was incorrectly defined, there was a negative sign in
  the mapping of the |11> state. Now, the swap gate has the correct
  definition.

Tests:

- An extra unit tests that now verifies the mappings of the swap and CZ
  gates, in addition to acknowledging that they're different.

## 0.1.1 - Reviewed README.md and QUICK_START.md

Fixes:

- Reviewed both documents mentioned in title, correcting spelling errors
  and sentences that didn't flow well.
- Corrected other spelling errors in other documents.

## 0.1.0 - Initial commit

The initial commit of quantr! 

See the 
[quick start guide](QUICK_START.md) to get started with quantr.

