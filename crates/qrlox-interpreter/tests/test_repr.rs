mod common;

use common::test_io;

#[test]
fn test_number() {
    test_io("print 1;", "1");
    test_io("print 1.23;", "1.23");
    test_io("print 1.0000;", "1");
}
