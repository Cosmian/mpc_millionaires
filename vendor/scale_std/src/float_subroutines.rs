// Copyright (c) 2021, COSIC-KU Leuven, Kasteelpark Arenberg 10, bus 2452, B-3001 Leuven-Heverlee, Belgium.
// Copyright (c) 2021, Cosmian Tech SAS, 53-55 rue La Boétie, Paris, France.

use crate::bit_protocols::*;
use crate::integer::*;
use crate::slice::*;
use scale::*;

/**************************************
 * Helper routines for floating point *
 **************************************/

/* Input an integer of size K,
 * Output float tuple with mantissa of size L
 */
#[allow(non_snake_case)]
pub fn Clear_Int2Fl<const K: u64, const L: u64>(
    a_int: ClearInteger<K>,
    _: ConstU64<L>,
) -> (ClearModp, ClearModp, ClearModp, ClearModp, ClearModp)
where
    ConstU64<{ L - 1 }>: ,
    ConstU64<{ L + 1 }>: ,
    ConstU64<{ K - 1 }>: ,
    ConstU64<{ K + 1 }>: ,
    ConstU64<{ K - L - 1 }>: ,
{
    let s = a_int.ltz();
    let z = a_int.eqz();
    let a = a_int.rep();
    let aa = (ClearModp::from(1) - s - s) * a;
    let vec_a = BitDec_ClearModp(aa, K - 1);
    let mut rev_a: Slice<ClearModp> = Slice::uninitialized(K - 1);
    for i in 0..K - 1 {
        rev_a.set(i, &*vec_a.get_unchecked(K - 2 - i));
    }
    let vec_b = rev_a.PreOr();
    let one = ClearModp::from(1_i64);
    let mut v = one;
    let mut p = ClearModp::from((K - 1) as i64);
    let mut twop = ClearModp::from(1_i64);
    for i in 0..K - 1 {
        v = v + twop * (one - *vec_b.get_unchecked(i));
        p = p - *vec_b.get_unchecked(i);
        twop = twop + twop;
    }
    p = -p;
    v = a * v;
    if (K - 1) > L {
        let mut v_int: ClearInteger<{ K - 1 }> = ClearInteger::from(v);
        v_int = v_int.Trunc(ConstU64::<{ K - L - 1 }>, ConstBool::<false>);
        v = v_int.rep();
    } else {
        v = v * modp_two_power(L - K + 1);
    }
    p = (p + ClearModp::from((K - L - 1) as i64)) * (ClearModp::from(1) - z);
    let err = ClearModp::from(0_i64);
    (v, p, z, s, err)
}

/* Input an integer of size K,
 * Output float tuple with mantissa of size L
 */
#[allow(non_snake_case)]
pub fn Secret_Int2Fl<const K: u64, const L: u64, const KAPPA: u64>(
    a_int: SecretInteger<K, KAPPA>,
    _: ConstU64<L>,
) -> (SecretModp, SecretModp, SecretModp, SecretModp, SecretModp)
where
    ConstU64<{ L - 1 }>: ,
    ConstU64<{ L + 1 }>: ,
    ConstU64<{ K - 1 }>: ,
    ConstU64<{ K + 1 }>: ,
    ConstU64<{ K - L - 1 }>: ,
{
    let s = a_int.ltz();
    let z = a_int.eqz();
    let a = a_int.rep();
    let aa = (SecretModp::from(1) - s - s) * a;
    let vec_a = BitDec::<{ K - 1 }, { K - 1 }, KAPPA>(aa);
    let mut rev_a: Slice<SecretModp> = Slice::uninitialized(K - 1);
    for i in 0..K - 1 {
        rev_a.set(i, &*vec_a.get_unchecked(K - 2 - i));
    }
    let vec_b = rev_a.PreOr();
    let one = SecretModp::from(1_i64);
    let mut v = one;
    let mut p = SecretModp::from((K - 1) as i64);
    let mut twop = SecretModp::from(1_i64);
    for i in 0..K - 1 {
        v = v + twop * (one - *vec_b.get_unchecked(i));
        p = p - *vec_b.get_unchecked(i);
        twop = twop + twop;
    }
    p = -p;
    v = a * v;
    if (K - 1) > L {
        let mut v_int: SecretInteger<{ K - 1 }, KAPPA> = SecretInteger::from(v);
        v_int = v_int.Trunc(ConstU64::<{ K - L - 1 }>, ConstBool::<false>);
        v = v_int.rep();
    } else {
        v = v * modp_two_power(L - K + 1);
    }
    p = (p + SecretModp::from((K - L - 1) as i64)) * (SecretModp::from(1) - z);
    let err = SecretModp::from(0_i64);
    (v, p, z, s, err)
}
