mod lexer;

use lexer::{LexemeKind, Lexer};

fn main() {
  println!("Hello, space!");

  let mut stdout = std::io::stdout();
  let stdin = std::io::stdin();

  loop {
    use std::io::Write;

    let mut input = String::new();

    stdout.write(b"> ").expect("Couldn't write to stdout");
    stdout.flush().expect("Couldn't flush stdout");

    stdin
      .read_line(&mut input)
      .expect("Couldn't read line from stdin");

    let lexer = Lexer::new(&input);
    
    for lexeme in lexer {
      println!("{:?}", lexeme);
    }
  }
}
