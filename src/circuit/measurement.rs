/*
* Copyright (c) 2024 Andrew Rowan Barlow. Licensed under the EUPL-1.2
* or later. You may obtain a copy of the licence at
* https://joinup.ec.europa.eu/collection/eupl/eupl-text-eupl-12. A copy
* of the EUPL-1.2 licence in English is given in LICENCE.txt which is
* found in the root directory of this repository.
*
* Author: Andrew Rowan Barlow <a.barlow.dev@gmail.com>
*/

/// Distinguishes observable and non-observable quantities.
///
/// For example, this will distinguish the retrieval of a superposition (that cannot be measured
/// directly), and the state resulting from the collapse of a superposition upon measurement. See
/// [crate::Circuit::get_superposition] and [crate::Circuit::repeat_measurement] for examples.
pub enum Measurement<T> {
    Observable(T),
    NonObservable(T),
}

impl<T> Measurement<T> {
    pub fn take(self) -> T {
        match self {
            Self::Observable(item) => item,
            Self::NonObservable(item) => item,
        }
    }
}
