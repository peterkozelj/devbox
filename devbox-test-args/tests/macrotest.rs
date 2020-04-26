use devbox_test_args::{args, test_args};

//-- #[args] ---------------------------------------------------------------------------------------

#[args]
#[test]
fn args_noargs() {
    assert!(true, "Letter code incorrect");
}

#[args(
    char_a: 97, 'a';
    char_b: 98, 'b';
)]
#[args(
    offset_0: 0;
    offset_1: 1 ! "code incorrect";
)]
#[test]
fn args_cartesic(code:_, letter:_, offset:_) {
    assert_eq!(code + offset, letter as u8, "Letter code incorrect");
}

//-- #[test_args] ----------------------------------------------------------------------------------

#[test_args]
fn test_noargs() {
    assert!(true, "Letter code incorrect");
}

#[test_args(
    char_a: 97, 'a';
    char_b: 97, 'b' ! "code incorrect";
)]
fn test_standard(code:_, letter:_) {
    assert_eq!(code, letter as u8, "Letter code incorrect");
}
