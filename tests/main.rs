#![feature(unboxed_closures, fn_traits)]
use named_fn::named_fn;

fn regular_fn (a: i32, b: Vec<String>) {
    todo!()
}

#[named_fn]
fn as_str<'a> (str: &'a String) -> &'a str {
    return str as &str
}

struct Test {
    f: AsStr,
}

#[test]
fn test () {
    let hi = (AsStr::new())(&"alex".to_string());
}