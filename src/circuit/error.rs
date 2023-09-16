/*
* Copyright (c) 2023 Andrew Rowan Barlow. Licensed under the EUPL-1.2
* or later. You may obtain a copy of the licence at
* https://joinup.ec.europa.eu/collection/eupl/eupl-text-eupl-12. A copy
* of the EUPL-1.2 licence in English is given in LICENCE.txt which is
* found in the root directory of this repository.
*
* Author: Andrew Rowan Barlow <a.barlow.dev@gmail.com>
*/

//! Custom errors that result from the incorrect use of the quantr library.

use std::error::Error;
use std::fmt;

/// Relays error messages resulting from quantr.
pub struct QuantrError {
    pub message: String,
}

impl fmt::Display for QuantrError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "\x1b[91m[Quantr Error] {}\x1b[0m ", self.message)
    }
}

impl fmt::Debug for QuantrError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        fmt::Display::fmt(&self, f)
    }
}

impl Error for QuantrError {}

// Below is the idea to construct errors by different structs. Even though this cleans up the code
// in the fucntions themselves, hard to return different errors that correspond to different
// structs.
/*
#[macro_export]
macro_rules! implErrorLifetime {
    ($struct_name:ident, $format_string:literal, $($field:ident),*) => {
        impl fmt::Display for $struct_name<'_> {
            fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
                write!(f, $format_string, $($field = self.$field),*)
            }
        }

        impl fmt::Debug for $struct_name<'_> {
            fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
                fmt::Display::fmt(&self, f)
            }
        }

        impl<'a> Error for $struct_name<'a> {}
    };
}

#[macro_export]
macro_rules! implError {
    ($struct_name:ident, $format_string:literal, $($field:ident),*) => {
        impl fmt::Display for $struct_name {
            fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
                write!(f, $format_string, $($field = self.$field),*)
            }
        }

        impl fmt::Debug for $struct_name {
            fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
                fmt::Display::fmt(&self, f)
            }
        }

        impl Error for $struct_name {}
    };
}

pub struct AddingGateError<'a> {
    pub gate: StandardGate<'a>,
    pub wire_num: usize,
    pub num_qubits: usize,
}
implErrorLifetime!(AddingGateError, "The {:?} gate failed to be added to wire {} of a circuit with {} qubits.", gate, wire_num, num_qubits);

pub struct CompletionError {
    pub num_gates: usize,
    pub num_qubits: usize,
}
implError!(CompletionError, "The number of gates, {}, does not match the number of wires, {}. All wires must have gates added.", num_gates, num_qubits);

pub struct RepetitionError {
    pub wire_num: usize,
}
implError!(RepetitionError, "Attempted to add more than one gate to wire {}.", wire_num);
*/
