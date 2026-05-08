use solution::{eval, parse};

fn run(input: &str) -> Result<String, String> {
    match parse(input) {
        Err(e) => Err(format!("parse: {e}")),
        Ok(expr) => match eval(&expr) {
            Err(e) => Err(format!("eval: {e}")),
            Ok(v) => Ok(format!("{v}")),
        },
    }
}

#[test]
fn literal_int() {
    assert_eq!(run("42"), Ok("42".to_string()));
}

#[test]
fn literal_bool_true() {
    assert_eq!(run("true"), Ok("true".to_string()));
}

#[test]
fn literal_bool_false() {
    assert_eq!(run("false"), Ok("false".to_string()));
}

#[test]
fn unary_minus() {
    assert_eq!(run("-5"), Ok("-5".to_string()));
}

#[test]
fn arithmetic_precedence() {
    assert_eq!(run("1 + 2 * 3"), Ok("7".to_string()));
}

#[test]
fn parens_override_precedence() {
    assert_eq!(run("(1 + 2) * 3"), Ok("9".to_string()));
}

#[test]
fn subtraction_left_associative() {
    assert_eq!(run("10 - 3 - 2"), Ok("5".to_string()));
}

#[test]
fn division() {
    assert_eq!(run("20 / 4"), Ok("5".to_string()));
}

#[test]
fn less_than() {
    assert_eq!(run("1 < 2"), Ok("true".to_string()));
}

#[test]
fn equal_bools() {
    assert_eq!(run("true == true"), Ok("true".to_string()));
}

#[test]
fn and_op() {
    assert_eq!(run("true and false"), Ok("false".to_string()));
}

#[test]
fn or_op() {
    assert_eq!(run("false or true"), Ok("true".to_string()));
}

#[test]
fn not_op() {
    assert_eq!(run("not true"), Ok("false".to_string()));
}

#[test]
fn if_true_branch() {
    assert_eq!(run("if true then 1 else 2"), Ok("1".to_string()));
}

#[test]
fn if_with_comparison() {
    assert_eq!(run("if 1 < 2 then 10 else 20"), Ok("10".to_string()));
}

#[test]
fn let_basic() {
    assert_eq!(run("let x = 5 in x + 1"), Ok("6".to_string()));
}

#[test]
fn let_nested() {
    assert_eq!(run("let x = 5 in let y = 10 in x + y"), Ok("15".to_string()));
}

#[test]
fn let_shadowing() {
    assert_eq!(run("let x = 1 in let x = 2 in x"), Ok("2".to_string()));
}

#[test]
fn parse_error_incomplete() {
    let r = run("1 +");
    assert!(r.is_err() && r.as_ref().unwrap_err().starts_with("parse"), "got {r:?}");
}

#[test]
fn parse_error_let_missing_equals() {
    let r = run("let x in x");
    assert!(r.is_err() && r.as_ref().unwrap_err().starts_with("parse"), "got {r:?}");
}

#[test]
fn type_error_add_int_bool() {
    let r = run("1 + true");
    assert!(r.is_err() && r.as_ref().unwrap_err().starts_with("eval"), "got {r:?}");
}

#[test]
fn type_error_if_condition_int() {
    let r = run("if 1 then 2 else 3");
    assert!(r.is_err() && r.as_ref().unwrap_err().starts_with("eval"), "got {r:?}");
}

#[test]
fn divide_by_zero() {
    let r = run("1 / 0");
    assert!(r.is_err() && r.as_ref().unwrap_err().starts_with("eval"), "got {r:?}");
}

#[test]
fn unbound_identifier() {
    let r = run("x + 1");
    assert!(r.is_err() && r.as_ref().unwrap_err().starts_with("eval"), "got {r:?}");
}
