mod common;

use common::test_io;

#[test]
fn test_test_io() {
    test_io("print \"hello\";", "hello");
    test_io("", "");
}

#[test]
#[should_panic]
fn test_test_io_negative() {
    test_io("print \"hello\";", "bello");
}
