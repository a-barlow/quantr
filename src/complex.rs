/*
* Copyright (c) 2024 Andrew Rowan Barlow. Licensed under the EUPL-1.2
* or later. You may obtain a copy of the licence at
* https://joinup.ec.europa.eu/collection/eupl/eupl-text-eupl-12. A copy
* of the EUPL-1.2 licence in English is given in LICENCE.txt which is
* found in the root directory of this repository.
*
* Author: Andrew Rowan Barlow <a.barlow.dev@gmail.com>
*/

//! Macros for the num_complex crate.

/// Usage: `complex_re_array!(input: [f64; n]) -> [Complex<f64>; n]`
/// Returns an array of complex numbers with zero imaginary part, and the real part set by `input`.
#[macro_export]
macro_rules! complex_re_array {
    ( $( $x:expr ),*  ) => {
        [
        $(
            $crate::num_complex::Complex64 {re: $x, im: 0f64}
        ),*
        ]
    };
}

/// Usage: `complex_im_array!(input: [f64; n]) -> [Complex<f64>; n]`
/// Returns an array of complex number with zero real part, and imaginaries set by `input`.
#[macro_export]
macro_rules! complex_im_array {
    ( $( $x:expr ),*  ) => {
        [
        $(
            $crate::num_complex::Complex64 {re: 0f64, im: $x}
        ),*
        ]
    };
}

/// Usage: `complex_re_vec!(input: [f64; n]) -> Vec<Complex<f64>>`
/// Returns a vector of complex number with zero imaginary part, and reals set by `input`.
#[macro_export]
macro_rules! complex_re_vec {
    ( $( $x:expr ),*  ) => {
        {
            let mut temp_vec: Vec<Complex<f64>> = Vec::new();
            $(
                temp_vec.push(
                    $crate::num_complex::Complex64 { re: $x, im: 0f64 }
                );
            )*
            temp_vec
        }
    };
}

/// Usage: `complex_im_vec!(input: [f64; n]) -> Vec<Complex<f64>>`
/// Returns a vector of complex numbers with zero real part, and imaginaries set by `input`.
#[macro_export]
macro_rules! complex_im_vec {
    ( $( $x:expr ),*  ) => {
        {
            let mut temp_vec: Vec<Complex<f64>> = Vec::new();
            $(
                temp_vec.push(
                    $crate::num_complex::Complex64 { re: 0f64, im: $x }
                );
            )*
            temp_vec
        }
    };
}

/// Usage: `complex_re!(re: f64) -> Complex<f64>`
/// A quick way to define a real f64; the imaginary part is set to zero.
#[macro_export]
macro_rules! complex_re {
    ($r:expr) => {
        $crate::num_complex::Complex64 { re: $r, im: 0f64 }
    };
}

/// Usage: `complex_im!(im: f64) -> Complex<f64>`
/// A quick way to define an imaginary f64; the real part is set to zero.
#[macro_export]
macro_rules! complex_im {
    ($i:expr) => {
        $crate::num_complex::Complex64 { re: 0f64, im: $i }
    };
}
