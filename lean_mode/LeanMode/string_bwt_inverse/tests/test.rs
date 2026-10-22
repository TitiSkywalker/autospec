// Characters are `usize` byte values in `1..=255`; `0` is the reserved sentinel.

/// Element-wise equality of two `usize` sequences (see the note about
/// `assert_eq!` on `Vec`s: compare length + elements instead).
fn same_seq(v: &[usize], w: &[usize]) -> bool {
    let mut ok = true;
    if v.len() != w.len() {
        ok = false;
    } else {
        let n = v.len();
        let mut i = 0;
        while i < n {
            if v[i] != w[i] {
                ok = false;
            }
            i += 1;
        }
    }
    ok
}

#[test]
fn test_empty() {
    let s: [usize; 0] = [];
    let enc = string_bwt_inverse::bwt_encode(&s);
    assert_eq!(enc.len(), 1);
    assert_eq!(enc[0], 0);
    let dec = string_bwt_inverse::bwt_decode(&enc);
    assert_eq!(dec.len(), 0);
}

#[test]
fn test_single() {
    let s: [usize; 1] = [97]; // "a"
    let enc = string_bwt_inverse::bwt_encode(&s);
    assert_eq!(enc.len(), 2);
    let dec = string_bwt_inverse::bwt_decode(&enc);
    assert_eq!(dec.len(), 1);
    assert_eq!(dec[0], 97);
}

#[test]
fn test_two_same() {
    let s: [usize; 2] = [97, 97]; // "aa"
    let enc = string_bwt_inverse::bwt_encode(&s);
    assert_eq!(enc.len(), 3);
    assert_eq!(enc[0], 97);
    assert_eq!(enc[1], 97);
    assert_eq!(enc[2], 0);
    let dec = string_bwt_inverse::bwt_decode(&enc);
    assert!(same_seq(&dec, &s));
}

#[test]
fn test_two_distinct() {
    let s: [usize; 2] = [97, 98]; // "ab"
    let enc = string_bwt_inverse::bwt_encode(&s);
    assert_eq!(enc.len(), 3);
    assert_eq!(enc[0], 98);
    assert_eq!(enc[1], 0);
    assert_eq!(enc[2], 97);
    let dec = string_bwt_inverse::bwt_decode(&enc);
    assert!(same_seq(&dec, &s));
}

#[test]
fn test_banana() {
    let s: [usize; 6] = [98, 97, 110, 97, 110, 97]; // "banana"
    let enc = string_bwt_inverse::bwt_encode(&s);
    assert_eq!(enc.len(), 7);
    let dec = string_bwt_inverse::bwt_decode(&enc);
    assert!(same_seq(&dec, &s));
}

#[test]
fn test_mississippi() {
    let s: [usize; 11] = [109, 105, 115, 115, 105, 115, 115, 105, 112, 112, 105];
    let enc = string_bwt_inverse::bwt_encode(&s);
    assert_eq!(enc.len(), 12);
    let dec = string_bwt_inverse::bwt_decode(&enc);
    assert!(same_seq(&dec, &s));
}

#[test]
fn test_abracadabra() {
    let s: [usize; 11] = [97, 98, 114, 97, 99, 97, 100, 97, 98, 114, 97];
    let enc = string_bwt_inverse::bwt_encode(&s);
    assert_eq!(enc.len(), 12);
    let dec = string_bwt_inverse::bwt_decode(&enc);
    assert!(same_seq(&dec, &s));
}

#[test]
fn test_aabbccdd() {
    let s: [usize; 8] = [97, 97, 98, 98, 99, 99, 100, 100];
    let enc = string_bwt_inverse::bwt_encode(&s);
    assert_eq!(enc.len(), 9);
    let dec = string_bwt_inverse::bwt_decode(&enc);
    assert!(same_seq(&dec, &s));
}

#[test]
fn test_long_same() {
    let s: [usize; 40] = [97; 40];
    let enc = string_bwt_inverse::bwt_encode(&s);
    assert_eq!(enc.len(), 41);
    let dec = string_bwt_inverse::bwt_decode(&enc);
    assert_eq!(dec.len(), 40);
    let mut i = 0;
    let mut all_same = true;
    while i < 40 {
        if dec[i] != 97 {
            all_same = false;
        }
        i += 1;
    }
    assert!(all_same);
}

#[test]
fn test_high_byte_values() {
    let s: [usize; 4] = [255, 1, 200, 255];
    let enc = string_bwt_inverse::bwt_encode(&s);
    assert_eq!(enc.len(), 5);
    let dec = string_bwt_inverse::bwt_decode(&enc);
    assert!(same_seq(&dec, &s));
}
