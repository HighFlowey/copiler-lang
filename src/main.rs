use crate::frontend::{lexer::Lexer, parser::Parser};

pub mod frontend;
pub mod util;

const SRC: &str = r#"

let a = "This is a string"

{
    let c = 'c'

    {
        let _long_str = 5000
    }
}

let b = 25.5234

"#;

fn main() {
    let mut lexer = Lexer::new(SRC);
    let tokens = lexer.load();
    let mut parser = Parser::new(tokens);
    let expression = parser.load();

    println!("{:#?}", expression);
}
