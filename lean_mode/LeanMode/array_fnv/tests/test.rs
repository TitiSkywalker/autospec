use array_fnv::fnv1a;

#[test]
fn test_empty() {
    let data: Vec<u64> = vec![];
    assert_eq!(fnv1a(&data), 14695981039346656037);
}

#[test]
fn test_single_a() {
    assert_eq!(fnv1a(&[97]), 12638187200555641996);
}

#[test]
fn test_abc() {
    assert_eq!(fnv1a(&[97, 98, 99]), 16654208175385433931);
}

#[test]
fn test_foobar() {
    assert_eq!(fnv1a(&[102, 111, 111, 98, 97, 114]), 9625390261332436968);
}
