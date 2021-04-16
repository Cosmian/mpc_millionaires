// Copyright (c) 2021, COSIC-KU Leuven, Kasteelpark Arenberg 10, bus 2452, B-3001 Leuven-Heverlee, Belgium.
// Copyright (c) 2021, Cosmian Tech SAS, 53-55 rue La Boétie, Paris, France.

use crate::array::*;
use crate::bit_protocols::*;
use crate::circuits::*;
use crate::fixed_point::*;
use crate::floating_point::*;
use crate::ieee::*;
use crate::integer::*;
use crate::local_functions::*;
use crate::math_generic::*;
use core::ops::{Add, Div, Mul};
use scale::alloc::GetAllocator;
use scale::*;

/********************************/
/*             Traits           */
/********************************/

pub trait Floor {
    fn floor(self) -> Self;
}

pub trait FAbs {
    fn fabs(self) -> Self;
}

pub trait Sqrt {
    fn sqrt(self) -> Self;
}

pub trait Constants
where
    Self: From<f64>,
{
    #[inline(always)]
    fn two_pi() -> Self {
        Self::from(6.2831853071795864769252867665590057684)
    }
    #[inline(always)]
    fn pi() -> Self {
        Self::from(3.1415926535897932384626433832795028842)
    }
    #[inline(always)]
    fn half_pi() -> Self {
        Self::from(1.5707963267948966192313216916397514421)
    }
    #[inline(always)]
    fn ln2() -> Self {
        Self::from(0.69314718055994530941723212145817656807)
    }
    #[inline(always)]
    fn e() -> Self {
        Self::from(2.7182818284590452353602874713526624978)
    }
}

/* Make some supertraits to help in where clauses */

pub trait Float:
    Copy
    + From<i64>
    + From<f64>
    + Floor
    + FAbs
    + Sqrt
    + Constants
    + LoadFromMem<i64>
    + StoreInMem<i64>
    + GetAllocator
{
}

impl Float for ClearIEEE {}

impl Float for SecretIEEE {}

impl<const K: u64, const F: u64> Float for ClearFixed<K, F>
where
    ConstU64<{ K - F }>: ,
    ConstU64<{ F + 1 }>: ,
    ConstU64<{ K + 1 }>: ,
    ConstU64<{ K - 1 }>: ,
    ConstU64<{ 2 * F }>: ,
    ConstU64<{ 2 * K }>: ,
    ConstU64<{ 2 * (K - F) }>: ,
    ConstI32<{ f_as_i32(F) }>: ,
    ConstI32<{ f_as_i32(K) }>: ,
    ConstU64<{ ClearFixed::<K, F>::THETA }>: ,
{
}

impl<const K: u64, const F: u64, const KAPPA: u64> Float for SecretFixed<K, F, KAPPA>
where
    ConstU64<{ K - F }>: ,
    ConstU64<{ F + 1 }>: ,
    ConstU64<{ K + 1 }>: ,
    ConstU64<{ K - 1 }>: ,
    ConstU64<{ 2 * K }>: ,
    ConstU64<{ 2 * F }>: ,
    ConstU64<{ 2 * (K - F) }>: ,
    ConstI32<{ f_as_i32(F) }>: ,
    ConstI32<{ f_as_i32(K) }>: ,
    ConstU64<{ SecretFixed::<K, F, KAPPA>::THETA }>: ,
{
}

impl Constants for ClearIEEE {}
impl Constants for SecretIEEE {}
impl<const K: u64, const F: u64> Constants for ClearFixed<K, F> {}
impl<const K: u64, const F: u64, const KAPPA: u64> Constants for SecretFixed<K, F, KAPPA> {}

impl ClearIEEE {
    #[allow(non_snake_case)]
    pub fn NaN() -> Self {
        ClearIEEE::set(9218868437227405312_i64)
    }
}

impl SecretIEEE {
    #[allow(non_snake_case)]
    pub fn NaN() -> Self {
        SecretIEEE::set(SecretI64::from(9218868437227405312_i64))
    }
}

// Evaluate polynomial using Horner's Rule
pub fn poly_eval<S, C, const N: u64>(poly: Array<C, N>, x: S) -> S
where
    S: Float,
    S: Add<C, Output = S>,
    S: Mul<S, Output = S>,
    C: Float,
{
    let mut sum: S = S::from(0_i64);
    for i in 0..N {
        sum = sum * x + *poly.get_unchecked(N - 1 - i);
    }
    sum
}

