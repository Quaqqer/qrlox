mod common;

use common::test_io;
use indoc::indoc;

#[test]
fn test_for() {
    let s = indoc! {r#"
        for (var i = 0; i < 10; i = i + 1) {
            print i;
        }
    "#};
    test_io(s, "0\n1\n2\n3\n4\n5\n6\n7\n8\n9");
}

#[test]
fn test_while() {
    let s = indoc! {r#"
        var i = 0;
        while (i < 11) {
            print i;
            i = i + 1;
        }
    "#};
    test_io(s, "0\n1\n2\n3\n4\n5\n6\n7\n8\n9\n10");
}
