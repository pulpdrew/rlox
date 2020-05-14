mod common;

#[test]
fn empty_source() {
    let source = "".trim().to_string();

    let expected_stderr = "".trim();
    let expected_stdout = "".trim();

    let (stdout, stderr) = common::run(source);
    assert_eq!(expected_stderr, stderr.contents.trim());
    assert_eq!(expected_stdout, stdout.contents.trim());
}

#[test]
fn print_primitives() {
    let source = "
        print 6;
        print \"Hello World.\";
        print true;
        print false;
        print nil;
    "
    .trim()
    .to_string();

    let expected_stderr = "".trim();
    let expected_stdout = "
6
Hello World.
true
false
nil
    "
    .trim();

    let (stdout, stderr) = common::run(source);
    assert_eq!(expected_stderr, stderr.contents.trim());
    assert_eq!(expected_stdout, stdout.contents.trim());
}

#[test]
fn strings() {
    let source = "
        print \"A\";
        print \"\";
        print \"A\" + \"BC\";
    "
    .trim()
    .to_string();

    let expected_stderr = "".trim();
    let expected_stdout = "
A

ABC
    "
    .trim();

    let (stdout, stderr) = common::run(source);
    assert_eq!(expected_stderr, stderr.contents.trim());
    assert_eq!(expected_stdout, stdout.contents.trim());
}

#[test]
fn globals() {
    let source = "
        var a = 6;
        var b = 1.5;
        print a;
        print b;
        print a + b;
        print a = b = 9;
        print a;
        print b;

        var b = b;
        print b;
        b = \"Hi\";
        print b;

        var c;
        print c;
    "
    .trim()
    .to_string();

    let expected_stderr = "".trim();
    let expected_stdout = "
6
1.5
7.5
9
9
9
9
Hi
nil
"
    .trim();

    let (stdout, stderr) = common::run(source);
    assert_eq!(expected_stderr, stderr.contents.trim());
    assert_eq!(expected_stdout, stdout.contents.trim());
}

#[test]
fn comparisons() {
    let source = "
        var two = 2;
        var three = 3;
        var string = \"string\";

        print two == two;
        print two == three;
        print two == string;
        print two > two;
        print two >= two;
        print two < three;
        print two == 2;
        print two < 3;
        print two < 2;
        print true == two;
    "
    .trim()
    .to_string();

    let expected_stderr = "".trim();
    let expected_stdout = "
true
false
false
false
true
true
true
true
false
true
    "
    .trim();

    let (stdout, stderr) = common::run(source);
    assert_eq!(expected_stderr, stderr.contents.trim());
    assert_eq!(expected_stdout, stdout.contents.trim());
}

#[test]
fn arithmetic() {
    let source = "
        print 2 * (1 + -4 / 2);
    "
    .trim()
    .to_string();

    let expected_stderr = "".trim();
    let expected_stdout = "
-2
    "
    .trim();

    let (stdout, stderr) = common::run(source);
    assert_eq!(expected_stderr, stderr.contents.trim());
    assert_eq!(expected_stdout, stdout.contents.trim());
}

#[test]
fn blocks() {
    let source = "
        print 1;
        {
            print 2;
        }
        print 3;
    "
    .trim()
    .to_string();

    let expected_stderr = "".trim();
    let expected_stdout = "
1
2
3
    "
    .trim();

    let (stdout, stderr) = common::run(source);
    assert_eq!(expected_stderr, stderr.contents.trim());
    assert_eq!(expected_stdout, stdout.contents.trim());
}

#[test]
fn locals() {
    let source = "
        var a = 1;
        print a;
        {
            var a;
            print a;
            a = 2;
            print a;

            var b = 3;
            print a;
            print b;
        }
        print a;
        {
            a = 3;
            var a = 2;
            print a;
        }
        print a;

        {
            var c = 6;
            var d = 7;
            print c + d;
        }
    "
    .trim()
    .to_string();

    let expected_stderr = "".trim();
    let expected_stdout = "
1
nil
2
2
3
1
2
3
13
    "
    .trim();

    let (stdout, stderr) = common::run(source);
    assert_eq!(expected_stderr, stderr.contents.trim());
    assert_eq!(expected_stdout, stdout.contents.trim());
}

#[test]
fn nested_locals() {
    let source = "   
    {
        var a = 1;
        {
            var a = 2;
            print a;
        }
        print a;
    }
    "
    .trim()
    .to_string();

    let expected_stderr = "".trim();
    let expected_stdout = "
2
1
    "
    .trim();

    let (stdout, stderr) = common::run(source);
    assert_eq!(expected_stderr, stderr.contents.trim());
    assert_eq!(expected_stdout, stdout.contents.trim());
}

