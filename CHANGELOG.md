# Changelog

This file logs the versions of quantr.

## 0.6.0 - Overhaul of Interface

The interface is being overhauled to increase the safety in using
quantr. Perhaps the most noticeable difference is that when a circuit is
simulated with `Circuit::simulate`, it will now return a struct instead
of a superposition. This struct, called `SimulatedCircuit`, can
guarantee that the circuit has been simulated, and when requested, the
resulting superposition can be measured repeatedly. It can also
re-simulate the circuit (as true to reality), if there is an insertion
of a `Custom::gate` that implements a non-unitary operation. With
`SimulatedCircuit`, it can now be guaranteed that the
circuit has been simulated and ready to be measured. Unlike previously
where one could attempt to retrieve the superposition, but would get an
error stating that the circuit has not been simulated yet.

These implementations are heading toward a final 
design that we are happy with. Of course, this library will forever be 
evolving but the design implemented now should be kept for the 
foreseeable future; minimising breaking changes.

Breaking changes:

- `Circuit::simulate` now returns `SimulatedCircuit`, and will consume
the circuit (`SimulatedCircuit` takes ownership).
- All functions that return `Result<T, QuantrError>` for some type
`T` are:
  - `Circuit::new`
  - `Circuit::add_gate`
  - `Circuit::add_gates`
  - `Circuit::add_gates_with_positions`
  - `Circuit::add_repeating_gate`
  - `Circuit::change_register`
  - `states::ProductState::invert_digit`
  - `states::ProductState::new`
  - `states::SuperPosition::new`
  - `states::SuperPosition::new_with_amplitudes`
  - `states::SuperPosition::get_amplitude_from_state`
  - `states::SuperPosition::set_amplitudes`
  - `states::SuperPosition::set_amplitudes_from_state`
- The `Gate::custom` enum now takes a `Vec<uisze>` instead of a slice for
  the qubit indices that the gate should operate on. Now, the circuit
can outlive the slice.
The following methods have been replaced:
  - `Circuit::repeat_measurement` with `SimulatedCirucit::measure_all`.
  - `Circuit::get_superposition` with `SimulatedCirucit::get_state`.
  - `Circuit::toggle_simulation_progress` with 
  `SimulatedCirucit::print_progress` and `Circuit::set_print_progress`

Features:

- A new struct `SimulatedCirucit` that holds information about the
simulated circuit, and can guarantee a superposition that can be
measured.
- Re-introduced `QuantrError`, which as originally introduced for,
will hold error messages that result from the incorrect use of quantr.
It implements the error trait, and so can be handled through that, or
directly through `QuantrError`.
- The `Measurement` enum now has a `take` method, which moves out the
value that it wraps.
- All printed '[Quantr Warnings]' are now sent to stderr, instead of
stdout.
- Added `Circuit::clone_and_simulate` simulates the circuit, but does not
consume the circuit in the process. The circuit is instead borrowed,
and its register cloned. This will lead to an increase in memory
consumption.
- Added `Superposition::measure` which returns simulates the effect of 
measuring a superposition, and observing the state that it reduced to.
- `From<Qubit>` was implemented for `Superposition`.
- `Printer::set_print_warnings` was added that allows the user to turn warnings
from the printer off and on.
- `SuperPosition::new_with_amplitudes_unchecked` and 
`SuperPosition::set_amplitudes_unchecked` are two new methods that allow for
the implementation of _some_ quantum channels, as they do not enforce
probability conservation of the state vector. That act in an equivalent way
to their 'checked' counterparts.

Internal improvements:

- Attaching a custom register is more memory efficient; it use to
create another vector, in addition to the circuit's state vector, in
effect doubling the necessary memory.
- Removed `QuantrErrorConst`.
- The Iterator trait implementations for `ProductState` and the
`Measurment` enum have moved to their own files.  
- Bumped Rust version to 1.79.0

## 0.5.2 - Review of docs and minor code improvements 

