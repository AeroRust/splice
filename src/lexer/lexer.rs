use crate::lexer::{Lexeme, LexemeKind};

#[derive(Debug)]
pub struct Lexer<'a> {
  input: &'a str,

  position: usize,
  line_number: usize,
  column_number: usize,

  has_hit_eof: bool,
}

impl<'a> Iterator for Lexer<'a> {
  type Item = Lexeme<'a>;

  fn next(&mut self) -> Option<Self::Item> {
    self.next_lexeme()
  }
}

impl<'a> Lexer<'a> {
  fn lexeme(&mut self, kind: LexemeKind, length: usize) -> Option<Lexeme<'a>> {
    let lexeme = Lexeme {
      kind,
      code_span: &self.input[self.position..self.position + length],
      location: (self.line_number, self.column_number),
    };

    self.position += length;
    self.column_number += length;

    Some(lexeme)
  }

  fn new_line(&mut self) -> Option<Lexeme<'a>> {
    let lexeme = Lexeme {
      kind: LexemeKind::Whitespace,
      code_span: &self.input[self.position..self.position + 1],
      location: (self.line_number, self.column_number),
    };

    self.position += 1;
    self.line_number += 1;
    self.column_number = 0;

    Some(lexeme)
  }

  pub fn new(input: &'a str) -> Lexer<'a> {
    Lexer {
      input,
      position: 0,
      line_number: 0,
      column_number: 0,
      has_hit_eof: false,
    }
  }

  pub fn next_lexeme(&mut self) -> Option<Lexeme<'a>> {
    let mut chars = self.input.chars();

    if let Some(ch) = chars.nth(self.position) {
      match ch {
        '(' => self.lexeme(LexemeKind::LeftParen, 1),
        ')' => self.lexeme(LexemeKind::RightParen, 1),
        '{' => self.lexeme(LexemeKind::LeftBrace, 1),
        '}' => self.lexeme(LexemeKind::RightBrace, 1),
        '[' => self.lexeme(LexemeKind::LeftSquare, 1),
        ']' => self.lexeme(LexemeKind::RightSquare, 1),

        '+' => self.lexeme(LexemeKind::Plus, 1),
        '-' => self.lexeme(LexemeKind::Minus, 1),
        '*' => self.lexeme(LexemeKind::Asterisk, 1),
        '/' => {
          let next_ch = chars.next()?;

          if next_ch == '/' {
            let mut length = 2;

            while let Some(comment_ch) = chars.next() {
              if comment_ch == '\n' {
                break;
              }

              length += 1;
            }

            self.lexeme(LexemeKind::Comment, length)
          }
          else {
            self.lexeme(LexemeKind::Slash, 1)
          }
        },

        '\n' => self.new_line(),
        '\t' => self.lexeme(LexemeKind::Whitespace, 1),

        ch @ _ if ch.is_whitespace() => self.lexeme(LexemeKind::Whitespace, 1),
        ch @ _ if ch.is_alphabetic() => {
          let mut length = 1;

          while let Some(ident_ch) = chars.next() {
            if !ident_ch.is_alphanumeric() && ident_ch != '_' {
              break;
            }

            length += 1;
          }

          self.lexeme(LexemeKind::Identifier, length)
        },

        _ => self.lexeme(LexemeKind::Unknown, 1),
      }
    } else {
      if self.has_hit_eof {
        None
      } else {
        self.has_hit_eof = true;

        Some(Lexeme {
          kind: LexemeKind::Eof,
          code_span: &self.input[self.input.len()..],
          location: (self.line_number, self.column_number),
        })
      }
    }
  }
}
