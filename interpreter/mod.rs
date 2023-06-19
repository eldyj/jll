mod tokenizer;
use std::fs;

pub fn run_tokens(tokens: &Vec<tokenizer::Token>,
									depth: u16,
									args: &Vec<tokenizer::Token>
) -> Vec<tokenizer::Token> {
	let mut stack: Vec<tokenizer::Token> = Vec::new();
	let mut ret: Vec<tokenizer::Token> = Vec::new();
	let mut deep: u16 = 0;

	for t in tokens {
		if deep > 0 {
			match t {
				tokenizer::Token::CPair => {
					deep -= 1;
				}

				tokenizer::Token::OPair => {
					deep += 1;
				}

				_ => {
					// VOID //;
				}
			}

			if deep == 0 {
				if depth == 0 {
					ret.clear();
				}
				
				for i in run_tokens(&stack, depth+1, args) {
					ret.push(i);
				}
				
				stack.clear();
			} else {
				match t {
					tokenizer::Token::OPair => stack.push(tokenizer::Token::OPair),
					tokenizer::Token::CPair => stack.push(tokenizer::Token::CPair),
					tokenizer::Token::Digit(d) => stack.push(tokenizer::Token::Digit(*d)),
					tokenizer::Token::Ident(i) => stack.push(tokenizer::Token::Ident(i.to_string())),
					tokenizer::Token::Str(s) => stack.push(tokenizer::Token::Str(s.to_string())),
				}
			}
		} else if *t == tokenizer::Token::OPair {
			deep = 1;
		} else if *t == tokenizer::Token::CPair {
			eprintln!("mismatched closing pair");
		} else {
			match t {
				tokenizer::Token::OPair => ret.push(tokenizer::Token::OPair),
				tokenizer::Token::CPair => ret.push(tokenizer::Token::CPair),
				tokenizer::Token::Digit(d) => ret.push(tokenizer::Token::Digit(*d)),
				tokenizer::Token::Ident(i) => ret.push(tokenizer::Token::Ident(i.to_string())),
				tokenizer::Token::Str(s) => ret.push(tokenizer::Token::Str(s.to_string())),
			}
		}
	}

	if deep > 0 {
		eprintln!("unclosed pair");
	}

	if depth > 0 && ret.len() > 0 {
		let fun: tokenizer::Token = ret.remove(0);
		match fun {
			tokenizer::Token::Ident(i) => {
				match i.as_str() {
					"=" => {
						let token: tokenizer::Token = ret.remove(0);
						match token {
							tokenizer::Token::Str(_)|tokenizer::Token::Digit(_) => {
								// VOID //;
							}

							_ => {
								eprintln!("mismatched type for `=`");
							}
						}
						
						let mut val: tokenizer::Token = tokenizer::Token::Digit(1);
						for t in ret {
							match t {
								tokenizer::Token::Str(_)|tokenizer::Token::Digit(_) => {
									// VOID //;
								}

								_ => {
									eprintln!("mismatched type for `=`");
								}
							}
							
							if t != token {
								val = tokenizer::Token::Digit(0);
								break;
							}
						}

						return vec![val];
					}

					"pr" => {
						let mut st: String = String::new();
						let rl: usize = ret.len();

						for i in 0..rl {
							let t: &tokenizer::Token = &ret[i];

							if i > 0 && i < rl {
								st.push(' ');
							}

							match t {
								tokenizer::Token::Str(s) => {
									st.push_str(&s);
								}

								tokenizer::Token::Digit(d) => {
									st.push_str(&format!("{d}"));
								}

								_ => {
									eprintln!("wrong type for `pr`");
								}
							}
						}

						print!("{}", st);
						return vec![];
					}

					"prn" => {
						let mut st: String = String::new();
						let rl: usize = ret.len();

						for i in 0..rl {
							let t: &tokenizer::Token = &ret[i];
							
							if i > 0 && i < rl {
								st.push(' ');
							}
							
							match t {
								tokenizer::Token::Str(s) => {
									st.push_str(&s);
								}

								tokenizer::Token::Digit(d) => {
									st.push_str(&format!("{d}"));
								}

								_ => {
									eprintln!("wrong type for `prn`");
								}
							}
						}

						println!("{}", st);
						return vec![];
					}

					"+" => {
						let token: tokenizer::Token = ret.remove(0);
						let mut r: u128 = 0;
						match token {
							tokenizer::Token::Digit(d) => r = d,
							_ => {
								eprintln!("wrong type for `+`");
							} 
						}

						for t in ret {
							match t {
								tokenizer::Token::Digit(d) => r += d,
								_ => {
									eprintln!("wrong type for `+`");
								}
							}
						}

						return vec![tokenizer::Token::Digit(r)];
					}

					"-" => {
						let token: tokenizer::Token = ret.remove(0);
						let mut r: u128 = 0;
						match token {
							tokenizer::Token::Digit(d) => r = d,
							_ => {
								eprintln!("wrong type for `-`");
							} 
						}

						for t in ret {
							match t {
								tokenizer::Token::Digit(d) => r -= d,
								_ => {
									eprintln!("wrong type for `-`");
								}
							}
						}
						
						return vec![tokenizer::Token::Digit(r)];
					}

					"*" => {
						let token: tokenizer::Token = ret.remove(0);
						let mut r: u128 = 0;
						match token {
							tokenizer::Token::Digit(d) => r = d,
							_ => {
								eprintln!("wrong type for `*`");
							} 
						}

						for t in ret {
							match t {
								tokenizer::Token::Digit(d) => r *= d,
								_ => {
									eprintln!("wrong type for `*`");
								}
							}
						}
						
						return vec![tokenizer::Token::Digit(r)];
					}

					"/" => {
						let token: tokenizer::Token = ret.remove(0);
						let mut r: u128 = 0;
						match token {
							tokenizer::Token::Digit(d) => r = d,
							_ => {
								eprintln!("wrong type for `/`");
							} 
						}

						for t in ret {
							match t {
								tokenizer::Token::Digit(d) => r /= d,
								_ => {
									eprintln!("wrong type for `/`");
								}
							}
						}
						
						return vec![tokenizer::Token::Digit(r)];
					}

					"range" => {
						let token: tokenizer::Token = ret.remove(0);
						let mut start: u128 = 0;
						let mut end: u128 = 0;

						match token {
							tokenizer::Token::Digit(d) => {
								if ret.len() > 0 {
									let token2: tokenizer::Token = ret.remove(0);
									match token2 {
										tokenizer::Token::Digit(d2) => {
											start = d;
											end = d2; 
										}

										_ => {
											eprintln!("wrong type for `range`");
										}
									}
								} else {
									end = d;
								}

								ret.clear();
								for i in start..end {
									ret.push(tokenizer::Token::Digit(i));
								}

								return ret;
							}

							_ => {
								eprintln!("invalid type for `range`");
							}
						}
					}

					_ => {
						eprintln!("unknown function `{}`", i);
					}
				}
			}

			_ => {
				eprintln!("{} is not a valid identifier", tokenizer::token_show(&fun));
			}
		}
	}

	return ret;
}

pub fn run_str(s: &str, depth: u16, args: &Vec<tokenizer::Token>) -> Vec<tokenizer::Token> {
	return run_tokens(&tokenizer::tokenize(s), depth, args);
}

pub fn run_file(f: &str, depth: u16, args: &Vec<tokenizer::Token>) -> Vec<tokenizer::Token> {
	return run_str(fs::read_to_string(f).expect("failed to open file").as_str(), depth, args);
}

pub fn run_file_init(f: &str) -> Vec<tokenizer::Token> {
	return run_file(f, 0, &vec![]);
}
