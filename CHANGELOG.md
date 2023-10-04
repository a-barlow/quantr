# Changelog

This file logs the versions of quantr.

## 0.2.1 - Fixed `Circuit::repeat_measurement`

Fix:

- The method `Circuit::repeat_measurement` now works as expected. In
  release 0.2.0, this method only returned an empty hash map.

## 0.2.0 - Input validation and isolating multi-controlled gates

Features:

- Gates with multiple control nodes are now automatically pushed to
  columns where they are isolated. This automatic pushing occurs when
  the user attempts to add several multi-control node gates at the same
  time via the methods that append gates to the circuit.
- All methods and initialisations of structs return a 
  `Result<_,QuantError>`.
- The documentation has been re-shuffled for the `circuit` module, where
  they are now ordered in such a way that they are most likely to be
  used.
- Compiled with Rust 1.72.1.

Breaking changes:

- `Circuit::new` now returns `Result<Circuit,QuantrError>`, instead of
  just a `Circuit` struct.
- A circuit now has an upper bound of 50 qubits (although, much less is
  recommended due to incomplete optimisations).
- The following methods have now been made unavailable to the user:
    - `States::insert_qubits`
    - `States::num_qubits`
    - `States::get`
    - `States::comp_basis`
    - `States::binary_basis`
- The following methods have been removed:
    - `States::as_hash_string`
- The following methods have been renamed:
    - `States::as_hash` -> `States::as_hash_map`
    - `States::get_amp_from_pos` -> `States::get_amplitude`
    - `States::get_amp_from_state` -> `States::get_amplitude_from_state`
- `QuantrError` is no longer accessible nor referenced in documentation.

Additions:

- Unit tests for adding multiple gates with multiple control nodes.

## 0.1.2 - CZ and Swap gate confusion

Fixes:

- There was confusion in thinking that CZ and Swap gates were the same
  (probably due to the similar notation in circuit diagrams). This has
  now been corrected in the documentation of the code and quick start
  guide.
- The swap gate was incorrectly defined, there was a negative sign in
  the mapping of the |11> state. Now, the swap gate has the correct
  definition.

Additions:

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

