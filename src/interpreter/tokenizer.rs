#[derive(PartialEq)]
#[derive(Eq)]
#[derive(Clone)]
#[derive(Hash)]
pub enum Token {
	Str(String),
	OPair,
	CPair,
	Digit(u128),
	Ident(String),
}

#[derive(PartialEq)]
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

/* // was needed for debug //;
pub fn print_tokens(v: &Vec<Token>) -> () {
	for i in 0..v.len() {
		println!("tokens[{}] => {}", i, token_show(&v[i]));
	}
} */

macro_rules! token_push_new {
	($token:ident, $kind:ident, $tokens:ident, $deref:ident, $next_kind:path) => {
		if $deref && $kind != TokenKind::Ident {
			eprintln!("ERR: tokenizer: {} given, ident expected for `!`",
								token_kind(&$kind));			
		}

		match $kind {
			TokenKind::Nil => {
				// VOID //;
			}

			TokenKind::Digit => {
				$tokens.push(Token::Digit($token.parse::<u128>().unwrap()));
			}

			TokenKind::Ident => {
				$tokens.push(Token::Ident($token.clone()));
				if $deref {
					$tokens.push(Token::CPair);
					$deref = false;
				}
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
		$kind = $next_kind;
	};

	($token:ident, $kind:ident, $tokens:ident, $deref:ident) => {
		token_push_new!($token, $kind, $tokens, $deref, TokenKind::Nil);
	};
}

pub fn tokenize(s: &str) -> Vec<Token> {
	let mut state: TokenizerState = TokenizerState::Common;
	let mut tmp_st: String = String::new();
	let mut kind: TokenKind = TokenKind::Nil;
	let mut tokens: Vec<Token> = Vec::new();
	let mut pairs: u16 = 0;
	let mut in_quote: bool = false;
	let mut quote_start: bool = false;
	let mut quote_level: usize = 0;
	let mut deref: bool = false;

	for c in s.chars().into_iter() {
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
						if !in_quote {
							token_push_new!(tmp_st, kind, tokens, deref);
						}
					}

					'\\' => {
						state = TokenizerState::InStringEscape;
					}

					_ => {
						if !in_quote {
							tmp_st.push(c);
						}
					}
				}
			}

			TokenizerState::InStringEscape => {
				if !in_quote {
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
				}
				
				state = TokenizerState::InString;
			}

			TokenizerState::Common => {
				match c {
					'\'' => {
						state = TokenizerState::InString;
						if !in_quote {
							kind = TokenKind::Str;
						}
					}

					'!' => {
						if !in_quote {
							token_push_new!(tmp_st, kind, tokens, deref, TokenKind::OPair);
							token_push_new!(tmp_st, kind, tokens, deref, TokenKind::Ident);
							tmp_st.push('!');
							token_push_new!(tmp_st, kind, tokens, deref);
							deref = true;
						}
					}

					' '|'\t'|'\n' => {
						if !in_quote {
							token_push_new!(tmp_st, kind, tokens, deref);
						}
					}

					'`' => {
						if !in_quote && !quote_start {
							quote_start = true;
						} else if quote_start {
							eprintln!(
								"ERR: tokenizer: '`' found while expected '(' to start quote");

							return vec![];
						}
					}

					'(' => {
						if quote_start {
							in_quote = true;
							quote_start = false;
							quote_level = 0;
						}
						
						if in_quote {
							quote_level += 1;
						} else {
							token_push_new!(tmp_st, kind, tokens, deref, TokenKind::OPair);
							token_push_new!(tmp_st, kind, tokens, deref);
						}
						pairs += 1;
					}

					')' => {
						if pairs == 0 {
							eprintln!("ERR: tokenizer: mismatched pair after {}",
												token_kind(&kind));

							return vec![];
						}

						if in_quote {
							quote_level -= 1;
							if quote_level == 0 {
								in_quote = false;
							}
						} else {
							token_push_new!(tmp_st, kind, tokens, deref, TokenKind::CPair);
							token_push_new!(tmp_st, kind, tokens, deref);
						}
						
						pairs -= 1;
					}

					';' => {
						if !in_quote {
							token_push_new!(tmp_st, kind, tokens, deref);
						}
						
						state = TokenizerState::InComment;
					}

					'0'..='9' => {
						if !in_quote {
							match kind {
								TokenKind::Nil => {
									kind = TokenKind::Digit;
									tmp_st.push(c);
								}

								TokenKind::Digit|TokenKind::Ident => {
									tmp_st.push(c);
								}

								_ => {
									eprintln!("ERR: tokenizer: uncompleted token {} before digit",
														token_kind(&kind));

									return vec![];
								}
							}
						}
					}

					_ => {
						if !in_quote {
							match kind {
								TokenKind::Nil => {
									kind = TokenKind::Ident;
									tmp_st.push(c);
								}

								TokenKind::Ident => {
									tmp_st.push(c);
								}

								TokenKind::Digit if c == '_' => {
									// VOID //;
								}

								_ => {
									eprintln!("ERR: tokenizer: uncompleted token {} before ident",
														token_kind(&kind));
									
									return vec![];
								}
							}
						}
					}
				}

				if c != '`' {
					if quote_start {
						eprintln!("ERR: tokenizer: '{}' expected '(' to start quote",
											c);

						return vec![];
					}
				}
			}
		}
	}

	match state {
		TokenizerState::InString|TokenizerState::InStringEscape => {
			eprintln!("ERR: tokenizer: unterminated string '{}'",
								tmp_st);

			return vec![];
		}

		_ => {
			// VOID //;
		}
	}

	if pairs != 0 {
		eprintln!("ERR: tokenizer: {} unclosed pairs",
							pairs);
		
		return vec![];
	}

	// print_tokens(&tokens);
	return tokens;
}