#[test]
fn if_statements() {
    let source = "
    if (true) {
        print \"A\";
    } else {
        print \"B\";
    }

    if (1 < 2) {
        print \"C\";
    }

    if (1 >= 2) {
        print \"D\";
    } else {
        print \"E\";
    }
    "
    .trim()
    .to_string();

    let expected_stderr = "".trim();
    let expected_stdout = "
A
C
E
    "
    .trim();

    let (stdout, stderr) = common::run(source);
    assert_eq!(expected_stderr, stderr.contents.trim());
    assert_eq!(expected_stdout, stdout.contents.trim());
}

#[test]
fn while_statements() {
    let source = "
    var c = 0;
    while (c < 5) {
        print c;
        c = c + 1;
    }
    "
    .trim()
    .to_string();

    let expected_stderr = "".trim();
    let expected_stdout = "
0
1
2
3
4
    "
    .trim();

    let (stdout, stderr) = common::run(source);
    assert_eq!(expected_stderr, stderr.contents.trim());
    assert_eq!(expected_stdout, stdout.contents.trim());
}

#[test]
fn for_statements() {
    let source = "
for (var i = 0; i < 5; i = i + 2) {
    print i;
}

var i = 0;
for (; i < 10;) {
    print i;
    i = i + 4;
}

{
    for (var i = 0; i < 1;) {
        print i;
        i = 4;
    }
} 
    "
    .trim()
    .to_string();

    let expected_stderr = "".trim();
    let expected_stdout = "
0
2
4
0
4
8
0
    "
    .trim();

    let (stdout, stderr) = common::run(source);
    assert_eq!(expected_stderr, stderr.contents.trim());
    assert_eq!(expected_stdout, stdout.contents.trim());
}

#[test]
fn function_declaration() {
    let source = "
    fun foo() {
        print \"foo\";
    }
    print foo;

    {
        fun foo2(a) {
            print a;
        }
        print foo2;
    }
    "
    .trim()
    .to_string();

    let expected_stderr = "".trim();
    let expected_stdout = "
<fn: foo>
<fn: foo2>
    "
    .trim();

    let (stdout, stderr) = common::run(source);
    assert_eq!(expected_stderr, stderr.contents.trim());
    assert_eq!(expected_stdout, stdout.contents.trim());
}

#[test]
fn function_call() {
    let source = "
    fun foo() {
        print \"foo\";
    }
    foo();
    "
    .trim()
    .to_string();

    let expected_stderr = "".trim();
    let expected_stdout = "
foo
    "
    .trim();

    let (stdout, stderr) = common::run(source);
    assert_eq!(expected_stderr, stderr.contents.trim());
    assert_eq!(expected_stdout, stdout.contents.trim());
}

#[test]
fn function_arguments() {
    let source = "
    fun foo(a, b) {
        print a + b;
    }
    foo(1, 2);
    "
    .trim()
    .to_string();

    let expected_stderr = "".trim();
    let expected_stdout = "
3
    "
    .trim();

    let (stdout, stderr) = common::run(source);
    assert_eq!(expected_stderr, stderr.contents.trim());
    assert_eq!(expected_stdout, stdout.contents.trim());
}

#[test]
fn function_return() {
    let source = "
    fun foo(a, b) {
        return a + b;
    }
    var a = foo(1, 2);
    print a;
    "
    .trim()
    .to_string();

    let expected_stderr = "".trim();
    let expected_stdout = "
3
    "
    .trim();

    let (stdout, stderr) = common::run(source);
    assert_eq!(expected_stderr, stderr.contents.trim());
    assert_eq!(expected_stdout, stdout.contents.trim());
}

#[test]
fn function_local() {
    let source = "   
    fun foo(a, b) {
        fun bar(c, d) {
            print c + d;
        }
        bar(a, b);
    }
    foo(1, 2);
    "
    .trim()
    .to_string();

    let expected_stderr = "".trim();
    let expected_stdout = "
3
    "
    .trim();

    let (stdout, stderr) = common::run(source);
    assert_eq!(expected_stderr, stderr.contents.trim());
    assert_eq!(expected_stdout, stdout.contents.trim());
}

