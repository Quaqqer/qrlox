use indoc::indoc;
use qrlox_tests::test_io;

#[test]
fn test_fibonacci_recursive() {
    let s = indoc! {"
        fun fib_rec(i) {
            if (i < 2) {
                return i;
            } else {
                return fib_rec(i - 1) + fib_rec(i - 2);
            }
        }

        print fib_rec(0);
        print fib_rec(1);
        print fib_rec(10);
    "};

    test_io(s, "0\n1\n55");
}

#[test]
fn test_fibonacci_iterative() {
    let s = indoc! {"
        fun fib_iter(i) {
            var a = 0;
            var b = 1;

            for (var j = 0; j < i; j = j + 1) {
                var c = a + b;
                a = b;
                b = c;
            }

            return a;
        }

        print fib_iter(0);
        print fib_iter(1);
        print fib_iter(10);
    "};

    test_io(s, "0\n1\n55");
}

#[test]
fn test_nested() {
    let s = indoc! {r#"
        fun f(x) {
            fun g(y) {
                return y * 2;
            }

            return g(x) * 3;
        }

        print f(3);
    "#};

    test_io(s, "18");
}
