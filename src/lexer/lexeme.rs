#[derive(Clone, Copy, Debug, PartialEq)]
pub enum LexemeKind {
  Eof,
  Whitespace,
  Comment,
  Unknown,

  LeftParen,
  RightParen,
  LeftBrace,
  RightBrace,
  LeftSquare,
  RightSquare,

  Plus,
  Minus,
  Asterisk,
  Slash,

  Identifier,
}

#[derive(Clone, Copy, Debug)]
pub struct Lexeme<'a> {
  pub kind: LexemeKind,
  pub code_span: &'a str,
  pub location: (usize, usize),
}
