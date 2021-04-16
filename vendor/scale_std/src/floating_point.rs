// Copyright (c) 2021, COSIC-KU Leuven, Kasteelpark Arenberg 10, bus 2452, B-3001 Leuven-Heverlee, Belgium.
// Copyright (c) 2021, Cosmian Tech SAS, 53-55 rue La Boétie, Paris, France.

use crate::array::*;
use crate::fixed_point::*;
use crate::float_subroutines::*;
use crate::ieee::*;
use crate::integer::*;
use crate::local_functions::*;
use scale::*;

/* This gives floating point arithmetic
 *
 * It uses the global statistical security parameter kappa
 * Follows the algorithms in Section 14.5 of the main Scale
 * manual
 *
 */

#[derive(Clone)]
pub struct ClearFloat<const V: u64, const P: u64> {
    param: Array<ClearModp, 5>, // v, p, z, s, err
}

#[derive(Clone)]
pub struct SecretFloat<const V: u64, const P: u64, const KAPPA: u64> {
    param: Array<SecretModp, 5>, // v, p, z, s, err
}

/* Prints a clear float */

impl<const V: u64, const P: u64> Print for ClearFloat<V, P> {
    #[inline(always)]
    fn print(self) {
        unsafe {
            __print_float(
                *self.param.get_unchecked(0),
                *self.param.get_unchecked(1),
                *self.param.get_unchecked(2),
                *self.param.get_unchecked(3),
                *self.param.get_unchecked(4),
            )
        }
    }
}

/* Basic Constructors */

impl<const V: u64, const P: u64> From<ClearIEEE> for ClearFloat<V, P>
where
    ConstU64<{ V - 1 }>: ,
    ConstU64<{ V + 1 }>: ,
{
    #[inline(always)]
    fn from(a: ClearIEEE) -> Self {
        let (v, p, z, s, err) = to_float(a.rep(), V as i64, P as i64);
        ClearFloat::set(v, p, z, s, err)
    }
}

impl<const V: u64, const P: u64> From<f64> for ClearFloat<V, P>
where
    ConstU64<{ V - 1 }>: ,
    ConstU64<{ V + 1 }>: ,
{
    #[inline(always)]
    fn from(f: f64) -> Self {
        let a: ClearIEEE = ClearIEEE::from(f);
        Self::from(a)
    }
}

impl<const V: u64, const P: u64> From<i64> for ClearFloat<V, P>
where
    ConstU64<{ V - 1 }>: ,
    ConstU64<{ V + 1 }>: ,
{
    #[inline(always)]
    fn from(a: i64) -> Self {
        let b: ClearIEEE = ClearIEEE::from(a);
        Self::from(b)
    }
}

impl<const V: u64, const P: u64, const KAPPA: u64> From<f64> for SecretFloat<V, P, KAPPA>
where
    ConstU64<{ V - 1 }>: ,
    ConstU64<{ V + 1 }>: ,
{
    #[inline(always)]
    fn from(f: f64) -> Self {
        let a: ClearIEEE = ClearIEEE::from(f);
        Self::from(ClearFloat::from(a))
    }
}

impl<const V: u64, const P: u64, const KAPPA: u64> From<i64> for SecretFloat<V, P, KAPPA>
where
    ConstU64<{ V - 1 }>: ,
    ConstU64<{ V + 1 }>: ,
{
    #[inline(always)]
    fn from(a: i64) -> Self {
        let b: ClearIEEE = ClearIEEE::from(a);
        Self::from(ClearFloat::from(b))
    }
}

impl<const V: u64, const P: u64, const KAPPA: u64> From<ClearFloat<V, P>>
    for SecretFloat<V, P, KAPPA>
where
    ConstU64<{ V - 1 }>: ,
    ConstU64<{ V + 1 }>: ,
{
    #[inline(always)]
    fn from(a: ClearFloat<V, P>) -> Self {
        let mut ans: Array<SecretModp, 5> = Array::uninitialized();
        ans.set(0, &SecretModp::from(*a.param.get_unchecked(0)));
        ans.set(1, &SecretModp::from(*a.param.get_unchecked(1)));
        ans.set(2, &SecretModp::from(*a.param.get_unchecked(2)));
        ans.set(3, &SecretModp::from(*a.param.get_unchecked(3)));
        ans.set(4, &SecretModp::from(*a.param.get_unchecked(4)));
        Self { param: ans }
    }
}

impl<const V: u64, const P: u64, const K: u64> From<ClearInteger<K>> for ClearFloat<V, P>
where
    ConstU64<{ V - 1 }>: ,
    ConstU64<{ V + 1 }>: ,
    ConstU64<{ K - 1 }>: ,
    ConstU64<{ K + 1 }>: ,
    ConstU64<{ K - V - 1 }>: ,
{
    #[inline(always)]
    fn from(a: ClearInteger<K>) -> Self {
        let (v, p, z, s, err) = Clear_Int2Fl(a, ConstU64::<V>);
        ClearFloat::set(v, p, z, s, err)
    }
}

impl<const V: u64, const P: u64, const KAPPA: u64, const K: u64> From<SecretInteger<K, KAPPA>>
    for SecretFloat<V, P, KAPPA>
