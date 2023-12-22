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
//! Simple implementation with operations that are needed for simulating the a quantum circuit. Quantr will
//! mostly use `Complex<f64>`, so additional functionality is added for this type, such as square
//! roots and multiplication with `f64`.

use std::fmt;
use std::fmt::{Debug, Formatter};
use std::ops::{Add, Mul, Sub};

/// The zero complex number, 0+0i.
pub const COMPLEX_ZERO: Complex<f64> = Complex::<f64> { re: 0f64, im: 0f64 };

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

/// Generic complex number.
#[derive(Debug, PartialEq, Copy, Clone)]
pub struct Complex<T> {
    pub re: T,
    pub im: T,
}

impl Complex<f64> {
    /// Returns `exp(i*theta)` as a complex number.
    ///
    /// # Example
    /// ```
    /// use quantr::Complex;
    /// use quantr::complex_im;
    /// use std::f64::consts::PI;
    ///
    /// let num: Complex<f64> = Complex::<f64>::exp_im(0.5f64 * PI);
    /// ```
    pub fn exp_im(theta: f64) -> Complex<f64> {
        Complex {
            re: theta.cos(),
            im: theta.sin(),
        }
    }
}

impl Complex<f32> {
    /// Returns `exp(i*theta)` as a complex number.
    ///
    /// # Example
    /// ```
    /// use quantr::Complex;
    /// use quantr::complex_im;
    /// use std::f32::consts::PI;
    ///
    /// let num: Complex<f32> = Complex::<f32>::exp_im(0.5f32 * PI);
    /// ```
    pub fn exp_im(theta: f32) -> Complex<f32> {
        Complex {
            re: theta.cos(),
            im: theta.sin(),
        }
    }
}

/// Addition of two generic complex numbers.
impl<T: Add<Output = T>> Add for Complex<T> {
    type Output = Self;

    /// # Example
    /// ```
    /// use quantr::Complex;
    ///
    /// let z1: Complex<i16> = Complex{ re: -4i16, im: 2i16 };
    /// let z2: Complex<i16> = Complex{ re: 2i16, im: 7i16 };
    /// assert_eq!(z1 + z2, Complex{ re: -2i16, im: 9i16 });
    /// ```
    fn add(self, rhs: Self) -> Self::Output {
        Complex {
            re: self.re.add(rhs.re),
            im: self.im.add(rhs.im),
        }
    }
}

/// Subtracts two generic complex numbers.
impl<T: Sub<Output = T>> Sub for Complex<T> {
    type Output = Self;

    /// # Example
    /// ```
    /// use quantr::Complex;
    ///
    /// let z1: Complex<i16> = Complex{ re: -4i16, im: 2i16 };
    /// let z2: Complex<i16> = Complex{ re: 2i16, im: 7i16 };
    /// assert_eq!(z1 - z2, Complex{ re: -6i16, im: -5i16 });
    /// ```
    fn sub(self, rhs: Self) -> Self::Output {
        Complex {
            re: self.re.sub(rhs.re),
            im: self.im.sub(rhs.im),
        }
    }
}

/// Multiplying two generic complex numbers.
impl<T: Mul<Output = T> + Add<Output = T> + Sub<Output = T> + Copy> Mul for Complex<T> {
    type Output = Self;

    /// # Example
    /// ```
    /// use quantr::Complex;
    ///
    /// let z1: Complex<i16> = Complex{ re: -4i16, im: 2i16 };
    /// let z2: Complex<i16> = Complex{ re: 2i16, im: 7i16 };
    /// assert_eq!(z1 * z2, Complex{ re: -22i16, im: -24i16 });
    /// ```
    fn mul(self, rhs: Self) -> Self::Output {
        Complex {
            re: self.re.mul(rhs.re).sub(self.im.mul(rhs.im)),
            im: self.re.mul(rhs.im).add(self.im.mul(rhs.re)),
        }
    }
}

impl Mul<f64> for Complex<f64> {
    type Output = Self;

    /// LHS scalar multiplication: `f64 * Complex<f64>`.
    ///
    /// # Example
    /// ```
    /// use quantr::Complex;
    ///
    /// let z1: f64 = 2f64;
    /// let z2: Complex<f64> = Complex{ re: 2f64, im: 7f64 };
    /// assert_eq!(z1 * z2, Complex{ re: 4f64, im: 14f64 });
    /// ```
    fn mul(self, rhs: f64) -> Self::Output {
        Complex {
            re: self.re.mul(rhs),
            im: self.im.mul(rhs),
        }
    }
}

/// Multiplication `Complex<f64> * f64`.
impl Mul<Complex<f64>> for f64 {
    type Output = Complex<f64>;

    /// RHS scalar multiplication: `Complex<f64> * f64`.
    ///
    /// # Example
    /// ```
    /// use quantr::Complex;
    ///
    /// let z1: f64 = 2f64;
    /// let z2: Complex<f64> = Complex{ re: 2f64, im: 7f64 };
    /// assert_eq!(z2 * z1, Complex{ re: 4f64, im: 14f64 });
    /// ```
    fn mul(self, rhs: Complex<f64>) -> Self::Output {
        Complex {
            re: rhs.re.mul(self),
            im: rhs.im.mul(self),
        }
    }
}

