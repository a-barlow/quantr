/*
* Copyright (c) 2023 Andrew Rowan Barlow. Licensed under the EUPL-1.2
* or later. You may obtain a copy of the licence at
* https://joinup.ec.europa.eu/collection/eupl/eupl-text-eupl-12. A copy
* of the EUPL-1.2 licence in English is given in LICENCE.txt which is
* found in the root directory of this repository.
*
* Author: Andrew Rowan Barlow <a.barlow.dev@gmail.com>
*/

#![doc = include_str!("../README.md")]

mod circuit;
mod complex;
mod error;

// Make available for public use.
pub use circuit::gate::Gate;
pub use circuit::printer::Printer;
pub use circuit::states;
pub use circuit::{Circuit, Measurement};
pub use complex::{Complex, COMPLEX_ZERO};
pub use error::QuantrError;