// Evaluate Pade approximation
//   P and Q must have same degree
#[allow(non_snake_case)]
pub fn Pade<S, C, const N: u64>(poly_p: Array<C, N>, poly_q: Array<C, N>, x: S) -> S
where
    S: Float,
    S: Add<S, Output = S>,
    S: Mul<S, Output = S>,
    S: Div<S, Output = S>,
    S: Mul<C, Output = S>,
    S: From<C>,
    C: Float,
{
    let mut num: S = S::from(*poly_p.get_unchecked(0));
    let mut den: S = S::from(*poly_q.get_unchecked(0));
    let mut x_pow = x;
    for i in 1..N {
        num = num + x_pow * *poly_p.get_unchecked(i);
        den = den + x_pow * *poly_q.get_unchecked(i);
        x_pow = x_pow * x;
    }
    num / den
}

/********************************/
/*           ClearIEEE          */
/********************************/

impl ClearIEEE {
    #[inline(always)]
    pub fn acos(self) -> ClearIEEE {
        ClearIEEE::set(FP_acos(self.rep()))
    }
    #[inline(always)]
    pub fn asin(self) -> ClearIEEE {
        ClearIEEE::set(FP_asin(self.rep()))
    }
    #[inline(always)]
    pub fn atan(self) -> ClearIEEE {
        ClearIEEE::set(FP_atan(self.rep()))
    }
    #[inline(always)]
    pub fn cos(self) -> ClearIEEE {
        ClearIEEE::set(FP_cos(self.rep()))
    }
    #[inline(always)]
    pub fn cosh(self) -> ClearIEEE {
        ClearIEEE::set(FP_cosh(self.rep()))
    }
    #[inline(always)]
    pub fn sin(self) -> ClearIEEE {
        ClearIEEE::set(FP_sin(self.rep()))
    }
    #[inline(always)]
    pub fn sinh(self) -> ClearIEEE {
        ClearIEEE::set(FP_sinh(self.rep()))
    }
    #[inline(always)]
    pub fn tan(self) -> ClearIEEE {
        let num = self.sin();
        let cos = self.cos();
        num / cos
    }
    #[inline(always)]
    pub fn tanh(self) -> ClearIEEE {
        ClearIEEE::set(FP_tanh(self.rep()))
    }
    #[inline(always)]
    pub fn exp(self) -> ClearIEEE {
        ClearIEEE::set(FP_exp(self.rep()))
    }
    #[inline(always)]
    pub fn log(self) -> ClearIEEE {
        ClearIEEE::set(FP_log(self.rep()))
    }
    #[inline(always)]
    pub fn log2(self) -> ClearIEEE {
        let v = ClearIEEE::set(FP_log(self.rep()));
        v / ClearIEEE::ln2()
    }
    #[inline(always)]
    pub fn log10(self) -> ClearIEEE {
        ClearIEEE::set(FP_log10(self.rep()))
    }
    #[inline(always)]
    pub fn ceil(self) -> ClearIEEE {
        ClearIEEE::set(FP_ceil(self.rep()))
    }
    #[inline(always)]
    pub fn exp2(self) -> ClearIEEE {
        let y = self * ClearIEEE::ln2();
        y.exp()
    }
}

impl Floor for ClearIEEE {
    #[inline(always)]
    fn floor(self) -> Self {
        ClearIEEE::set(FP_floor(self.rep()))
    }
}

impl FAbs for ClearIEEE {
    #[inline(always)]
    fn fabs(self) -> Self {
        ClearIEEE::set(FP_fabs(self.rep()))
    }
}

impl Sqrt for ClearIEEE {
    #[inline(always)]
    fn sqrt(self) -> ClearIEEE {
        ClearIEEE::set(FP_sqrt(self.rep()))
    }
}

/********************************/
/*           SecretIEEE         */
/********************************/

impl FAbs for SecretIEEE {
    #[inline(always)]
    fn fabs(self) -> SecretIEEE {
        let a = self.rep().set_bit(SecretBit::from(false), ConstU32::<63>);
        SecretIEEE::set(a)
    }
}

impl Floor for SecretIEEE {
    #[inline(always)]
    fn floor(self) -> Self {
        SecretIEEE::set(IEEE_floor(self.rep()))
    }
}

