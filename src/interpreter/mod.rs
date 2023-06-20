mod tokenizer;
use std::fs;
use std::collections::HashMap;

pub fn run_tokens(tokens: &Vec<tokenizer::Token>,
									depth: usize,
									args: &Vec<tokenizer::Token>,
									funcs: &mut HashMap<String, Vec<tokenizer::Token>>
) -> Vec<tokenizer::Token> {
	let mut stack: Vec<tokenizer::Token> = Vec::new();
	let mut ret: Vec<tokenizer::Token> = Vec::new();
	let mut catch_vec: Vec<Vec<tokenizer::Token>> = Vec::new();
	let mut deep: u16 = 0;
	let mut in_catch: bool = false;

	if depth > 0 {
		in_catch = tokens[0] == tokenizer::Token::Ident(String::from("if"))
						|| tokens[0] == tokenizer::Token::Ident(String::from("let"))
						|| tokens[0] == tokenizer::Token::Ident(String::from("each"));
	}

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
				if in_catch {
					catch_vec.push(stack.clone());
				} else {
					if depth == 0 {
						ret.clear();
					}
					
					for i in run_tokens(&stack, depth+1, args, funcs) {
						ret.push(i.clone());
					}
				}
				stack.clear();
			} else {
				stack.push(t.clone());
			}
		} else if *t == tokenizer::Token::OPair {
			deep = 1;
		} else if *t == tokenizer::Token::CPair {
			eprintln!("ERR: {}: mismatched closing pair",
								tokenizer::token_show(t));
		} else if in_catch {
			catch_vec.push(vec![t.clone()]);
		} else {
			ret.push(t.clone());
		}
	}

	if deep > 0 {
		eprintln!("ERR: {} unclosed pair(s)", deep);
	}

	if in_catch {
		let fun: tokenizer::Token = catch_vec.remove(0)[0].clone();

		match fun {
			tokenizer::Token::Ident(i) => {
				match i.as_str() {
					"if" => {
						if catch_vec.len() < 2 {
							eprintln!("ERR: too few arguments for `if`: {} given, 2..3 expected",
								catch_vec.len());

							return vec![];
						}
						
						let cond: &tokenizer::Token = &run_tokens(&catch_vec.remove(0),
																											depth+1, args, funcs)[0];
						match cond {
							tokenizer::Token::Digit(1) => {
								return run_tokens(&catch_vec.remove(0), depth+1, args, funcs);
							}

							tokenizer::Token::Digit(0) => {
								if catch_vec.len() > 1 {
									return run_tokens(&catch_vec.remove(1), depth+1, args, funcs);
								}
							}

							_ => {
								eprintln!("ERR: {}: expected 0 or 1 in `if` (1)",
													tokenizer::token_show(&cond));
							}
						}

						return vec![];
					}


					"each" => {
						let vl: usize = catch_vec.len();
						let ident: String;
						
						if vl != 3 {
							eprintln!("ERR: wrong arguments count for `each`: {} given, 3 expected",
								vl);
						}
						
						let code: Vec<tokenizer::Token> = catch_vec.pop().unwrap();
						let arr: Vec<tokenizer::Token> = run_tokens(&catch_vec.pop().unwrap(), depth+1, args, funcs);
						match catch_vec.pop().unwrap()[0].clone() {
							tokenizer::Token::Ident(i) => {
								if funcs.contains_key(&i) {
									eprintln!("ERR: `each {i}`: function is already defined");
									return vec![];
								}

								ident = i.clone();
							}

							name => {
								eprintln!("ERR: {}: wrong type for `each` (1)",
									tokenizer::token_show(&name));
								return vec![];
							}
						}

						for i in arr {
							funcs.insert(ident.clone(), vec![i]);
							for j in run_tokens(&code, depth+1, args, funcs) {
								ret.push(j);
							}
						}

						funcs.remove(&ident);
						return ret;
					}
					
					"let" => {
						if catch_vec.len() != 2 {
							eprintln!("ERR: wrong arguments count for `let`: {} given, 2 expected",
								catch_vec.len());

							return vec![];
						}

						match catch_vec.remove(0)[0].clone() {
							tokenizer::Token::Ident(i) => {
								if funcs.contains_key(&i) {
									eprintln!("ERR: `let {i}`: function is already defined");
									return vec![];
								}
								
								funcs.insert(i.clone(), catch_vec.remove(0));
							}

							name => {
								eprintln!("ERR: {}: wrong type for `let`",
									tokenizer::token_show(&name));
								return vec![];
							}
						}
					}

					_ => {
						unreachable!("what");
					}
				}
			}

			_ => {
				unreachable!("what");
			}
		}
	}

	if depth > 0 && ret.len() > 0 {
		let fun: tokenizer::Token = ret.remove(0);
		match fun {
			tokenizer::Token::Ident(i) => {
				match i.as_str() {
					"def?" => {
						if ret.len() != 1 {
							eprintln!("ERR: wrong arguments count for `def?`: {} given, 1 expected",
												ret.len());
						}

						match ret.pop().unwrap() {
							tokenizer::Token::Ident(i) => {
								return vec![tokenizer::Token::Digit(funcs.contains_key(&i) as u128)];
							}

							t => {
								eprintln!("ERR: {}: wrong type for `def?`",
													tokenizer::token_show(&t));

								return vec![];
							}
						}
					}
					
					"int" => {
						if ret.len() == 0 {
							eprintln!("ERR: too few arguments for `int`: 0 given, 1.. expected");

							return vec![];
						}

						let tmp: Vec<tokenizer::Token> = ret.clone();
						ret.clear();

						for i in tmp {
							match i {
								tokenizer::Token::Digit(_) => {
									ret.push(i.clone());
								}

								tokenizer::Token::Str(s) => {
									ret.push(
										tokenizer::Token::Digit(
											s.parse::<u128>()
												.expect(
													format!("ERR: {}: not valid integer for `int`", s).as_str())));
								}

								_ => {
									eprintln!("ERR: {}: invalid type for `int`",
														tokenizer::token_show(&i));
								}
							}
						}

						return ret;
					}

					"str" => {
						if ret.len() == 0 {
							eprintln!("ERR: too few arguments for `str`: 0 given, 1.. expected");

							return vec![];
						}

						let tmp: Vec<tokenizer::Token> = ret.clone();
						ret.clear();

						for i in tmp {
							match i {
								tokenizer::Token::Digit(d) => {
									ret.push(tokenizer::Token::Str(format!("{}", d)));
								}

								tokenizer::Token::Str(_) => {
									ret.push(i.clone());
								}

								_ => {
									eprintln!("ERR: {}: invalid type for `str`",
														tokenizer::token_show(&i));

									return vec![];
								}
							}
						}

						return ret;
					}
					
					"len" => {
						return vec![tokenizer::Token::Digit(ret.len() as u128)];
					}
					
					"nth" => {
						let vl: usize = ret.len();
						if vl < 2 {
							eprintln!("ERR: too few arguments for `nth`: {} given, 2.. expected",
												vl);

							return vec![];
						}

						let tok: tokenizer::Token = ret.remove(0);

						match tok {
							tokenizer::Token::Digit(d) => {
								let i: usize = d as usize;
								if i >= vl-1 {
									eprintln!("ERR: `nth {}`: index out of bounds, max {}",
														d, vl-1);
									return vec![];
								}

								return vec![ret[i].clone()];
							}

							_ => {
								eprintln!("ERR: {}: wrong type for `nth`",
													tokenizer::token_show(&tok));
								return vec![];
							}
						}
					}

					"rm" => {
						let vl: usize = ret.len();
						if vl < 2 {
							eprintln!("ERR: too few arguments for `rm`: {} given, 2.. expected",
												vl);

							return vec![];
						}

						let token: tokenizer::Token = ret.remove(0);
						match token {
							tokenizer::Token::Digit(d) => {
								let i: usize = d as usize;
								if i >= vl-1 {
									eprintln!("ERR: `nth {}`: index out of bounds, max {}",
														d, vl-1);

									return vec![];
								}

								ret.remove(i);
								return ret;
							}

							_ => {
								eprintln!("ERR: {}: wrong type for `rm`",
									tokenizer::token_show(&token));

								return vec![];
							}
						}
					}
					
					"=" => {
						let token: tokenizer::Token = ret.remove(0);
						match token {
							tokenizer::Token::Str(_)|tokenizer::Token::Digit(_) => {
								// VOID //;
							}

							_ => {
								eprintln!("ERR: {}: wrong type for `=` (1)",
													tokenizer::token_show(&token));
							}
						}
						
						let mut val: tokenizer::Token = tokenizer::Token::Digit(1);
						for t in ret {
							match t {
								tokenizer::Token::Str(_)|tokenizer::Token::Digit(_) => {
									// VOID //;
								}

								_ => {
									eprintln!("ERR: {}: wrong type for `=` (2)",
														tokenizer::token_show(&t));
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
									eprintln!("ERR: {}: wrong type for `pr`",
														tokenizer::token_show(&t));
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
									eprintln!("ERR: {}: wrong type for `prn`",
														tokenizer::token_show(&t));
								}
							}
						}

						println!("{}", st);
						return vec![];
					}

					">" => {
						if ret.len() < 2 {
							eprintln!("ERR: too few arguments for `>`: {} found, 2.. expected",
												ret.len());
							return vec![];
						}
						
						let token: tokenizer::Token = ret.remove(0);
						let mut r: u128 = 0;
						match token {
							tokenizer::Token::Digit(d) => r = d,
							_ => {
								eprintln!("ERR: {}: wrong type for `>` (1)",
													tokenizer::token_show(&token));
							} 
						}

						for t in ret {
							match t {
								tokenizer::Token::Digit(d) => {
									if d <= r {
										return vec![tokenizer::Token::Digit(0)];
									}

									r = d;
								}
								_ => {
									eprintln!("ERR: {}: wrong type for `>` (2)",
														tokenizer::token_show(&t));
								}
							}
						}

						return vec![tokenizer::Token::Digit(1)];
					}

					"<" => {
						if ret.len() < 2 {
							eprintln!("ERR: too few arguments for `>`: {} found, 2.. expected",
												ret.len());
							return vec![];
						}
						
						let token: tokenizer::Token = ret.remove(0);
						let mut r: u128 = 0;
						match token {
							tokenizer::Token::Digit(d) => r = d,
							_ => {
								eprintln!("ERR: {}: wrong type for `>` (1)",
													tokenizer::token_show(&token));
							} 
						}

						for t in ret {
							match t {
								tokenizer::Token::Digit(d) => {
									if d >= r {
										return vec![tokenizer::Token::Digit(0)];
									}

									r = d;
								}
								_ => {
									eprintln!("ERR: {}: wrong type for `>` (2)",
														tokenizer::token_show(&t));
								}
							}
						}

						return vec![tokenizer::Token::Digit(1)];
					}

					"+" => {
						if ret.len() == 0 {
							eprintln!("ERR: too few arguments for `+`: 0 found, 1.. expected");
							return vec![];
						}
						
						let token: tokenizer::Token = ret.remove(0);
						let mut r: u128 = 0;
						match token {
							tokenizer::Token::Digit(d) => r = d,
							_ => {
								eprintln!("ERR: {}: wrong type for `+` (1)",
													tokenizer::token_show(&token));
							} 
						}

						for t in ret {
							match t {
								tokenizer::Token::Digit(d) => r += d,
								_ => {
									eprintln!("ERR: {}: wrong type for `+` (2)",
														tokenizer::token_show(&t));
								}
							}
						}

						return vec![tokenizer::Token::Digit(r)];
					}

					"-" => {
						if ret.len() == 0 {
							eprintln!("ERR: too few arguments for `-`: 0 found, 1.. expected");
							return vec![];
						}
						
						let token: tokenizer::Token = ret.remove(0);
						let mut r: u128 = 0;
						match token {
							tokenizer::Token::Digit(d) => r = d,
							_ => {
								eprintln!("ERR: {}: wrong type for `-` (1)",
													tokenizer::token_show(&token));
							} 
						}

						for t in ret {
							match t {
								tokenizer::Token::Digit(d) => r -= d,
								_ => {
									eprintln!("ERR: {}: wrong type for `-` (2)",
														tokenizer::token_show(&t));
								}
							}
						}
						
						return vec![tokenizer::Token::Digit(r)];
					}

					"*" => {
						if ret.len() == 0 {
							eprintln!("ERR: too few arguments for `*`: 0 found, 1.. expected");
							return vec![];
						}
						
						let token: tokenizer::Token = ret.remove(0);
						let mut r: u128 = 0;
						match token {
							tokenizer::Token::Digit(d) => r = d,
							_ => {
								eprintln!("ERR: {}: wrong type for `*` (1)",
													tokenizer::token_show(&token));
							} 
						}

						for t in ret {
							match t {
								tokenizer::Token::Digit(d) => r *= d,
								_ => {
									eprintln!("ERR: {}: wrong type for `*` (2)",
														tokenizer::token_show(&t));
								}
							}
						}
						
						return vec![tokenizer::Token::Digit(r)];
					}

					"/" => {
						if ret.len() == 0 {
							eprintln!("ERR: too few arguments for `/`: 0 found, 1.. expected");
							return vec![];
						}
						
						let token: tokenizer::Token = ret.remove(0);
						let mut r: u128 = 0;
						match token {
							tokenizer::Token::Digit(d) => r = d,
							_ => {
								eprintln!("ERR: {}: wrong type for `/` (1)",
													tokenizer::token_show(&token));
							} 
						}

						for t in ret {
							match t {
								tokenizer::Token::Digit(d) => r /= d,
								_ => {
									eprintln!("ERR: {}: wrong type for `/` (2)",
														tokenizer::token_show(&t));
								}
							}
						}
						
						return vec![tokenizer::Token::Digit(r)];
					}

					"&"|"bit-and" => {
						if ret.len() == 0 {
							eprintln!("ERR: too few arguments for `&`: 0 found, 1.. expected");
							return vec![];
						}
						
						let token: tokenizer::Token = ret.remove(0);
						let mut r: u128 = 0;
						match token {
							tokenizer::Token::Digit(d) => r = d,
							_ => {
								eprintln!("ERR: {}: wrong type for `&` (1)",
													tokenizer::token_show(&token));
							} 
						}

						for t in ret {
							match t {
								tokenizer::Token::Digit(d) => r &= d,
								_ => {
									eprintln!("ERR: {}: wrong type for `&` (2)",
														tokenizer::token_show(&t));
								}
							}
						}
						
						return vec![tokenizer::Token::Digit(r)];
					}

					"|"|"bit-or" => {
						if ret.len() == 0 {
							eprintln!("ERR: too few arguments for `|`: 0 found, 1.. expected");
							return vec![];
						}
						
						let token: tokenizer::Token = ret.remove(0);
						let mut r: u128 = 0;
						match token {
							tokenizer::Token::Digit(d) => r = d,
							_ => {
								eprintln!("ERR: {}: wrong type for `|` (1)",
													tokenizer::token_show(&token));
							} 
						}

						for t in ret {
							match t {
								tokenizer::Token::Digit(d) => r |= d,
								_ => {
									eprintln!("ERR: {}: wrong type for `|` (2)",
														tokenizer::token_show(&t));
								}
							}
						}
						
						return vec![tokenizer::Token::Digit(r)];
					}

					"^"|"bit-xor" => {
						if ret.len() == 0 {
							eprintln!("ERR: too few arguments for `^`: 0 found, 1.. expected");
							return vec![];
						}
						
						let token: tokenizer::Token = ret.remove(0);
						let mut r: u128 = 0;
						match token {
							tokenizer::Token::Digit(d) => r = d,
							_ => {
								eprintln!("ERR: {}: wrong type for `&` (1)",
													tokenizer::token_show(&token));
							} 
						}

						for t in ret {
							match t {
								tokenizer::Token::Digit(d) => r ^= d,
								_ => {
									eprintln!("ERR: {}: wrong type for `&` (2)",
														tokenizer::token_show(&t));
								}
							}
						}
						
						return vec![tokenizer::Token::Digit(r)];
					}

					">>"|"bit-rshift" => {
						if ret.len() == 0 {
							eprintln!("ERR: too few arguments for `>>`: 0 found, 1.. expected");
							return vec![];
						}
						
						let token: tokenizer::Token = ret.remove(0);
						let mut r: u128 = 0;
						match token {
							tokenizer::Token::Digit(d) => r = d,
							_ => {
								eprintln!("ERR: {}: wrong type for `>>` (1)",
													tokenizer::token_show(&token));
							} 
						}

						for t in ret {
							match t {
								tokenizer::Token::Digit(d) => r >>= d,
								_ => {
									eprintln!("ERR: {}: wrong type for `>>` (2)",
														tokenizer::token_show(&t));
								}
							}
						}
						
						return vec![tokenizer::Token::Digit(r)];
					}

					"<<"|"bit-lshift" => {
						if ret.len() == 0 {
							eprintln!("ERR: too few arguments for `<<`: 0 found, 1.. expected");
							return vec![];
						}
						
						let token: tokenizer::Token = ret.remove(0);
						let mut r: u128 = 0;
						match token {
							tokenizer::Token::Digit(d) => r = d,
							_ => {
								eprintln!("ERR: {}: wrong type for `<<` (1)",
													tokenizer::token_show(&token));
							} 
						}

						for t in ret {
							match t {
								tokenizer::Token::Digit(d) => r <<= d,
								_ => {
									eprintln!("ERR: {}: wrong type for `<<` (2)",
														tokenizer::token_show(&t));
								}
							}
						}
						
						return vec![tokenizer::Token::Digit(r)];
					}
					
					"range"|".." => {
						if ret.len() == 0 {
							eprintln!("ERR: too few arguments for `{}`: 0 found, 1..3 expected",
												i);
							return vec![];
						}
						
						let token: tokenizer::Token = ret.remove(0);
						let mut start: u128 = 0;
						let mut end: u128 = 0;
						let mut step: u128 = 1;

						match token {
							tokenizer::Token::Digit(d0) => {
								end = d0;
								start = 0;
								if ret.len() > 0 {
									let token2: tokenizer::Token = ret.remove(0);

									match token2 {
										tokenizer::Token::Digit(d1) => {
											start = d0;
											end = d1;

											if ret.len() > 0 {
												let token3: tokenizer::Token = ret.remove(0);

												match token3 {
													tokenizer::Token::Digit(d2) => {
														step = d2;
													}

													_ => {
														eprintln!("ERR: {}: wrong type for `range` (3)",
																			tokenizer::token_show(&token3));
													}
												}
											}
										}

										_ => {
											eprintln!("ERR: {}: wrong type for `range` (2)",
																tokenizer::token_show(&token2));
										}
									}
								}

								if ret.len() > 0 {
									eprintln!("ERR: {}: wrong arguments count for `range`, expected 1..3, found {}",
										tokenizer::token_show(&ret[0]), 3+ret.len());
									return vec![];
								}
								
								ret.clear();	
								for i in (start..end).step_by(step as usize) {
									ret.push(tokenizer::Token::Digit(i));
								}

								return ret;
							}

							_ => {
								eprintln!("ERR: {}: wrong type for `range` (1)",
													tokenizer::token_show(&token));
							}
						}
					}

					"load" => {
						if ret.len() == 0 {
							eprintln!("ERR: too few arguments for `load`: 0 found, 1.. expected");
							return vec![];
						}
						
						let token: tokenizer::Token = ret.remove(0);

						match token {
							tokenizer::Token::Str(s) => {
								return run_file(s.as_str(), 0, &ret, funcs);
							}

							_ => {
								eprintln!("ERR: {}: wrong type for `load`",
													tokenizer::token_show(&token));
								return vec![];
							}
						}
					}

					"include" => {
						if ret.len() == 0 {
							eprintln!("ERR: too few arguments for `include`: 0 found, 1.. expected");

							return vec![];
						}

						for i in ret {
							match i {
								tokenizer::Token::Str(s)|tokenizer::Token::Ident(s) => {
									run_include(s.as_str(), funcs);	
								}

								_ => {
									eprintln!("ERR: {}: invalid type for `include`",
														tokenizer::token_show(&i));

									return vec![];
								}
							}
						}

						return vec![];
					}

					"eval" => {
						if ret.len() == 0 {
							eprintln!("ERR: too few arguments for `^`: 0 found, 1.. expected");
							return vec![];
						}
						
						let token: tokenizer::Token = ret.remove(0);

						match token {
							tokenizer::Token::Str(s) => {
								return run_str(s.as_str(), 0, &ret, funcs);
							}

							_ => {
								eprintln!("ERR: {}: wrong type for `eval`",
													tokenizer::token_show(&token));
							}
						}
					}

					"%%"|"args" => {
						return args.clone();
					}

					"%"|"arg" => {
						if ret.len() != 1 {
							eprintln!("ERR: wrong arguments count for `{}`: {} found, 1 expected",
								i, ret.len());
							
							return vec![];
						}

						let token: tokenizer::Token = ret.remove(0);
						match token {
							tokenizer::Token::Digit(d) => {
								if d as usize >= args.len() {
									eprintln!("ERR: {}: index out of bounds for `{}`",
														d, i);
									return vec![];
								}

								return vec![args[d as usize].clone()];
							}

							_ => {
								eprintln!("ERR: {}: wrong type for `{}`",
													tokenizer::token_show(&token), i);

								return vec![];
							}
						}
					}

					_ => {
						if !funcs.contains_key(&i) {
							eprintln!("ERR: `{}`: unknown function", i);
							return vec![];
						}

						return run_tokens(&funcs.get(&i).unwrap().clone(), depth+1, &ret, funcs);
					}
				}
			}

			tokenizer::Token::Str(_)|tokenizer::Token::Digit(_) => {
				ret.insert(0, fun);
				return ret;
			}
			
			_ => {
				eprintln!("ERR: {}: not a valid identifier",
									tokenizer::token_show(&fun));
			}
		}
	}

	return ret;
}

pub fn run_str(s: &str,
							 depth: usize,
							 args: &Vec<tokenizer::Token>,
							 funcs: &mut HashMap<String, Vec<tokenizer::Token>>
) -> Vec<tokenizer::Token> {
	let tokens: Vec<tokenizer::Token> = tokenizer::tokenize(s);
	return run_tokens(&tokens, depth, args, funcs);
}

pub fn run_file(f: &str,
								depth: usize,
								args: &Vec<tokenizer::Token>,
								funcs: &mut HashMap<String, Vec<tokenizer::Token>>
) -> Vec<tokenizer::Token> {
	return run_str(
		fs::read_to_string(f)
			.expect("failed to open file")
			.as_str(), depth, args, funcs);
}

pub fn run_include(f: &str, funcs: &mut HashMap<String, Vec<tokenizer::Token>>) -> () {
	run_file(("/usr/include/jll/".to_owned()+f+".jll").as_str(), 0, &vec![], funcs);
}

pub fn run_file_init(f: &str) -> Vec<tokenizer::Token> {
	let mut funcs: HashMap<String, Vec<tokenizer::Token>> = HashMap::new();
	return run_file(f, 0, &vec![], &mut funcs);
}
