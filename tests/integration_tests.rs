mod common;

#[test]
fn empty_source() {
    let source = "".trim().to_string();

    let expected_stderr = "".trim();
    let expected_stdout = "".trim();

    let (stdout, stderr) = common::run(source);
    assert_eq!(expected_stdout, stdout.contents.trim());
    assert_eq!(expected_stderr, stderr.contents.trim());
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
    assert_eq!(expected_stdout, stdout.contents.trim());
    assert_eq!(expected_stderr, stderr.contents.trim());
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
    assert_eq!(expected_stdout, stdout.contents.trim());
    assert_eq!(expected_stderr, stderr.contents.trim());
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
nil
Hi
nil
"
    .trim();

    let (stdout, stderr) = common::run(source);
    assert_eq!(expected_stdout, stdout.contents.trim());
    assert_eq!(expected_stderr, stderr.contents.trim());
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
    assert_eq!(expected_stdout, stdout.contents.trim());
    assert_eq!(expected_stderr, stderr.contents.trim());
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
    assert_eq!(expected_stdout, stdout.contents.trim());
    assert_eq!(expected_stderr, stderr.contents.trim());
}