impl Sqrt for SecretIEEE {
    #[inline(always)]
    fn sqrt(self) -> SecretIEEE {
        SecretIEEE::set(IEEE_sqrt(self.rep()))
    }
}

impl SecretIEEE {
    #[inline(always)]
    pub fn acos(self) -> SecretIEEE {
        kernel_acos::<SecretIEEE, ClearIEEE, SecretBit>(self)
    }
    #[inline(always)]
    pub fn asin(self) -> SecretIEEE {
        kernel_asin::<SecretIEEE, ClearIEEE, SecretBit>(self)
    }
    #[inline(always)]
    pub fn atan(self) -> SecretIEEE {
        kernel_atan::<SecretIEEE, ClearIEEE, SecretBit>(self)
    }
    #[inline(always)]
    pub fn cos(self) -> SecretIEEE {
        let (w, _s1, s2) = TrigSubroutine::<SecretIEEE, ClearIEEE, SecretBit>(self);
        kernel_cos::<SecretIEEE, ClearIEEE>(w, s2)
    }
    #[inline(always)]
    pub fn cosh(self) -> SecretIEEE {
        let (a1, a2) = self.exp_2();
        let ans = (a1 + a2) * ClearIEEE::from(0.5);
        ans
    }
    #[inline(always)]
    pub fn sin(self) -> SecretIEEE {
        let (w, s1, _s2) = TrigSubroutine::<SecretIEEE, ClearIEEE, SecretBit>(self);
        kernel_sin::<SecretIEEE, ClearIEEE>(w, s1)
    }
    #[inline(always)]
    pub fn sinh(self) -> SecretIEEE {
        let (a1, a2) = self.exp_2();
        let ans = (a1 - a2) * ClearIEEE::from(0.5);
        ans
    }
    #[inline(always)]
    pub fn tan(self) -> SecretIEEE {
        let (w, s1, s2) = TrigSubroutine::<SecretIEEE, ClearIEEE, SecretBit>(self);
        let ans1 = kernel_sin::<SecretIEEE, ClearIEEE>(w, s1);
        let ans2 = kernel_cos::<SecretIEEE, ClearIEEE>(w, s2);
        ans1 / ans2
    }
    #[inline(always)]
    pub fn tanh(self) -> SecretIEEE {
        let (a1, a2) = self.exp_2();
        let ans = (a1 - a2) / (a1 + a2);
        ans
    }
    #[inline(always)]
    pub fn exp(self) -> SecretIEEE {
        let log2 = ClearIEEE::ln2();
        let ans = (self / log2).exp2();
        ans
    }
    #[inline(always)]
    pub fn log(self) -> SecretIEEE {
        let z = self.log2();
        z * ClearIEEE::ln2()
    }
    #[inline(always)]
    pub fn log10(self) -> SecretIEEE {
        let z = self.log2();
        z * ClearIEEE::from(0.30102999566398119521373889472449302677)
    }
    #[inline(always)]
    pub fn log2(self) -> SecretIEEE {
        let mut y = self.rep();
        // Extract exponent and normalize it
        let ex = (y >> ConstU32::<52>) - i64::from(1022);
        // Set the exponent in y to 1022
        let one = SecretBit::from(true);
        let zero = SecretBit::from(false);
        y = y.set_bit(zero, ConstU32::<62>);
        y = y.set_bit(one, ConstU32::<61>);
        y = y.set_bit(one, ConstU32::<60>);
        y = y.set_bit(one, ConstU32::<59>);
        y = y.set_bit(one, ConstU32::<58>);
        y = y.set_bit(one, ConstU32::<57>);
        y = y.set_bit(one, ConstU32::<56>);
        y = y.set_bit(one, ConstU32::<55>);
        y = y.set_bit(one, ConstU32::<54>);
        y = y.set_bit(one, ConstU32::<53>);
        y = y.set_bit(zero, ConstU32::<52>);
        // Now form the log2
        let z = SecretIEEE::set(y);
        let mut ans = kernel_log2::<SecretIEEE, ClearIEEE>(z);
        ans = ans + SecretIEEE::from(ex);
        ans
    }
    #[inline(always)]
    pub fn ceil(self) -> SecretIEEE {
        SecretIEEE::set(IEEE_ceil(self.rep()))
    }
    #[inline(always)]
    pub fn exp2(self) -> SecretIEEE {
        let s_t = self.ltz();
        let s: SecretIEEE = SecretIEEE::from(s_t);
        let g = kernel_exp2::<SecretIEEE, ClearIEEE, SecretBit>(self);
        let ans = (ClearIEEE::from(1_i64) - s) * g + s / g;
        ans
    }
    #[inline(always)]
    pub fn exp_2(self) -> (SecretIEEE, SecretIEEE) {
        let log2 = ClearIEEE::ln2();
        let ans1 = (self / log2).exp2();
        let ans2 = ClearIEEE::from(1_i64) / ans1;
        (ans1, ans2)
    }
}

