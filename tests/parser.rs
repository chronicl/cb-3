use cb_3::{C1Lexer, C1Parser};
use std::fs;

#[test]
fn run_example() {
    let text = fs::read_to_string("tests/data/beispiel.c-1").unwrap();
    let mut lexer = C1Lexer::new(&text);
    while let Some(token) = lexer.current_token() {
        println!("{:?}", token);
        lexer.eat();
    }
    let result = C1Parser::parse(text.as_str());
    assert!(result.is_ok(), "Parse result: {}", result.err().unwrap());
}