#[test]
fn function_recursive() {
    let source = "   
    fun foo(a, b) {
        print a;
        if (a > 0) {
            foo(a - 1);
        }
    }
    foo(2);
    "
    .trim()
    .to_string();

    let expected_stderr = "".trim();
    let expected_stdout = "
2
1
0
    "
    .trim();

    let (stdout, stderr) = common::run(source);
    assert_eq!(expected_stderr, stderr.contents.trim());
    assert_eq!(expected_stdout, stdout.contents.trim());
}

#[test]
fn closure_immediate() {
    let source = "
    fun foo(f) {
        fun bar(b) {
            return f + b;
        }
        return bar(3);
    }
    print foo(5);
    "
    .trim()
    .to_string();

    let expected_stderr = "".trim();
    let expected_stdout = "
8
    "
    .trim();

    let (stdout, stderr) = common::run(source);
    assert_eq!(expected_stderr, stderr.contents.trim());
    assert_eq!(expected_stdout, stdout.contents.trim());
}

#[test]
fn closure_deferred() {
    let source = "
    fun adder(a) {
        fun f(b) {
            return a + b;
        }
        return f;
    }
    var add2 = adder(2);
    print add2(1);
    print adder(2)(3);
    "
    .trim()
    .to_string();

    let expected_stderr = "".trim();
    let expected_stdout = "
3
5
    "
    .trim();

    let (stdout, stderr) = common::run(source);
    assert_eq!(expected_stderr, stderr.contents.trim());
    assert_eq!(expected_stdout, stdout.contents.trim());
}

#[test]
fn closure_nested() {
    let source = "
    fun foo(f) {
        fun bar(b) {
            fun baz(z) {
                return f + b + z;
            }
            return baz;
        }
        return bar(3);
    }
    print foo(5)(7);
    "
    .trim()
    .to_string();

    let expected_stderr = "".trim();
    let expected_stdout = "
15
    "
    .trim();

    let (stdout, stderr) = common::run(source);
    assert_eq!(expected_stderr, stderr.contents.trim());
    assert_eq!(expected_stdout, stdout.contents.trim());
}

#[test]
fn class_decl() {
    let source = "
    class foo {}
    print foo;
    "
    .trim()
    .to_string();

    let expected_stderr = "".trim();
    let expected_stdout = "
foo
    "
    .trim();

    let (stdout, stderr) = common::run(source);
    assert_eq!(expected_stderr, stderr.contents.trim());
    assert_eq!(expected_stdout, stdout.contents.trim());
}

#[test]
fn class_decl_nested() {
    let source = "
    {
        class foo {}
        print foo;
    }
    "
    .trim()
    .to_string();

    let expected_stderr = "".trim();
    let expected_stdout = "
foo
    "
    .trim();

    let (stdout, stderr) = common::run(source);
    assert_eq!(expected_stderr, stderr.contents.trim());
    assert_eq!(expected_stdout, stdout.contents.trim());
}

#[test]
fn instantiation() {
    let source = "
    class foo {}
    print foo();
    var f = foo();
    print f;
    "
    .trim()
    .to_string();

    let expected_stderr = "".trim();
    let expected_stdout = "
foo instance
foo instance
    "
    .trim();

    let (stdout, stderr) = common::run(source);
    assert_eq!(expected_stderr, stderr.contents.trim());
    assert_eq!(expected_stdout, stdout.contents.trim());
}

#[test]
fn closure_set_captured() {
    let source = "
    fun adder(a) {
        fun f(b) {
            a = a + 1;
            return a + b;
        }
        return f;
    }
    var add3 = adder(2);
    print add3(1);
    "
    .trim()
    .to_string();

    let expected_stderr = "".trim();
    let expected_stdout = "
4
    "
    .trim();

    let (stdout, stderr) = common::run(source);
    assert_eq!(expected_stderr, stderr.contents.trim());
    assert_eq!(expected_stdout, stdout.contents.trim());
}

#[test]
fn instance_get_set() {
    let source = "
    class foo {}
    var f1 = foo();
    var f2 = foo();

    f1.field_one = 123;
    f1.field_two = 456;
    f2.field_one = 789;

    print f1.field_one;
    print f1.field_two;
    print f2.field_one;

    f1.field_one = 101;
    print f1.field_one;
    print f1.field_two;
    "
    .trim()
    .to_string();

    let expected_stderr = "".trim();
    let expected_stdout = "
123
456
789
101
456
    "
    .trim();

    let (stdout, stderr) = common::run(source);
    assert_eq!(expected_stderr, stderr.contents.trim());
    assert_eq!(expected_stdout, stdout.contents.trim());
}