/********************************/
/*           ClearFixed         */
/********************************/

impl<const K: u64, const F: u64> FAbs for ClearFixed<K, F>
where
    ConstU64<{ K + 1 }>: ,
    ConstU64<{ K - 1 }>: ,
{
    fn fabs(self) -> ClearFixed<K, F> {
        let s = self.rep().ltz();
        let v = (ClearModp::from(1) - s - s) * self.rep().rep();
        let u: ClearInteger<K> = ClearInteger::from(v);
        ClearFixed::set(u)
    }
}

impl<const K: u64, const F: u64> Floor for ClearFixed<K, F>
where
    ConstU64<{ K - F }>: ,
{
    fn floor(self) -> Self {
        let v = self.rep().Trunc(ConstU64::<{ K - F }>, ConstBool::<true>);
        ClearFixed::from(v)
    }
}

impl<const K: u64, const F: u64> ClearFixed<K, F>
where
    ConstU64<{ F + 1 }>: ,
    ConstU64<{ F - 1 }>: ,
    ConstU64<{ 2 * F }>: ,
    ConstU64<{ K + 1 }>: ,
    ConstU64<{ K - 1 }>: ,
    ConstU64<{ 2 * K }>: ,
    ConstU64<{ K - F }>: ,
    ConstU64<{ 2 * (K - F) }>: ,
    ConstI32<{ f_as_i32(F) }>: ,
    ConstI32<{ f_as_i32(K) }>: ,
    ConstU64<{ ClearFixed::<K, F>::THETA }>: ,
    ConstU64<{ K - F - 1 }>: ,
{
    pub fn ceil(self) -> Self {
        let f = self.floor();
        let a = self - f;
        let b = a.gtz();
        let ans = f + ClearFixed::from(1_i64) * ClearFixed::from(b);
        ans
    }
    pub fn sin(self) -> Self {
        let (w, s1, _s2) = TrigSubroutine::<ClearFixed<K, F>, ClearFixed<K, F>, ClearModp>(self);
        let ans = kernel_sin::<ClearFixed<K, F>, ClearFixed<K, F>>(w, s1);
        ans
    }
    pub fn cos(self) -> Self {
        let (w, _s1, s2) = TrigSubroutine::<ClearFixed<K, F>, ClearFixed<K, F>, ClearModp>(self);
        let ans = kernel_cos::<ClearFixed<K, F>, ClearFixed<K, F>>(w, s2);
        ans
    }
    pub fn tan(self) -> Self {
        let (w, s1, s2) = TrigSubroutine::<ClearFixed<K, F>, ClearFixed<K, F>, ClearModp>(self);
        let ans1 = kernel_sin::<ClearFixed<K, F>, ClearFixed<K, F>>(w, s1);
        let ans2 = kernel_cos::<ClearFixed<K, F>, ClearFixed<K, F>>(w, s2);
        ans1 / ans2
    }
    pub fn asin(self) -> Self {
        let ans = kernel_asin::<ClearFixed<K, F>, ClearFixed<K, F>, ClearModp>(self);
        ans
    }
    pub fn acos(self) -> Self {
        let ans = kernel_acos::<ClearFixed<K, F>, ClearFixed<K, F>, ClearModp>(self);
        ans
    }
    pub fn atan(self) -> Self {
        let ans = kernel_atan::<ClearFixed<K, F>, ClearFixed<K, F>, ClearModp>(self);
        ans
    }

    // Computes 2^x
    pub fn exp2(self) -> Self {
        let s_t = self.ltz();
        let s: ClearFixed<K, F> = ClearFixed::from(s_t);
        let g = kernel_exp2::<ClearFixed<K, F>, ClearFixed<K, F>, ClearModp>(self);
        let ans = (ClearFixed::from(1_i64) - s) * g + s / g;
        ans
    }

    // Computes exp(x)
    pub fn exp(self) -> Self {
        let log2 = ClearFixed::ln2();
        let ans = (self / log2).exp2();
        ans
    }

    // Computes exp(x) and exp(-x)
    fn exp_2(self) -> (Self, Self) {
        let log2 = ClearFixed::ln2();
        let ans1 = (self / log2).exp2();
        let ans2 = ClearFixed::from(1_i64) / ans1;
        (ans1, ans2)
    }

    // Computes sinh(x)
    pub fn sinh(self) -> Self {
        let (a1, a2) = self.exp_2();
        let ans = (a1 - a2) * ClearFixed::from(0.5);
        ans
    }

    // Computes cosh(x)
    pub fn cosh(self) -> Self {
        let (a1, a2) = self.exp_2();
        let ans = (a1 + a2) * ClearFixed::from(0.5);
        ans
    }

    // Computes tanh(x)
    pub fn tanh(self) -> Self {
        let (a1, a2) = self.exp_2();
        let ans = (a1 - a2) / (a1 + a2);
        ans
    }

    // Computes log2(x)
    pub fn log2(self) -> Self {
        let fl: ClearFloat<F, F> = ClearFloat::from(self);
        let vv: ClearFixed<K, F> = ClearFixed::set(ClearInteger::set(fl.clone().v()));
        let mut a = kernel_log2::<ClearFixed<K, F>, ClearFixed<K, F>>(vv);
        let pp: ClearFixed<K, F> = ClearFixed::from(fl.clone().p() + ClearModp::from(F as i64));
        a = a + pp;
        let mask = (ClearModp::from(1) - fl.clone().z()) * (ClearModp::from(1) - fl.clone().s());
        let t = a.rep().rep() * mask;
        ClearFixed::set(ClearInteger::set(t))
    }

    // Computes log(x)
    pub fn log(self) -> Self {
        let v = self.log2() * ClearFixed::ln2();
        v
    }

    // Computes log10(x)
    pub fn log10(self) -> Self {
        let v = self.log2() * ClearFixed::from(0.30102999566398119521373889472449302677);
        v
    }
}

