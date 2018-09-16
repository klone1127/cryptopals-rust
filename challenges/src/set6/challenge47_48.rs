use errors::*;

use rsa::Rsa;

use bignum::OpensslBigNum as BigNum;
use bignum::{BigNumExt, BigNumTrait};

use rand;
use rand::Rng;

use std::cmp;

// We use the variable names from the paper
#[allow(non_snake_case)]
pub fn run(rsa_bits: usize) -> Result<(), Error> {
    let _0 = BigNum::zero();
    let _1 = BigNum::one();
    let _2 = BigNum::from_u32(2);
    let _3 = BigNum::from_u32(3);

    let rsa = Rsa::<BigNum>::generate(rsa_bits);
    let n = rsa.n();
    let k = n.bytes() as usize;
    let B = _1.lsh(8 * (k - 2));
    let _2B = &_2 * &B;
    let _3B = &_3 * &B;

    let oracle = |ciphertext: &BigNum| -> bool {
        let cleartext = rsa.decrypt(ciphertext);
        cleartext.rsh(8 * (k - 2)) == _2
    };

    let cleartext = b"kick it, CC";
    let cleartext_len = cleartext.len();
    assert!(cleartext_len <= k - 11);
    let mut padded_cleartext = vec![2u8];
    let mut rng = rand::thread_rng();
    padded_cleartext.extend((0..(k - 3 - cleartext_len)).map(|_| rng.gen_range(1, 256) as u8));
    padded_cleartext.push(0);
    padded_cleartext.extend_from_slice(cleartext);
    let m: BigNum = BigNumTrait::from_bytes_be(&padded_cleartext);
    let c = rsa.encrypt(&m);

    // We are only ever going to use `oracle` in the following way
    let wrapped_oracle = |s: &BigNum| -> bool { oracle(&(&c * &rsa.encrypt(s))) };

    let mut M_prev = vec![(_2B.clone(), &_3B - &_1)];
    let mut s_prev = _1.clone();
    let mut i = 1;

    loop {
        // Step 2
        let mut si;
        if i == 1 {
            // Step 2.a
            si = n.ceil_quotient(&_3B);
            while !wrapped_oracle(&si) {
                si = &si + &_1;
            }
        } else if M_prev.len() >= 2 {
            // Step 2.b
            si = &s_prev + &_1;
            while !wrapped_oracle(&si) {
                si = &si + &_1;
            }
        } else {
            // Step 2.c
            let (ref a, ref b) = M_prev[0];
            let mut ri = (&_2 * &(&(b * &s_prev) - &_2B)).ceil_quotient(n);
            'outer: loop {
                si = (&_2B + &(&ri * n)).ceil_quotient(b);
                let U = (&_3B + &(&ri * n)).ceil_quotient(a);
                while si < U {
                    if wrapped_oracle(&si) {
                        break 'outer;
                    }
                    si = &si + &_1;
                }
                ri = &ri + &_1;
            }
        }

        let mut Mi = Vec::new();
        for &(ref a, ref b) in &M_prev {
            let mut r = (&(&(a * &si) - &_3B) + &_1).ceil_quotient(n);
            let U = (&(b * &si) - &_2B).floor_quotient(n);
            while r <= U {
                Mi.push((
                    cmp::max(a.clone(), (&_2B + &(&r * n)).ceil_quotient(&si)),
                    cmp::min(b.clone(), (&(&_3B - &_1) + &(&r * n)).floor_quotient(&si)),
                ));
                r = &r + &_1;
            }
        }

        Mi.sort();
        Mi.dedup();
        if Mi.len() == 1 && Mi[0].0 == Mi[0].1 {
            return compare_eq(&m, &Mi[0].0);
        }
        i += 1;
        s_prev = si;
        M_prev = Mi;
    }
}
