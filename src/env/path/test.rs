use alloc::string::{String, ToString};
use alloc::vec::Vec;
use crate::env::split_paths;

#[test]
fn split_paths_same_as_std() {
    extern crate std;
    fn compare_string(test_string: &str) {
        let q1: Vec<String> = std::env::split_paths(test_string).map(|t| t.display().to_string()).collect();
        let q2: Vec<String> = split_paths(test_string).map(|t| t.display().to_string()).collect();
        assert_eq!(q1, q2, "{test_string:?} fails");
    }

    compare_string("");
    compare_string("this is a /string\\");
    compare_string("this;is;some;strings");
    compare_string("this;is;some;str\\ings;but;we;;have;;;;a;b;unc\\h;;;of;empty;stuff");
    compare_string("\"quotes with semicolons ;;;;\";\"yes\"");
    compare_string("some\"funny string\"with some\"inl\\ine;;;;;stuff\";wo\\w");
    compare_string("some\"funny string\"with some\"inl\\ine;;;;;stuff;wo\\w I forgot to add a quote ooops;;;; guess the rest of the string is in the path :(");
}
