#![feature(unboxed_closures, fn_traits)]

use named_fn::named_fn;

fn regular_fn (a: i32, b: Vec<String>) {
    todo!()
}

#[named_fn]
fn named_fn<T: ToString> (a: Vec<T>) -> Vec<String> {
    return a.into_iter().map(|x| x.to_string()).collect()
}

#[test]
fn test () {
    let hi = (NamedFn::new())(vec!["alex", "andreba"]);
}