where
    ConstU64<{ V - 1 }>: ,
    ConstU64<{ V + 1 }>: ,
    ConstU64<{ K - 1 }>: ,
    ConstU64<{ K + 1 }>: ,
    ConstU64<{ K - V - 1 }>: ,
{
    #[inline(always)]
    fn from(a: SecretInteger<K, KAPPA>) -> Self {
        let (v, p, z, s, err) = Secret_Int2Fl(a, ConstU64::<V>);
        SecretFloat::set(v, p, z, s, err)
    }
}

impl<const K: u64, const F: u64, const V: u64, const P: u64> From<ClearFixed<K, F>>
    for ClearFloat<V, P>
where
    ConstU64<{ V - 1 }>: ,
    ConstU64<{ V + 1 }>: ,
    ConstU64<{ K - 1 }>: ,
    ConstU64<{ K + 1 }>: ,
    ConstU64<{ K - V - 1 }>: ,
{
    #[inline(always)]
    fn from(a: ClearFixed<K, F>) -> Self {
        let (v, pt, z, s, err) = Clear_Int2Fl(a.rep(), ConstU64::<V>);
        let p = (pt - ClearModp::from(F as i64)) * (ClearModp::from(1) - z);
        ClearFloat::set(v, p, z, s, err)
    }
}

impl<const K: u64, const F: u64, const V: u64, const P: u64, const KAPPA: u64>
    From<SecretFixed<K, F, KAPPA>> for SecretFloat<V, P, KAPPA>
where
    ConstU64<{ V - 1 }>: ,
    ConstU64<{ V + 1 }>: ,
    ConstU64<{ K - 1 }>: ,
    ConstU64<{ K + 1 }>: ,
    ConstU64<{ K - V - 1 }>: ,
{
    #[inline(always)]
    fn from(a: SecretFixed<K, F, KAPPA>) -> Self {
        let (v, pt, z, s, err) = Secret_Int2Fl(a.rep(), ConstU64::<V>);
        let p = (pt - ClearModp::from(F as i64)) * (ClearModp::from(1) - z);
        SecretFloat::set(v, p, z, s, err)
    }
}

/* Set the underlying representation to something */

impl<const V: u64, const P: u64> ClearFloat<V, P> {
    #[inline(always)]
    pub fn set(
        v: ClearModp,
        p: ClearModp,
        z: ClearModp,
        s: ClearModp,
        err: ClearModp,
    ) -> ClearFloat<V, P> {
        let mut ans: Array<ClearModp, 5> = Array::uninitialized();
        ans.set(0, &v);
        ans.set(1, &p);
        ans.set(2, &z);
        ans.set(3, &s);
        ans.set(4, &err);
        Self { param: ans }
    }
}

impl<const V: u64, const P: u64, const KAPPA: u64> SecretFloat<V, P, KAPPA> {
    #[inline(always)]
    pub fn set(
        v: SecretModp,
        p: SecretModp,
        z: SecretModp,
        s: SecretModp,
        err: SecretModp,
    ) -> SecretFloat<V, P, KAPPA> {
        let mut ans: Array<SecretModp, 5> = Array::uninitialized();
        ans.set(0, &v);
        ans.set(1, &p);
        ans.set(2, &z);
        ans.set(3, &s);
        ans.set(4, &err);
        Self { param: ans }
    }
}

/* Get the underlying representation */

impl<const V: u64, const P: u64> ClearFloat<V, P> {
    #[inline(always)]
    pub fn v(self) -> ClearModp {
        *self.param.get_unchecked(0)
    }
    #[inline(always)]
    pub fn p(self) -> ClearModp {
        *self.param.get_unchecked(1)
    }
    #[inline(always)]
    pub fn z(self) -> ClearModp {
        *self.param.get_unchecked(2)
    }
    #[inline(always)]
    pub fn s(self) -> ClearModp {
        *self.param.get_unchecked(3)
    }
    #[inline(always)]
    pub fn err(self) -> ClearModp {
        *self.param.get_unchecked(3)
    }
}

impl<const V: u64, const P: u64, const KAPPA: u64> SecretFloat<V, P, KAPPA> {
    #[inline(always)]
    pub fn v(self) -> SecretModp {
        *self.param.get_unchecked(0)
    }
    #[inline(always)]
    pub fn p(self) -> SecretModp {
        *self.param.get_unchecked(1)
    }
    #[inline(always)]
    pub fn z(self) -> SecretModp {
        *self.param.get_unchecked(2)
    }
    #[inline(always)]
    pub fn s(self) -> SecretModp {
        *self.param.get_unchecked(3)
    }
    #[inline(always)]
    pub fn err(self) -> SecretModp {
        *self.param.get_unchecked(3)
    }
}

/* Reveal Operation */

impl<const V: u64, const P: u64, const KAPPA: u64> Reveal for SecretFloat<V, P, KAPPA> {
    type Output = ClearFloat<V, P>;
    #[inline(always)]
    fn reveal(&self) -> ClearFloat<V, P> {
        let mut ans: Array<ClearModp, 5> = Array::uninitialized();
        ans.set(0, &self.param.get_unchecked(0).reveal());
        ans.set(1, &self.param.get_unchecked(1).reveal());
        ans.set(2, &self.param.get_unchecked(2).reveal());
        ans.set(3, &self.param.get_unchecked(3).reveal());
        ans.set(4, &self.param.get_unchecked(4).reveal());
        ClearFloat { param: ans }
    }
}
