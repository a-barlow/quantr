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
