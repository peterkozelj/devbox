#[devbox_test::test(
    char_a: 97, 'a';
    char_b: 98, 'b';
)]
#[devbox_test::test(
    offset_0: 0;
    offset_1: 1 ! "code incorrect";
)]
fn parametrized_test_for(code:_, letter:_, offset:_) {
    assert_eq!(code + offset, letter as u8, "Letter code incorrect");
}