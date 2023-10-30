# Changelog

This file logs the versions of quantr.

## 0.2.5 - Algorithm optimisations and ASCII warnings

Fixes:

- T and S conjugate gates now have ASCII names when printed in a circuit
  diagram. Before hand, this would have potentially ruined circuit
  diagrams.
- A warning has now been added when ASCII strings are used to label
  custom functions.

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

- The labelling of the toffoli gate by the printer has changed from 'To'
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

