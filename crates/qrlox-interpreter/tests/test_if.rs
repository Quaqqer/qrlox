mod common;

use common::test_io;
use indoc::indoc;

#[test]
fn test_if() {
    let s = indoc! {"
        if (true) {
            print 1;
        }
    "};

    test_io(s, "1");
}

#[test]
fn test_if_else() {
    let s = indoc! {"
        if (false) {
            print 1;
        } else {
            print 2;
        }
    "};
    test_io(s, "2");

    let s = indoc! {"
        if (true) {
            print 1;
        } else {
            print 2;
        }
    "};
    test_io(s, "1");
}

#[test]
fn test_truthiness() {
    let s = indoc! {r#"
        if (0) print 0;
        if (1) print 1;
        if (true) print 2;
        if (false) print 3;
        if (nil) print 4;
        if ("hello") print 5;
    "#};
    test_io(s, "0\n1\n2\n5");
}
