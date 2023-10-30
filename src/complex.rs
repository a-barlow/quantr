/*
* Copyright (c) 2023 Andrew Rowan Barlow. Licensed under the EUPL-1.2
* or later. You may obtain a copy of the licence at
* https://joinup.ec.europa.eu/collection/eupl/eupl-text-eupl-12. A copy
* of the EUPL-1.2 licence in English is given in LICENCE.txt which is
* found in the root directory of this repository.
*
* Author: Andrew Rowan Barlow <a.barlow.dev@gmail.com>
*/

//! Generic complex numbers.
//!
//! Simple implementation with operations that are needed for the quantum computer. Quantr will
//! mostly use `Complex<f64>`, so additional functionality is added for this type, such as square
//! roots and multiplication with `f64`.

use std::fmt;
use std::fmt::{Debug, Formatter};
use std::ops::{Add, Mul, Sub};

/// A square root trait, that is only implemented for `f32` and `f64` as Sqrt is not a closed
/// operation for int, uint, etc. This is needed for the absolute value of a complex number.
pub trait Sqr {
    fn square_root(self) -> Self;
}

impl Sqr for f32 {
    fn square_root(self) -> Self {
        self.sqrt()
    }
}

impl Sqr for f64 {
    fn square_root(self) -> Self {
        self.sqrt()
    }
}

/// Generic complex number for the quantum computer. Will mostly use `f64`.
#[derive(Debug, PartialEq, Copy, Clone)]
pub struct Complex<T> {
    pub real: T,
    pub imaginary: T,
}

impl Complex<f64> {
    /// Returns `exp(i*theta)` as a complex number.
    pub fn expi(theta: f64) -> Complex<f64> {
        Complex { 
            real: theta.cos(), 
            imaginary: theta.sin(),
        }
    }
}

impl Complex<f32> {
    /// Returns `exp(i*theta)` as a complex number.
    pub fn expi(theta: f32) -> Complex<f32> {
        Complex { 
            real: theta.cos(), 
            imaginary: theta.sin(),
        }
    }
}

/// Addition of two generic complex numbers.
impl<T: Add<Output = T>> Add for Complex<T> {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        Complex {
            real: self.real.add(rhs.real),
            imaginary: self.imaginary.add(rhs.imaginary),
        }
    }
}

/// Subtracts two generic complex numbers.
impl<T: Sub<Output = T>> Sub for Complex<T> {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self::Output {
        Complex {
            real: self.real.sub(rhs.real),
            imaginary: self.imaginary.sub(rhs.imaginary),
        }
    }
}

/// Multiplying two generic complex numbers.
impl<T: Mul<Output = T> + Add<Output = T> + Sub<Output = T> + Copy> Mul for Complex<T> {
    type Output = Self;

    fn mul(self, rhs: Self) -> Self::Output {
        Complex {
            real: self
                .real
                .mul(rhs.real)
                .sub(self.imaginary.mul(rhs.imaginary)),
            imaginary: self
                .real
                .mul(rhs.imaginary)
                .add(self.imaginary.mul(rhs.real)),
        }
    }
}

/// Multiplication of `f64 * Complex<f64>`.
impl Mul<f64> for Complex<f64> {
    type Output = Self;

    fn mul(self, rhs: f64) -> Self::Output {
        Complex {
            real: self.real.mul(rhs),
            imaginary: self.imaginary.mul(rhs),
        }
    }
}

/// Multiplication `Complex<f64> * f64`.
impl Mul<Complex<f64>> for f64 {
    type Output = Complex<f64>;

    fn mul(self, rhs: Complex<f64>) -> Self::Output {
        Complex {
            real: rhs.real.mul(self),
            imaginary: rhs.imaginary.mul(self),
        }
    }
}

impl<T: Add<Output = T> + Mul<Output = T> + Copy> Complex<T> {
    /// Absolute square of a complex number, that is `|z|^2 = a^2+b^2`
    /// where `z = a + bi`.
    pub fn abs_square(self) -> T {
        self.real
            .mul(self.real)
            .add(self.imaginary.mul(self.imaginary))
    }
}

impl<T: Add<Output = T> + Mul<Output = T> + Sqr + Copy> Complex<T> {
    /// Absolute value of a complex number, that is
    /// `|z| = Sqrt(a^2+b^2)` where `z = a + bi`.
    pub fn abs(&self) -> T {
        self.abs_square().square_root()
    }
}

impl<T: fmt::Display> fmt::Display for Complex<T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{} + {}i", self.real, self.imaginary)
    }
}

/// Shortcut for `complex!(0f64, 0f64)`.
#[macro_export]
macro_rules! complex_zero {
    () => {
        Complex::<f64> {
            real: 0f64,
            imaginary: 0f64,
        }
    };
}

/// Usage: `complex!(real: f64, imaginary: f64) -> Complex<f64>`
/// A quick way to define a f64 complex number.
#[macro_export]
macro_rules! complex {
    ($r:expr, $i:expr) => {
        Complex::<f64> {
            real: $r,
            imaginary: $i,
        }
    };
}