impl<T: Add<Output = T> + Mul<Output = T> + Copy> Complex<T> {
    /// Absolute square of a complex number, that is `|z|^2 = a^2+b^2`
    /// where `z = a + bi`.
    ///
    /// # Example
    /// ```
    /// use quantr::Complex;
    ///
    /// let z: Complex<i16> = Complex{ re: 3i16, im: 4i16 };
    /// assert_eq!(z.abs_square(), 25i16);
    /// ```
    pub fn abs_square(self) -> T {
        self.re.mul(self.re).add(self.im.mul(self.im))
    }
}

impl<T: Add<Output = T> + Mul<Output = T> + Sqr + Copy> Complex<T> {
    /// Absolute value of a complex number, that is
    /// `|z| = Sqrt(a^2+b^2)` where `z = a + bi`.
    ///
    /// # Example
    /// ```
    /// use quantr::Complex;
    ///
    /// let z: Complex<f64> = Complex{ re: 3f64, im: 4f64 };
    /// assert_eq!(z.abs(), 5f64);
    /// ```
    pub fn abs(&self) -> T {
        self.abs_square().square_root()
    }
}

impl<T: fmt::Display> fmt::Display for Complex<T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{} + {}i", self.re, self.im)
    }
}

/// Usage: `complex!(re: f64, im: f64) -> Complex<f64>`
/// A quick way to define a f64 complex number.
#[macro_export]
macro_rules! complex {
    ($r:expr, $i:expr) => {
        Complex::<f64> { re: $r, im: $i }
    };
}

/// Usage: `complex_re_array!(input: [f64; n]) -> [Complex<f64>; n]`
/// Returns an array of complex numbers with zero imaginary part, and the real part set by `input`.
#[macro_export]
macro_rules! complex_re_array {
    ( $( $x:expr ),*  ) => {
        [
        $(
            Complex::<f64> {
                re: $x,
                im: 0f64
            }
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
            Complex::<f64> {
                re: 0f64,
                im: $x
            }
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
                    Complex::<f64> {
                        re: $x,
                        im: 0f64
                    }
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
                    Complex::<f64> {
                        re: 0f64,
                        im: $x,
                    }
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
        Complex::<f64> { re: $r, im: 0f64 }
    };
}

/// Usage: `complex_im!(im: f64) -> Complex<f64>`
/// A quick way to define an imaginary f64; the real part is set to zero.
#[macro_export]
macro_rules! complex_im {
    ($i:expr) => {
        Complex::<f64> { re: 0f64, im: $i }
    };
}

#[rustfmt::skip]
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn complex_imaginary_and_imaginary() {
        let num_one = Complex::<i8>{ re: 0, im: 9};
        let num_two = Complex::<i8>{ re: 0, im: 2};

        assert_eq!(num_one.mul(num_two), Complex::<i8>{ re: -18, im: 0})
    }

    #[test]
    fn complex_imaginary_and_real() {
        let num_one = Complex::<i8>{ re: 0, im: -9};
        let num_two = Complex::<i8>{ re: 2, im: 0};

        assert_eq!(num_one.mul(num_two), Complex::<i8>{ re: 0, im: -18})
    }

    #[test]
    fn complex_multiply() {
        let num_one = Complex::<i8>{ re: 2, im: 9};
        let num_two = Complex::<i8>{ re: 7, im: 3};

        assert_eq!(num_one.mul(num_two), Complex::<i8>{ re: -13, im: 69})
    }

    #[test]
    fn complex_add() {
        let num_one = Complex::<i8>{ re: 2, im: 9};
        let num_two = Complex::<i8>{ re: 7, im: -3};

        assert_eq!(num_one.add(num_two), Complex::<i8>{ re: 9, im: 6})
    }

    #[test]
    fn complex_sub() {
        let num_one = Complex::<i8>{ re: 2, im: 9};
        let num_two = Complex::<i8>{ re: 7, im: -3};

        assert_eq!(num_one.sub(num_two), Complex::<i8>{ re: -5, im: 12})
    }

    #[test]
    fn complex_abs() {
        let num_one = Complex::<f32>{ re: 2f32, im: 9f32};

        assert_eq!(num_one.abs_square(), 85f32)
    }

    #[test]
    fn complex_abs_square_root() {
        let num_one = Complex::<f32>{ re: 2f32, im: 9f32};

        assert_eq!(num_one.abs(), 85f32.sqrt())
    }

    #[test]
    fn scale_a_complex_number_with_f64_rhs() {
        let num_one = Complex::<f64>{ re: 2f64, im: 9f64};

        assert_eq!(5f64 * num_one, Complex::<f64>{ re: 10f64, im: 45f64})
    }

    #[test]
    fn scale_a_complex_number_with_f64_lhs() {
        let num_one = Complex::<f64>{ re: 2f64, im: 9f64};

        assert_eq!(num_one * 5f64, Complex::<f64>{ re: 10f64, im: 45f64})
    }
}
