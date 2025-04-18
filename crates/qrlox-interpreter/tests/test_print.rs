mod common;

use common::test_io;
use indoc::indoc;

#[test]
fn test_number() {
    test_io("print 1;", "1");
    test_io("print 1.23;", "1.23");
    test_io("print 1.0000;", "1");
}

#[test]
fn test_string() {
    test_io(r#"print "hello";"#, "hello");
}

#[test]
fn test_boolean() {
    test_io(
        indoc! {"
            print true;
            print false;
        "},
        indoc! {"
            true
            false
        "},
    )
}

#[test]
fn test_nil() {
    test_io(r#"print nil;"#, "nil");
}