/// Usage: `complex_Re_array!(input: [f64; n]) -> [Complex<f64>; n]`
/// Returns an array of complex number with zero imaginary part, and reals set by `input`.
#[macro_export]
macro_rules! complex_Re_array {
    ( $( $x:expr ),*  ) => {
        [
        $(
            Complex::<f64> {
                real: $x,
                imaginary: 0f64
            }
        ),*
        ]
    };
}

/// Usage: `complex_Im_array!(input: [f64; n]) -> [Complex<f64>; n]`
/// Returns an array of complex number with zero real part, and imaginary set by `input`.
#[macro_export]
macro_rules! complex_Im_array {
    ( $( $x:expr ),*  ) => {
        [
        $(
            Complex::<f64> {
                real: 0f64,
                imaginary: $x
            }
        ),*
        ]
    };
}

/// Usage: `complex_Re_vec!(input: [f64; n]) -> Vec<Complex<f64>>`
/// Returns a vector of complex number with zero imaginary part, and reals set by `input`.
#[macro_export]
macro_rules! complex_Re_vec {
    ( $( $x:expr ),*  ) => {
        {
            let mut temp_vec: Vec<Complex<f64>> = Vec::new();
            $(
                temp_vec.push(
                    Complex::<f64> {
                        real: $x,
                        imaginary: 0f64
                    }
                );
            )*
            temp_vec
        }
    };
}

/// Usage: `complex_Im_vec!(input: [f64; n]) -> Vec<Complex<f64>>`
/// Returns a vector of complex numbers with zero real part, and imaginaries set by `input`.
#[macro_export]
macro_rules! complex_Im_vec {
    ( $( $x:expr ),*  ) => {
        {
            let mut temp_vec: Vec<Complex<f64>> = Vec::new();
            $(
                temp_vec.push(
                    Complex::<f64> {
                        real: 0f64,
                        imaginary: $x,
                    }
                );
            )*
            temp_vec
        }
    };
}

/// Usage: `complex_Re!(real: f64) -> Complex<f64>`
/// A quick way to define a real f64; the imaginary part is set to zero.
#[macro_export]
macro_rules! complex_Re {
    ($r:expr) => {
        Complex::<f64> {
            real: $r,
            imaginary: 0f64,
        }
    };
}

/// Usage: `complex_Im!(imaginary: f64) -> Complex<f64>`
/// A quick way to define an imaginary f64; the real part is set to zero.
#[macro_export]
macro_rules! complex_Im {
    ($i:expr) => {
        Complex::<f64> {
            real: 0f64,
            imaginary: $i,
        }
    };
}

#[rustfmt::skip]
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn complex_imaginary_and_imaginary() {
        let num_one = Complex::<i8>{real: 0, imaginary: 9};
        let num_two = Complex::<i8>{real: 0, imaginary: 2};

        assert_eq!(num_one.mul(num_two), Complex::<i8>{real: -18, imaginary: 0})
    }

    #[test]
    fn complex_imaginary_and_real() {
        let num_one = Complex::<i8>{real: 0, imaginary: -9};
        let num_two = Complex::<i8>{real: 2, imaginary: 0};

        assert_eq!(num_one.mul(num_two), Complex::<i8>{real: 0, imaginary: -18})
    }

    #[test]
    fn complex_multiply() {
        let num_one = Complex::<i8>{real: 2, imaginary: 9};
        let num_two = Complex::<i8>{real: 7, imaginary: 3};

        assert_eq!(num_one.mul(num_two), Complex::<i8>{real: -13, imaginary: 69})
    }

    #[test]
    fn complex_add() {
        let num_one = Complex::<i8>{real: 2, imaginary: 9};
        let num_two = Complex::<i8>{real: 7, imaginary: -3};

        assert_eq!(num_one.add(num_two), Complex::<i8>{real: 9, imaginary: 6})
    }

    #[test]
    fn complex_sub() {
        let num_one = Complex::<i8>{real: 2, imaginary: 9};
        let num_two = Complex::<i8>{real: 7, imaginary: -3};

        assert_eq!(num_one.sub(num_two), Complex::<i8>{real: -5, imaginary: 12})
    }

    #[test]
    fn complex_abs() {
        let num_one = Complex::<f32>{real: 2f32, imaginary: 9f32};

        assert_eq!(num_one.abs_square(), 85f32)
    }

    #[test]
    fn complex_abs_square_root() {
        let num_one = Complex::<f32>{real: 2f32, imaginary: 9f32};

        assert_eq!(num_one.abs(), 85f32.sqrt())
    }

    #[test]
    fn scale_a_complex_number_with_f64_rhs() {
        let num_one = Complex::<f64>{real: 2f64, imaginary: 9f64};

        assert_eq!(5f64 * num_one, Complex::<f64>{real: 10f64, imaginary: 45f64})
    }

    #[test]
    fn scale_a_complex_number_with_f64_lhs() {
        let num_one = Complex::<f64>{real: 2f64, imaginary: 9f64};

        assert_eq!(num_one * 5f64, Complex::<f64>{real: 10f64, imaginary: 45f64})
    }
}