The README was updated to give credit to another quantum circuit
simulator implemented in Rust:
[Spinoza](https://github.com/QuState/spinoza). Although yet used by the
current authors, the Spinoza project is more developed and mature than
quantr, and looks to be a good Rust alternative!

The LICENSES were updated to reflect the use of the
[fastrand](https://crates.io/crates/fastrand.) crate that is a
dependency of quantr. This was implemented in update 0.4.1; although
this wasn't stated in the licensing at the time, the fastrand crate was
implicitly being used under the MIT licence. 

Several unnecessary calls of `ProductState::to_string` through it's
trait implementation were made in printing macros, and are now removed.

Features:

  - The `IntoIter` trait was implemented for `ProductState`, which
  produces an iterator of the qubits that represent the state, from left
  to right in braket notation. 
  - `Circuit::get_toggle_progress` has been added, which returns if the 
  circuit has been set to print explicit simulation progress, set by 
  `Circuit::toggle_simulation_progress`.

## 0.5.1 - Review of docs and deprecated const functions

See the previous update, 0.5.0, for why some functions were promoted to
a const (TL;DR it was a mistake). These have been marked `deprecate` so
the rust compiler warns users to not use them in a const context.

The quick start guide has been updated for 0.5.0 use of quantr, where
the errors that functions return have now been made private, forcing the
user to handle them through their `std::error::Error` trait.

## 0.5.0 - Finalising interface

Following this update, interfacing with quantr can now be done safely.
All assumptions that are needed for the safe simulation of the circuit
can now be upheld, no matter how the user interfaces with this library.
Of course, any incorrect interfacing should result in an error. In this
update, that meant making the final public fields private for the
`Circuit` struct. 

Some functions have promoted to constant functions. Although this was
not a breaking change in itself, it meant that removing such constraints
constitutes as a breaking change. Moreover, I had not fully understood
it's use in Rust. So, this will be removed for some functions, if not
all. This will be most likely removed in the next major update. Of
course, it is the developers wishes to minimise these breaking changes.
- A. Barlow

Breaking changes:

- Changed return type of `states::super_positions::get_amplitude ->
  Result<Complex<f64>, QuantrError>` to
  `states::super_positions::get_amplitude -> Option<Complex<f64>>`.
- All fields of `Circuit` are now private; that is `num_qubits` and
  `circuit_gates`. These two can still be accessed through
  `Circuit::get_num_qubits` and `Circuit::get_gates` respectively.
- The argument of `Circuit::get_num_qubits` now only borrows the
  circuit, instead of consuming it (which was a mistake in the 0.4.1
  release).
- Removed `QuantrError` from the public interface. Now, it has to be
  used through it's trait `std::error::Error`. See `examples` and the
  `main()` function return type. 
- The following functions have changed their returning error type to
  `QuantrErrorConst`:
  - `Circuit::new`
  - `Circuit::get_superposition`
  - `Circuit::repeat_measurement`
  - `states::SuperPosition::new`
  - `states::SuperPosition::new_with_amplitudes`
  - `states::ProductState::new`

Constant functions:

The following functions have been made constant.

  - `Circuit::new`
  - `Circuit::get_superposition`
  - `Circuit::get_num_qubits`
  - `SuperPosition::get_num_qubits`

Internal improvements:

- Added `QuantrErrorConst` that consumes a `&str`. This can be used for
  constant strings (error messages) and so enables some functions to
  become constant.

## 0.4.1 - More optimisations

Edited the README to include "No parallelisation" to limitations, and
reduced the tractable number of qubit simulations to 18. There has also
been a large overhaul of the code to increase maintainability. Some
common mistakes were also fixed with the help of `cargo clippy`.

Features:

- `Circuit::get_num_qubits`, this is to replace the `num_qubits` field
  that will be made private in the next major update. However, the
  argument consumes the circuit which was a mistake. This will be fixed
  in the next major update too. This returns the number of qubits of the
  circuit.
- `Circuit::get_gates`, returns the vector of gates that represent the
  quantum circuit. This will replace the `circuit_gates` field.

Change of dependency:

- The `rand` crate has been swapped with `fastrand` which decreases
  compilation time.

Optimisations:

- The definition of the gates in `standard_gate_ops.rs` have had there
  arguments changed so that the `kronecker_prod` is not used; increasing
  speed for multi gate processing.
- The main simulating algorithm has been updated to increase it's speed,
  mostly bypassing computations that are unneeded, for instance product
  state qubits are flipped only if they are indeed different.

Deprecated features:

- The public fields of `Circuit` are to be made private (specifically
  updated to `pub(crate)` status in the next breaking update).

## 0.4.0 - Optimisations of speed and memory allocation

The optimisations and breaking changes that this update induces greatly
increases the speed of simulating circuits. Even though it's generally
discouraged to make breaking changes without deprecation warnings, or
even so suddenly after another breaking change (0.3.0), this
optimisation has been deemed beneficial enough to warrant such a soon
breaking update. Moreover, quantr is still in it's infancy, where
nobody, or very few people, are currently using it. 

The main difference is in how custom functions return an
`Option<SuperPosition>` object, where it returns `None` if the input
state has not been affected. This bypasses large amounts of unnecessary
computation. 

The last update is to conform to Rust protocol, where instead of using
`into_superposition` or like methods, the `From` trait is implemented
instead (which also implements the `Into` trait).

Lastly, the quantr repository will no longer follow the
Contributor Covenant Code of Conduct for moderating it's GitHub
repository. Please see `CODE_OF_CONDUCT.md` for the reason to why.

Breaking Changes:

- The `Gate::Custom` now requires a `fn (ProductState) ->
  Option<SuperPosition>` type as a function to define the custom gate.
  This function should return `None` if the input `ProductState` is
  unchanged, and `Some(SuperPosition)` if the product state has changed.
  This ultimately increases the speed of processing gates such as
  multi-controlled not gates.
- The function `SuperPosition::set_amplitudes(self, amplitudes:
  &[Complex<f64>]) -> Result<SuperPosition, QuantrError>` has changed
  its arguments so that borrows a mutable reference:
  `SuperPosition::set_amplitudes(&mut self, amplitudes: &[Complex<f64>])
  -> Result<&mut SuperPosition, QuantrError>`. The same has been done
  with `SuperPosition::set_amplitudes_from_states`.
- Changed the return value of `ProductState::invert_digit` to
  `Result<&mut ProductState, QuantrError>`.
- The conversion methods `Qubit::into_state` and
  `ProductState::into_super_position` have been replaced by the
  `ProductState::From<Qubit>` and `SuperPosition::From<ProductState>`
  trait implementations respectively. These trait implementations will
  automatically generate the `Into` traits for `Qubit` and
  `ProductState`. 
- The fields of `SuperPosition` and `ProductState` have been made
  private (inaccessible to the user). This forces the user to go
  initialise and change these structs through methods with validation
- The macros names for complex numbers are now lower case.

Examples:

- The `.unwrap()` on measurements have been removed, in favour of
  explicitly showing the `Result` return type of `Circuit::repeated_measurement`
  and `Circuit::get_superposition`.
- Added examples for implementing a controlled not gate with arbitrary
  number of control nodes. This uses generic constants. This can be
  found in `examples/generalised_control_not_gate.rs`.

Tests:

- The qft and grover tests have had their custom functions updated
  accordingly to satisfy the breaking change of custom functions.

Features:

- The upper bound for the circuit size of 50 qubits has been removed.
  Although, currently this version of quantr can reasonably simulate up
  to 20 qubits. 50 qubits would be unphysical to simulate anyway on a
  desktop due to the large amount of memory required to store the state
  vector alone.
- `SuperPosition::new_with_amplitudes` allows creation of a super
  position by defining amplitudes at the same time.
- `SuperPosition::new_with_hasp` allows creation of a super position
  based on a hash map defining states and their amplitudes. States that
  don't appear as a key will have zero amplitudes set.
- The states module has been fully documented with examples included for
  every object.
- Added the following methods to `ProductState` as its fields are now
  private (more info in documentation):
    - `get`, Returns the qubit of the given the list index. 
    - `get_qubits`, Returns the slice of qubits that label the state.
    - `get_mut_qubits`, Returns a mutable slice of qubits that label the
      state.
    - `num_qubits`, Returns the number of qubits that compose the
      product state in the computational basis.
- For the same reason as above, `SuperPosition` has the following new
  methods:
    - `get_amplitudes`, Returns a slice of amplitudes in the
      computational basis.
    - `get_dimension`, Returns the Hilbert space dimension that the
      super position exists in.
    - `get_num_qubits`, Returns the number of qubits that compose the
      product states in the computational basis.
    - `new_with_amplitudes`, Initialises a new super position based on a
      slice of amplitudes in the computational basis.
    - `new_with_hash_amplitudes`, Same as above, but uses a Hashmap as
      an argument to define the super position.
    

## 0.3.0 - Interface refresh 

This major update overhauls the structure of quantr, and the naming of
many methods. The aim is to increase simplicity in using the library,
in turn producing more readable and efficient code. The re-naming of
methods is meant to be more in keeping with the Rust standard library,
that is `to` represents a pass by reference, while `into` moves the value
into the method.

Moreover, some examples have been added showcasing custom functions and
printing the circuits in a variety of ways. 

Breaking Changes:

- The function `ProductState::new` now returns `Result<ProductState,
  QuantrError>`. An error is returned if an empty slice is given as an
  argument.
- Renamed the fields of `Complex` from `real` and `imaginary` to `re`
  and `im` respectively. 
- Removed `Circuit::simulate_with_register`. This is replaced with
  `Circuit::change_register` which can be called before simulation, to
  change the default register of |00..0> that is applied during
  simulating.
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
- Re-structured access of structs and module paths. Now, every struct is
  accessed through `quantr::...` except for those that control states,
  which are accessed through the module `quantr::states::...`.
- Changed the input type of two methods in `Circuit`:
    - `add_gates(Vec<Gate>)` -> `add_gates(&[Gate])`
    - `add_repeating_gate(Gate, Vec<usize>)` ->
      `add_repeating_gate(Gate, &[usize])`.
- `Circuit` methods that add gates now output a mutable reference to the
  mutated circuit. This allows for a 'chain of method calls' to be made.

Features:

- The `QuantrError` struct has been made public for the user (this was
  available in versions < 0.2.0). This allows for succinct error handling
  with `?` when creating circuits when the main function is allowed to
  return `Result<(), QuantrError>`.

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
- The qft example has been added as an external test.

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

