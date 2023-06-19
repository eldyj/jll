#[derive(PartialEq)]
pub enum Token {
	Str(String),
	OPair,
	CPair,
	Digit(u128),
	Ident(String),
}

pub enum TokenKind {
	Nil,
	Str,
	OPair,
	CPair,
	Digit,
	Ident,
}

enum TokenizerState {
	Common,
	InString,
	InStringEscape,
	InComment,
}

pub fn token_kind(k: &TokenKind) -> &'static str {
	match *k {
		TokenKind::Nil => "Nil",
		TokenKind::Str => "Str",
		TokenKind::OPair => "OPair",
		TokenKind::CPair => "CPair",
		TokenKind::Digit => "Digit",
		TokenKind::Ident => "Ident",
	}
}

pub fn token_show(t: &Token) -> String {
	match t {
		Token::Str(s) => format!("Str('{s}')"),
		Token::OPair => String::from("OPair"),
		Token::CPair => String::from("CPair"),
		Token::Digit(d) => format!("Digit({d})"),
		Token::Ident(i) => format!("Ident('{i}')"),
	}
}

pub fn print_tokens(v: &Vec<Token>) -> () {
	for i in 0..v.len() {
		println!("tokens[{}] => {}", i, token_show(&v[i]));
	}
}

macro_rules! token_push {
	($token:ident, $kind:ident, $tokens:ident) =>	{
		match $kind {
			TokenKind::Nil => {
				// VOID //;
			}

			TokenKind::Digit => {
				$tokens.push(Token::Digit($token.parse::<u128>().expect("atoi failed")));
			}

			TokenKind::Ident => {
				$tokens.push(Token::Ident($token.clone()));
			}

			TokenKind::OPair => {
				$tokens.push(Token::OPair);
			}

			TokenKind::CPair => {
				$tokens.push(Token::CPair);
			}

			TokenKind::Str => {
				$tokens.push(Token::Str($token.clone()));
			}
		}
		
		$token = String::new();
		$kind = TokenKind::Nil;
	}
}

pub fn tokenize(s: &str) -> Vec<Token> {
	let mut state: TokenizerState = TokenizerState::Common;
	let mut tmp_st: String = String::new();
	let mut kind: TokenKind = TokenKind::Nil;
	let mut tokens: Vec<Token> = Vec::new();
	let mut pairs: u16 = 0;

	for c in s.chars() {
		match state {
			TokenizerState::InComment => {
				if c == '\n' {
					state = TokenizerState::Common;
				}
			}

			TokenizerState::InString => {
				match c {
					'\'' => {
						state = TokenizerState::Common;
						token_push!(tmp_st, kind, tokens);
					}

					'\\' => {
						state = TokenizerState::InStringEscape;
					}

					_ => {
						tmp_st.push(c);
					}
				}
			}

			TokenizerState::InStringEscape => {
				match c {
					't' => {
						tmp_st.push('\t');
					}

					'n' => {
						tmp_st.push('\n');
					}

					_ => {
						tmp_st.push(c);
					}
				}
				
				state = TokenizerState::InString;
			}

			TokenizerState::Common => {
				match c {
					'\'' => {
						state = TokenizerState::InString;
						kind = TokenKind::Str;
					}

					' '|'\t'|'\n' => {
						token_push!(tmp_st, kind, tokens);
					}

					'(' => {
						token_push!(tmp_st, kind, tokens);
						kind = TokenKind::OPair;
						pairs += 1;
						token_push!(tmp_st, kind, tokens);
					}

					')' => {
						token_push!(tmp_st, kind, tokens);

						if pairs == 0 {
							eprintln!("mismatched pair after {}", token_kind(&kind));
						} else {
							pairs -= 1;
						}
						
						kind = TokenKind::CPair;
						token_push!(tmp_st, kind, tokens);
					}

					';' => {
						token_push!(tmp_st, kind, tokens);
						state = TokenizerState::InComment;
					}

					'0'..='9' => {
						match kind {
							TokenKind::Nil => {
								kind = TokenKind::Digit;
								tmp_st.push(c);
							}

							TokenKind::Digit|TokenKind::Ident => {
								tmp_st.push(c);
							}

							_ => {
								eprintln!("uncompleted token {} before digit", token_kind(&kind));
							}
						}
					}

					_ => {
						match kind {
							TokenKind::Nil => {
								kind = TokenKind::Ident;
								tmp_st.push(c);
							}

							TokenKind::Ident => {
								tmp_st.push(c);
							}

							_ => {
								eprintln!("uncompleted token {} before ident", token_kind(&kind));
							}
						}
					}
				}
			}
		}
	}

	match state {
		TokenizerState::InString|TokenizerState::InStringEscape => {
			eprintln!("unterminated string");
		}

		_ => {
			// VOID //;
		}
	}

	if pairs > 0 {
		eprintln!("unclosed pair after {}", token_kind(&kind));
	}

	return tokens;
}