/********************************/
/*           SecretFixed        */
/********************************/

impl<const K: u64, const F: u64, const KAPPA: u64> FAbs for SecretFixed<K, F, KAPPA>
where
    ConstU64<{ K + 1 }>: ,
    ConstU64<{ K - 1 }>: ,
{
    fn fabs(self) -> SecretFixed<K, F, KAPPA> {
        let s = self.rep().ltz();
        let v = (ClearModp::from(1) - s - s) * self.rep().rep();
        let u: SecretInteger<K, KAPPA> = SecretInteger::from(v);
        SecretFixed::set(u)
    }
}

impl<const K: u64, const F: u64, const KAPPA: u64> Floor for SecretFixed<K, F, KAPPA>
where
    ConstU64<{ K - F }>: ,
{
    fn floor(self) -> Self {
        let v = self.rep().Trunc(ConstU64::<{ K - F }>, ConstBool::<true>);
        SecretFixed::from(v)
    }
}

impl<const K: u64, const F: u64, const KAPPA: u64> SecretFixed<K, F, KAPPA>
where
    ConstU64<{ K - F }>: ,
    ConstU64<{ 2 * K }>: ,
    ConstU64<{ 2 * F }>: ,
    ConstU64<{ 2 * (K - F) }>: ,
    ConstU64<{ K - 1 }>: ,
    ConstU64<{ K + 1 }>: ,
    ConstU64<{ F + 1 }>: ,
    ConstU64<{ F - 1 }>: ,
    ConstI32<{ f_as_i32(F) }>: ,
    ConstI32<{ f_as_i32(K) }>: ,
    ConstU64<{ ClearFixed::<K, F>::THETA }>: ,
    ConstU64<{ SecretFixed::<K, F, KAPPA>::THETA }>: ,
    ConstU64<{ CeilLog2::<K>::RESULT }>: ,
    ConstU64<{ K - F - 1 }>: ,
{
    pub fn ceil(self) -> Self {
        let f = self.floor();
        let a = self - f;
        let b = a.gtz();
        let ans = f + ClearFixed::from(1_i64) * SecretFixed::from(b);
        ans
    }
    pub fn sin(self) -> Self {
        let (w, s1, _s2) =
            TrigSubroutine::<SecretFixed<K, F, KAPPA>, ClearFixed<K, F>, SecretModp>(self);
        let ans = kernel_sin::<SecretFixed<K, F, KAPPA>, ClearFixed<K, F>>(w, s1);
        ans
    }
    pub fn cos(self) -> Self {
        let (w, _s1, s2) =
            TrigSubroutine::<SecretFixed<K, F, KAPPA>, ClearFixed<K, F>, SecretModp>(self);
        let ans = kernel_cos::<SecretFixed<K, F, KAPPA>, ClearFixed<K, F>>(w, s2);
        ans
    }
    pub fn tan(self) -> Self {
        let (w, s1, s2) =
            TrigSubroutine::<SecretFixed<K, F, KAPPA>, ClearFixed<K, F>, SecretModp>(self);
        let ans1 = kernel_sin::<SecretFixed<K, F, KAPPA>, ClearFixed<K, F>>(w, s1);
        let ans2 = kernel_cos::<SecretFixed<K, F, KAPPA>, ClearFixed<K, F>>(w, s2);
        ans1 / ans2
    }
    pub fn asin(self) -> Self {
        let ans = kernel_asin::<SecretFixed<K, F, KAPPA>, ClearFixed<K, F>, SecretModp>(self);
        ans
    }
    pub fn acos(self) -> Self {
        let ans = kernel_acos::<SecretFixed<K, F, KAPPA>, ClearFixed<K, F>, SecretModp>(self);
        ans
    }
    pub fn atan(self) -> Self {
        let ans = kernel_atan::<SecretFixed<K, F, KAPPA>, ClearFixed<K, F>, SecretModp>(self);
        ans
    }

    // Computes 2^x
    pub fn exp2(self) -> Self {
        let s_t = self.ltz();
        let s: SecretFixed<K, F, KAPPA> = SecretFixed::from(s_t);
        let g = kernel_exp2::<SecretFixed<K, F, KAPPA>, ClearFixed<K, F>, SecretModp>(self);
        let ans = (ClearFixed::from(1_i64) - s) * g + s / g;
        ans
    }

    // Computes exp(x)
    pub fn exp(self) -> Self {
        let log2 = ClearFixed::ln2();
        let ans = (self / log2).exp2();
        ans
    }

    // Computes exp(x) and exp(-x)
    fn exp_2(self) -> (Self, Self) {
        let log2 = ClearFixed::ln2();
        let ans1 = (self / log2).exp2();
        let ans2 = ClearFixed::from(1_i64) / ans1;
        (ans1, ans2)
    }

    // Computes sinh(x)
    pub fn sinh(self) -> Self {
        let (a1, a2) = self.exp_2();
        let ans = (a1 - a2) * ClearFixed::from(0.5);
        ans
    }

    // Computes cosh(x)
    pub fn cosh(self) -> Self {
        let (a1, a2) = self.exp_2();
        let ans = (a1 + a2) * ClearFixed::from(0.5);
        ans
    }

    // Computes tanh(x)
    pub fn tanh(self) -> SecretFixed<K, F, KAPPA> {
        let (a1, a2) = self.exp_2();
        let ans = (a1 - a2) / (a1 + a2);
        ans
    }

    // Computes log2(x)
    pub fn log2(self) -> Self {
        let fl: SecretFloat<F, F, KAPPA> = SecretFloat::from(self);
        let vv: SecretFixed<K, F, KAPPA> = SecretFixed::set(SecretInteger::set(fl.clone().v()));
        let mut a = kernel_log2::<SecretFixed<K, F, KAPPA>, ClearFixed<K, F>>(vv);
        let pp: SecretFixed<K, F, KAPPA> =
            SecretFixed::from(fl.clone().p() + ClearModp::from(F as i64));
        a = a + pp;
        let mask = (ClearModp::from(1) - fl.clone().z()) * (ClearModp::from(1) - fl.s());
        let t = a.rep().rep() * mask;
        SecretFixed::set(SecretInteger::set(t))
    }

    // Computes log(x)
    pub fn log(self) -> Self {
        let v = self.log2() * ClearFixed::ln2();
        v
    }

    // Computes log10(x)
    pub fn log10(self) -> Self {
        let v = self.log2() * ClearFixed::from(0.30102999566398119521373889472449302677);
        v
    }
}
