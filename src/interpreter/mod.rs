mod tokenizer;
use std::fs;
use std::mem;
use std::env;
use std::io::{self, Write};
use std::collections::HashMap;
use std::process;

macro_rules! unless {
	(let $pat:pat = $expr:expr, $block:block) => {
		if let $pat = $expr {} else $block
	};

	($expr:expr, $block:block) => {
		if !$expr $block
	};
}

macro_rules! make_err {
	(argc, $i:ident, $given:expr, $emin:expr, $emax:expr) => {
		eprintln!("ERR: wrong argc for `{}`: {} given, {} expected",
							$i, $given,
							if $emin == $emax {
								format!("{}", $emin)
							} else if $emax == 0 {
								format!("{}..", $emin)
							} else {
								format!("{}..={}", $emin, $emax)
							});

		process::exit(1);
	};

	(argcn, $i:ident, $given:expr, $expt:expr) => {
		make_err!(argc, $i, $given, $expt, $expt);
	};

	(argcf, $i:ident, $given:expr, $emin:expr) => {
		make_err!(argc, $i, $given, $emin, 0);
	};

	(argcm, $i:ident,$given:expr, $emax:expr) => {
		make_err!(argc, $i, $given, 0, $emax);
	};

	(argt, $i:ident, $t:ident, $n:expr) => {
		eprintln!("ERR: wrong argument type for `{}`({}): {}",
							$i, $n, tokenizer::token_show(&$t));

		process::exit(2);
	};

	(zerodiv, $i:ident) => {
		eprintln!("ERR: `{}`: division by zero",
							$i);

		process::exit(3);
	};

	(notident, $i:ident) => {
		eprintln!("ERR: invalid type for call: `{}",
							tokenizer::token_show(&$i));

		process::exit(4);
	};

	(indexerr, $i:ident, $given:expr, $max:expr) => {
		eprintln!("ERR: `{}`; index out of bounds: {} given, ..{} expected",
							$i, $given, $max);

		process::exit(5);
	};

	(unknown_ident, $i:ident) => {
		eprintln!("ERR: not a function or binding/variable: `{}`",
							$i);

		process::exit(6);
	};

	(value, $i:ident, $given:ident, $expected:expr) => {
		eprintln!("ERR: value error for `{}`: {} given, {} expected",
							$i, tokenizer::token_show(&$given), $expected);

		process::exit(7);
	};

	(redef, $i:ident, $f:ident) => {
		eprintln!("ERR: in `{}`: redefenition of function `{}`",
							$i, $f);

		process::exit(8);
	};
}

pub fn run_tokens(tokens: &Vec<tokenizer::Token>,
									depth: usize,
									args: &Vec<tokenizer::Token>,
									funcs: &mut HashMap<String, Vec<tokenizer::Token>>,
									lambdas: &mut usize,
									vars: &mut HashMap<String, Vec<tokenizer::Token>>,
) -> Vec<tokenizer::Token> {
	let mut stack: Vec<tokenizer::Token> = Vec::new();
	let mut ret: Vec<tokenizer::Token> = Vec::new();
	let mut catch_vec: Vec<Vec<tokenizer::Token>> = Vec::new();
	let mut deep: u16 = 0;
	let mut in_catch: bool = false;

	if depth > 0 {
		if let tokenizer::Token::Ident(ref s) = tokens[0] {
			match s.as_str() {
				"if"|"let"|"bind"|"each"|"case"|"mut"|"set"|"while" => {
					in_catch = true;
				}

				"lambda" => {
					let lambda_ident: tokenizer::Token =
							tokenizer::Token::Ident(format!("{}l", *lambdas));

					*lambdas += 1;

					let mut tmp: Vec<tokenizer::Token> = tokens.clone();
					tmp.remove(0);
					tmp.insert(0, lambda_ident.clone());
					tmp.insert(0, tokenizer::Token::Ident(String::from("let")));
					run_tokens(&tmp, depth+1, args, funcs, lambdas, vars);
					return vec![lambda_ident.clone()];
				}

				"cond" => {
					let mut tmp: Vec<tokenizer::Token> = tokens.clone();
					tmp.insert(0, tokenizer::Token::Digit(1));
					tmp.insert(0, tokenizer::Token::Ident(String::from("case")));
					return run_tokens(&tmp, depth+1, args, funcs, lambdas, vars);
				}

				_ => {
					// not a macro //;
				}
			}
		}
	}

	for t in tokens.into_iter() {
		let tmp_t: tokenizer::Token = t.clone();
		if deep > 0 {
			match tmp_t {
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
						ret = run_tokens(&stack, depth+1, args, funcs, lambdas, vars);
					} else {
						for i in run_tokens(&stack, depth+1, args, funcs, lambdas, vars)
							.into_iter() {
							ret.push(i.clone());
						}
					}
				}
				stack.clear();
			} else {
				stack.push(tmp_t);
			}
		} else if tmp_t == tokenizer::Token::OPair {
			deep = 1;
		} else if in_catch {
			catch_vec.push(vec![tmp_t]);
		} else {
			ret.push(tmp_t);
		}
	}

	if in_catch {
		let fun: tokenizer::Token = catch_vec.remove(0).remove(0);

		match fun {
			tokenizer::Token::Ident(i) => {
				match i.as_str() {
					"while" => {
						let vl: usize = catch_vec.len();
						if vl != 2 {
							make_err!(argcn, i, vl, 2);
						}

						let code: Vec<tokenizer::Token> = catch_vec.pop().unwrap();
						let cond: Vec<tokenizer::Token> = catch_vec.pop().unwrap();

						loop {
							let cond_r: tokenizer::Token = run_tokens(
								&cond,
								depth+1,
								args,
								funcs,
								lambdas,
								vars
							).remove(0);

							match cond_r {
								tokenizer::Token::Digit(0) => {
									break
								}

								tokenizer::Token::Digit(1) => {
									let code_r: Vec<tokenizer::Token> = run_tokens(
										&code,
										depth+1,
										args,
										funcs,
										lambdas,
										vars
									);

									for i in code_r.into_iter() {
										ret.push(i);
									}
								}

								_ => {
									make_err!(argt, i, cond_r, "cond");
								}
							}
						}

						return ret;
					}

					"set" => {
						let vl: usize = catch_vec.len();
						if vl != 2 {
							make_err!(argcn, i, vl, 2);
						}

						let res: Vec<tokenizer::Token> = run_tokens(
							&catch_vec.pop().unwrap(),
							depth+1,
							args,
							funcs,
							lambdas,
							vars
						);

						
						let name_t: tokenizer::Token = catch_vec.pop().unwrap().remove(0);
						if let tokenizer::Token::Ident(s) = name_t {
							if !vars.contains_key(&s) {
								make_err!(unknown_ident, s);
							}

							vars.insert(s, res);
							return vec![];
						}

						make_err!(argt, i, name_t, 1);
					}

					"mut" => {
						let vl: usize = catch_vec.len();
						if vl != 2 {
							make_err!(argcn, i, vl, 2);
						}

						let res: Vec<tokenizer::Token> = run_tokens(
							&catch_vec.pop().unwrap(),
							depth+1,
							args,
							funcs,
							lambdas,
							vars
						);

						let name_t: tokenizer::Token = catch_vec.pop().unwrap().remove(0);
						if let tokenizer::Token::Ident(s) = name_t {
							if vars.contains_key(&s) {
								make_err!(redef, i, s);
							}

							vars.insert(s, res);
							return vec![];
						} 

						make_err!(argt, i, name_t, 1);
					}

					"case" => {
						let vl: usize = catch_vec.len();
						if vl < 3 {
							make_err!(argc, i, vl, 3, 0);
						}

						let val: Vec<tokenizer::Token> =
								run_tokens(&catch_vec.remove(0), depth+1, args, funcs, lambdas, vars);

						for pair in catch_vec.chunks(2) {
							match pair {
								[case, code]
								if run_tokens(case, depth+1, args, funcs, lambdas, vars) == val => {
									return run_tokens(code, depth+1, args, funcs, lambdas, vars);
								}

								[code] => {
									return run_tokens(code, depth+1, args, funcs, lambdas, vars);
								}

								_ => {
									// not matched //;
								}
							}
						}

						return vec![];
					}
					
					"if" => {
						let vl: usize = catch_vec.len();
						if vl < 2 || vl > 3 {
							make_err!(argc, i, vl, 2, 3);
						}
						
						let cond: &tokenizer::Token = &run_tokens(&catch_vec.remove(0),
																											depth+1, args, funcs,
																											lambdas, vars)[0];
						match cond {
							tokenizer::Token::Digit(1) => {
								return run_tokens(&catch_vec.remove(0), depth+1, args, funcs,
																	lambdas, vars);
							}

							tokenizer::Token::Digit(0) => {
								if catch_vec.len() > 1 {
									return run_tokens(&catch_vec.remove(1), depth+1, args,
																		funcs, lambdas, vars);
								}
							}

							_ => {
								make_err!(value, i, cond, "0|1");
							}
						}

						return vec![];
					}


					"each" => {
						let vl: usize = catch_vec.len();
						
						if vl != 3 {
							make_err!(argc, i, vl, 3, 3);
						}
						
						let code: Vec<tokenizer::Token> = catch_vec.pop().unwrap();
						let arr: Vec<tokenizer::Token> =
								run_tokens(&catch_vec.pop().unwrap(), depth+1, args,
													 funcs, lambdas, vars);

						let tok: tokenizer::Token = catch_vec.pop().unwrap().remove(0);

						unless!(let tokenizer::Token::Ident(_) = tok, {
							make_err!(argt, i, tok, 1);
						});

						for i in arr.into_iter() {
							let mut tmp: Vec<tokenizer::Token> = code.clone();
							let mut k: usize = 0;
							while k < tmp.len() {
								if tmp[k] == tok {
									tmp[k] = i.clone();
								}
								k += 1;
							}
							
							for j in run_tokens(&tmp, depth+1, args, funcs, lambdas, vars)
								.into_iter() {
								ret.push(j);
							}
						}

						return ret;
					}

					"bind" => {
						let vl: usize = catch_vec.len();
						
						if vl != 3 {
							make_err!(argcn, i, vl, 3);
						}
						
						//let ident: String;
						
						let mut code: Vec<tokenizer::Token> = catch_vec.pop().unwrap();
						let val: Vec<tokenizer::Token> =
								run_tokens(&catch_vec.pop().unwrap(), depth+1, args,
													 funcs, lambdas, vars);

						let tok: tokenizer::Token = catch_vec.pop().unwrap().remove(0);

						unless!(let tokenizer::Token::Ident(_) = tok, {
							make_err!(argt, i, tok, 1);
						});

						let fval: Vec<tokenizer::Token> =
								run_tokens(&val, depth+1, args, funcs, lambdas, vars);

						let mut k: usize = 0;
						while k < code.len() {
							if code[k] == tok {
								code.remove(k);
								for (index, t) in fval.clone().into_iter().enumerate() {
									code.insert(k+index, t);
								}
							}
							k += 1;
						}

						return run_tokens(&code, depth+1, args, funcs, lambdas, vars);
					}
					
					"let" => {
						let vl: usize = catch_vec.len();
						if vl < 2  {
							make_err!(argcf, i, vl, 2);
						}

						let mut code: Vec<tokenizer::Token> = catch_vec.pop().unwrap();
						let funn: tokenizer::Token = catch_vec.remove(0).remove(0);

						if let tokenizer::Token::Ident(fi) = funn {
							if funcs.contains_key(&fi) {
								make_err!(redef, i, fi);
							}

							if vl > 2 {
								let tok: tokenizer::Token =
										catch_vec.pop().unwrap().remove(0);

								if let tokenizer::Token::Ident(_) = tok {
									let argc: usize = catch_vec.len();
									let mut rest_toks: Vec<tokenizer::Token> = Vec::new();

									for _ in 0..argc {
										rest_toks.push(tokenizer::Token::OPair);
										rest_toks
											.push(tokenizer::Token::Ident(String::from("rm")));
										rest_toks.push(tokenizer::Token::Digit(0));
									}

									rest_toks.push(tokenizer::Token::OPair);
									rest_toks.push(tokenizer::Token::Ident(String::from("%%")));
									rest_toks.push(tokenizer::Token::CPair);

									for _ in 0..argc {
										rest_toks.push(tokenizer::Token::CPair);
									}

									let mut j: usize = 0;
									while j < code.len() {
										if code[j] == tok.clone() {
											code.remove(j);
											for (o, item) in rest_toks.clone().into_iter()
												.enumerate() {
												code.insert(j+o, item);
											}

											j += rest_toks.len();
										}
										j += 1;
									}

									for _ in 0..argc {
										let arg_i: usize = catch_vec.len()-1;
										let tok_arg: tokenizer::Token =
												catch_vec.pop().unwrap().remove(0);
										
										let arg_vec: Vec<tokenizer::Token> = vec![
											tokenizer::Token::OPair,
											tokenizer::Token::Ident(String::from("%")),
											tokenizer::Token::Digit(arg_i as u128),
											tokenizer::Token::CPair,
										];

										if let tokenizer::Token::Ident(_) = tok_arg {
											let mut k: usize = 0;
											while k < code.len() {
												if code[k] == tok_arg.clone() {
													code.remove(k);
													for (o, item) in arg_vec.clone().into_iter()
														.enumerate() {
														code.insert(k+o, item);
													}

													k += 4;
												}
												k += 1;
											}
										} else {
											make_err!(argt, i, tok_arg, arg_i+1);
										}
									}
								} else {
									make_err!(argt, i, tok, "rest");
								}
							}
								
							funcs.insert(fi.clone(), code);
							return vec![];
						}

						make_err!(argt, i, funn, 1);
					}

					_ => {
						unreachable!("what 2");
					}
				}
			}

			_ => {
				unreachable!("what 1");
			}
		}
	}

	if depth > 0 && ret.len() > 0 {
		let fun: tokenizer::Token = ret.remove(0);
		match fun {
			tokenizer::Token::Ident(i) => {
				match i.as_str() {
					"as-int" => {
						let vl: usize = ret.len();
						if vl != 1 {
							make_err!(argcn, i, vl, 1);
						}

						let t = ret.pop().unwrap();
						if let tokenizer::Token::Str(ref s) = t {
							if s.len() == 1 {
								return vec![
									tokenizer::Token::Digit(s.chars().nth(0).unwrap() as u128)
								];
							} else {
								make_err!(value, i, t, "single char");
							}
						} else {
							make_err!(argt, i, t, 1);
						}
					}

					"as-str" => {
						let vl: usize = ret.len();
						if vl == 0 {
							make_err!(argcf, i, 0, 1);
						}

						let mut tmp: Vec<u8> = Vec::new();
						for (index, t) in ret.into_iter().enumerate() {
							if let tokenizer::Token::Digit(b) = t {
								tmp.push(b as u8);
								continue
							}

							make_err!(argt, i, t, index+1);
						}

						return vec![
							tokenizer::Token::Str(String::from_utf8_lossy(tmp.as_slice())
								.to_string())
						];
					}

					"as-char" => {
						let vl: usize = ret.len();
						if vl != 1 {
							make_err!(argcn, i, vl, 1);
						}

						let t = ret.pop().unwrap();
						if let tokenizer::Token::Digit(d) = t{
							return vec![
								tokenizer::Token::Str(String::from_utf8_lossy(&[d as u8])
									.to_string())
							];
						} else {
							make_err!(argt, i, t, 1);
						}
					}

					"get-env" => {
						let vl: usize = ret.len();
						if vl != 1 {
							make_err!(argcn, i, vl, 1);
						}

						let tok: tokenizer::Token = ret.pop().unwrap();
						if let tokenizer::Token::Str(s) = tok {
							return vec![tokenizer::Token::Str(
								if let Ok(r) = env::var(s) {
									r
								} else {
									String::new()
								}
							)];
						} else {
							make_err!(argt, i, tok, 1);
						}
					}

					"input" => {
						let vl: usize = ret.len();
						if vl == 0 {
							make_err!(argcf, i, 0, 1);
						}

						let mut tmp: Vec<tokenizer::Token> = Vec::new();
						mem::swap(&mut ret, &mut tmp);

						let stdin = io::stdin();
						let mut stdout = io::stdout();
						for (index, t) in tmp.into_iter().enumerate() {
							if let tokenizer::Token::Str(s) = t {
								print!("{}", s);
								stdout.flush().expect("stdout flush failed");
								let mut rs: String = String::new();
								stdin.read_line(&mut rs).expect("stdin read_line failed");
								ret.push(tokenizer::Token::Str(rs.trim().to_string()));
								continue
							}

							make_err!(argt, i, t, index+1);
						}

						return ret;
					}

					"file-read" => {
						let vl: usize = ret.len();
						if vl == 0 {
							make_err!(argcf, i, 0, 1);
						}
						
						let mut tmp: Vec<tokenizer::Token> = Vec::new();
						mem::swap(&mut ret, &mut tmp);

						for (index, t) in tmp.into_iter().enumerate() {
							if let tokenizer::Token::Str(s) = t {
								ret.push(
									tokenizer::Token::Str(
										fs::read_to_string(s.as_str())
											.expect("failed to read file")));

								continue
							}

							make_err!(argt, i, t, index+1);
						}

						return ret;
					}

					"file-write" => {
						let vl: usize = ret.len();
						if vl != 2 {
							make_err!(argcn, i, vl, 2);
						}

						let fc: tokenizer::Token = ret.pop().unwrap();
						let fl: tokenizer::Token = ret.pop().unwrap();

						if let tokenizer::Token::Str(fls) = fl {
							if let tokenizer::Token::Str(fcs) = fc {
								fs::write(fls.as_str(), fcs.as_str())
									.expect("failed to write file");

								return vec![tokenizer::Token::Digit(1)];
							}
							
							make_err!(argt, i, fc, 2);	
						}
					
						make_err!(argt, i, fl, 1);	
					}
					
					"int?" => {
						let vl: usize = ret.len();
						if vl != 1 {
							make_err!(argcn, i, vl, 1);
						}

						if let tokenizer::Token::Digit(_) = ret.pop().unwrap() {
							return vec![tokenizer::Token::Digit(1)];
						}
						
						return vec![tokenizer::Token::Digit(0)];
					}

					"str?" => {
						let vl: usize = ret.len();
						if vl != 1 {
							make_err!(argcn, i, vl, 1);
						}

						if let tokenizer::Token::Str(_) = ret.pop().unwrap() {
							return vec![tokenizer::Token::Digit(1)];
						}
						
						return vec![tokenizer::Token::Digit(0)];
					}

					"ident?" => {
						let vl: usize = ret.len();
						if vl != 1 {
							make_err!(argcn, i, vl, 1);
						}

						if let tokenizer::Token::Ident(_) = ret.pop().unwrap() {
							return vec![tokenizer::Token::Digit(1)];
						}
						
						return vec![tokenizer::Token::Digit(0)];
					}

					"ident-name" => {
						let vl: usize = ret.len();
						if vl != 1 {
							make_err!(argcn, i, vl, 1);
						}

						let tok: tokenizer::Token = ret.pop().unwrap();

						if let tokenizer::Token::Ident(s) = tok {
							return vec![tokenizer::Token::Str(s)];
						} else {
							make_err!(argt, i, tok, 1);
						}
					}

					"ident-addr" => {
						let vl: usize = ret.len();
						if vl != 1 {
							make_err!(argcn, i, vl, 1);
						}

						let tok: tokenizer::Token = ret.pop().unwrap();
						
						if let tokenizer::Token::Ident(s) = tok {
							return vec![tokenizer::Token::Str(
								if let Some(val) = funcs.get(&s) {
									format!("{:p}", val.as_ptr())
								} else if let Some(val) = vars.get(&s) {
									format!("{:p}", val.as_ptr())
								} else {
									String::from("0x0")
								}
							)];
						} else {
							make_err!(argt, i, tok, 1);
						}
					}

					"bytes" => {
						let vl: usize = ret.len();
						if vl != 1 {
							make_err!(argcn, i, vl, 1);
						}

						let tok: tokenizer::Token = ret.pop().unwrap();
						if let tokenizer::Token::Str(s) = tok {
							for i in s.bytes() {
								ret.push(tokenizer::Token::Digit(i as u128));
							}
						} else {
							make_err!(argt, i, tok, 1);
						}

						return ret;
					}

					"chars" => {
						let vl: usize = ret.len();
						if vl != 1 {
							make_err!(argcn, i, vl, 1);
						}

						let tok: tokenizer::Token = ret.pop().unwrap();
						if let tokenizer::Token::Str(s) = tok {
							for i in s.chars() {
								ret.push(tokenizer::Token::Str(String::from(i)));
							}

							return ret;
						}
						
						make_err!(argt, i, tok, 1);
					}

					"str-collect" => {
						let vl: usize = ret.len();
						if vl == 0 {
							make_err!(argcf, i, vl, 1);
						}

						let mut s: String = String::new();
						for (index, t) in ret.into_iter().enumerate() {
							if let tokenizer::Token::Str(ref ts) = t {
								s += ts;
								continue
							}

							make_err!(argt, i, t, index+1);
						}

						return vec![tokenizer::Token::Str(s)];
					}
					
					"mut?" => {
						let vl: usize = ret.len();
						if vl != 1 {
							make_err!(argcn, i, vl, 1);
						}

						let tok: tokenizer::Token = ret.pop().unwrap();

						if let tokenizer::Token::Ident(i) = tok {
							if let Some(_) = vars.get(&i) {
								return vec![tokenizer::Token::Digit(1)];
							} else {
								return vec![tokenizer::Token::Digit(0)];
							}
						}

						make_err!(argt, i, tok, 1);
					}

					"def?" => {
						let vl: usize = ret.len();
						if vl != 1 {
							make_err!(argcn, i, vl, 1);
						}

						let tok = ret.pop().unwrap();
						if let tokenizer::Token::Ident(i) = tok {
							return
								vec![tokenizer::Token::Digit(funcs.contains_key(&i) as u128)];
						}
						
						make_err!(argt, i, tok, 1);						
					}

					"undef?" => {
						let vl: usize = ret.len();
						if vl != 1 {
							make_err!(argcn, i, vl, 1);
						}

						let tok = ret.pop().unwrap();
						if let tokenizer::Token::Ident(i) = tok {
							return
								vec![tokenizer::Token::Digit(!funcs.contains_key(&i) as u128)];
						}
						
						make_err!(argt, i, tok, 1);						
					}

					
					"int" => {
						if ret.len() == 0 {
							make_err!(argcf, i, 0, 1);
						}

						let mut tmp: Vec<tokenizer::Token> = Vec::new();
						mem::swap(&mut tmp, &mut ret);

						for (index, t) in tmp.into_iter().enumerate() {
							match t {
								tokenizer::Token::Digit(_) => {
									ret.push(t);
								}

								tokenizer::Token::Str(s) => {
									ret.push(
										tokenizer::Token::Digit(
											s.parse::<u128>()
												.expect(
													format!("ERR: {}: not valid integer for `int`", s)
														.as_str())));
								}

								_ => {
									make_err!(argt, i, t, index+1);
								}
							}
						}

						return ret;
					}

					"str" => {
						if ret.len() == 0 {
							make_err!(argcf, i, 0, 1);
						}

						let mut tmp: Vec<tokenizer::Token> = Vec::new();
						mem::swap(&mut tmp, &mut ret);

						for (index, t) in tmp.into_iter().enumerate() {
							match t {
								tokenizer::Token::Digit(d) => {
									ret.push(tokenizer::Token::Str(format!("{}", d)));
								}

								tokenizer::Token::Str(_) => {
									ret.push(t);
								}

								_ => {
									make_err!(argt, i, t, index+1);
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
							make_err!(argcf, i, vl ,2);
						}

						let tok: tokenizer::Token = ret.remove(0);

						if let tokenizer::Token::Digit(d) = tok {
							let i: usize = d as usize;
							if i >= vl-1 {
								eprintln!("ERR: `{} {}`: index out of bounds, max {}",
													i, d, vl-1);

								return vec![];
							}

							return vec![ret[i].clone()];
						}

						make_err!(argt, i, tok, 1);
					}

					"rm" => {
						let vl: usize = ret.len();
						if vl < 2 {
							make_err!(argcf, i, vl, 2);
						}

						let tok: tokenizer::Token = ret.remove(0);
						if let tokenizer::Token::Digit(d) = tok {
							let i: usize = d as usize;
							if i >= vl-1 {
								eprintln!("ERR: `{} {}`: index out of bounds, max {} in case",
													i, d, vl-1);

								return vec![];
							}

							ret.remove(i);
							return ret;
						}

						make_err!(argt, i, tok, 1);
					}
					
					"=" => {
						let vl: usize = ret.len();
						if vl == 0 {
							make_err!(argcf, i, 0, 1);
						}
						
						let tok: tokenizer::Token = ret.remove(0);
						match tok {
							tokenizer::Token::Str(_)|tokenizer::Token::Digit(_) => {
								// VOID //;
							}

							_ => {
								make_err!(argt, i, tok, 1);
							}
						}
						
						for (index, t) in ret.into_iter().enumerate() {
							match t {
								tokenizer::Token::Str(_)|tokenizer::Token::Digit(_) => {
									// VOID //;
								}

								_ => {
									make_err!(argt, i, t, index+1);
								}
							}
							
							if t != tok {
								return vec![tokenizer::Token::Digit(0)];
							}
						}

						return vec![tokenizer::Token::Digit(1)];
					}

					"pr" => {
						let vl: usize = ret.len();
						let mut st: String = String::new();

						for (index, t) in ret.into_iter().enumerate() {
							if index > 0 && index < vl {
								st.push(' ');
							}

							match t {
								tokenizer::Token::Str(s) => {
									st += &s;
								}

								tokenizer::Token::Digit(d) => {
									st += &format!("{d}");
								}

								_ => {
									make_err!(argt, i, t, index+1);
								}
							}
						}

						print!("{}", st);
						return vec![];
					}

					">" => {
						let vl: usize = ret.len();
						
						if vl != 2 {
							make_err!(argcn, i, vl, 2);
						}
						
						let tok2: tokenizer::Token = ret.pop().unwrap();
						let tok: tokenizer::Token = ret.pop().unwrap();
						if let tokenizer::Token::Digit(d) = tok {
							if let tokenizer::Token::Digit(d2) = tok2 {
								return vec![tokenizer::Token::Digit((d > d2) as u128)];
							} else {
								make_err!(argt, i, tok2, 2);
							}
						} else {
							make_err!(argt, i, tok, 1);
						}
					}
	
					"<" => {
						let vl: usize = ret.len();
						
						if vl != 2 {
							make_err!(argcn, i, vl, 2);
						}
						
						let tok2: tokenizer::Token = ret.pop().unwrap();
						let tok: tokenizer::Token = ret.pop().unwrap();
						if let tokenizer::Token::Digit(d) = tok {
							if let tokenizer::Token::Digit(d2) = tok2 {
								return vec![tokenizer::Token::Digit((d < d2) as u128)];
							} else {
								make_err!(argt, i, tok2, 2);
							}
						} else {
							make_err!(argt, i, tok, 1);
						}
					}

					"+" => {
						if ret.len() == 0 {
							make_err!(argcf, i, 0, 1);
						}
						
						let tok: tokenizer::Token = ret.remove(0);
						let mut r: u128;
						if let tokenizer::Token::Digit(d) = tok {
							 r = d;
						} else {
							make_err!(argt, i, tok, 1);
						}

						for (index, t) in ret.into_iter().enumerate() {
							if let tokenizer::Token::Digit(d) = t {
								 r += d;
								 continue
							}
							
							make_err!(argt, i, t, index+1);
						}

						return vec![tokenizer::Token::Digit(r)];
					}

					"-" => {
						if ret.len() == 0 {
							make_err!(argcf, i, 0, 1);
						}
						
						let tok: tokenizer::Token = ret.remove(0);
						let mut r: u128;
						if let tokenizer::Token::Digit(d) = tok {
							 r = d;
						} else {
							make_err!(argt, i, tok, 1);
						}

						for (index, t) in ret.into_iter().enumerate() {
							if let tokenizer::Token::Digit(d) = t {
								if d > r {
									eprintln!("ERR: {r} - {d}: substract with overflow");
									process::exit(69);
								}
								
								r -= d;
								continue
							}
							
							make_err!(argt, i, t, index+1);
						}

						return vec![tokenizer::Token::Digit(r)];
					}

					"*" => {
						if ret.len() == 0 {
							make_err!(argcf, i, 0, 1);
						}
						
						let tok: tokenizer::Token = ret.remove(0);
						let mut r: u128;
						if let tokenizer::Token::Digit(d) = tok {
							 r = d;
						} else {
							make_err!(argt, i, tok, 1);
						}

						for (index, t) in ret.into_iter().enumerate() {
							if let tokenizer::Token::Digit(d) = t {
								 r *= d;
								 continue
							}
							
							make_err!(argt, i, t, index+1);
						}

						return vec![tokenizer::Token::Digit(r)];
					}

					"/" => {
						if ret.len() == 0 {
							make_err!(argcf, i, 0, 1);
						}
						
						let tok: tokenizer::Token = ret.remove(0);
						let mut r: u128;
						if let tokenizer::Token::Digit(d) = tok {
							 r = d;
						} else {
							make_err!(argt, i, tok, 1);
						}

						for (index, t) in ret.into_iter().enumerate() {
							if let tokenizer::Token::Digit(d) = t {
								if d == 0 {
									make_err!(zerodiv, i);
								}
								
								r /= d;
								continue
							}
							
							make_err!(argt, i, t, index+1);
						}

						return vec![tokenizer::Token::Digit(r)];
					}

					"&"|"bit-and" => {
						if ret.len() == 0 {
							make_err!(argcf, i, 0, 1);
						}
						
						let tok: tokenizer::Token = ret.remove(0);
						let mut r: u128;
						if let tokenizer::Token::Digit(d) = tok {
							 r = d;
						} else {
							make_err!(argt, i, tok, 1);
						}

						for (index, t) in ret.into_iter().enumerate() {
							if let tokenizer::Token::Digit(d) = t {
								 r &= d;
								 continue
							}
							
							make_err!(argt, i, t, index+1);
						}

						return vec![tokenizer::Token::Digit(r)];
					}
					
					"|"|"bit-or" => {
						if ret.len() == 0 {
							make_err!(argcf, i, 0, 1);
						}
						
						let tok: tokenizer::Token = ret.remove(0);
						let mut r: u128;
						if let tokenizer::Token::Digit(d) = tok {
							 r = d;
						} else {
							make_err!(argt, i, tok, 1);
						}

						for (index, t) in ret.into_iter().enumerate() {
							if let tokenizer::Token::Digit(d) = t {
								 r |= d;
								 continue
							}
							
							make_err!(argt, i, t, index+1);
						}

						return vec![tokenizer::Token::Digit(r)];
					}
					
					"^"|"bit-xor" => {
						if ret.len() == 0 {
							make_err!(argcf, i, 0, 1);
						}
						
						let tok: tokenizer::Token = ret.remove(0);
						let mut r: u128;
						if let tokenizer::Token::Digit(d) = tok {
							 r = d;
						} else {
							make_err!(argt, i, tok, 1);
						}

						for (index, t) in ret.into_iter().enumerate() {
							if let tokenizer::Token::Digit(d) = t {
								r ^= d;
								continue
							}
							
							make_err!(argt, i, t, index+1);
						}

						return vec![tokenizer::Token::Digit(r)];
					}
					
					">>"|"bit-rshift" => {
						if ret.len() == 0 {
							make_err!(argcf, i, 0, 1);
						}
						
						let tok: tokenizer::Token = ret.remove(0);
						let mut r: u128;
						if let tokenizer::Token::Digit(d) = tok {
							 r = d;
						} else {
							make_err!(argt, i, tok, 1);
						}

						for (index, t) in ret.into_iter().enumerate() {
							if let tokenizer::Token::Digit(d) = t {
								 r >>= d;
								 continue
							}
							
							make_err!(argt, i, t, index+1);
						}

						return vec![tokenizer::Token::Digit(r)];
					}
					
					"<<"|"bit-lshift" => {
						if ret.len() == 0 {
							make_err!(argcf, i, 0, 1);
						}
						
						let tok: tokenizer::Token = ret.remove(0);
						let mut r: u128;
						if let tokenizer::Token::Digit(d) = tok {
							r = d;
						} else {
							make_err!(argt, i, tok, 1);
						}

						for (index, t) in ret.into_iter().enumerate() {
							if let tokenizer::Token::Digit(d) = t {
								r <<= d;
								continue
							}
							
							make_err!(argt, i, t, index+1);
						}

						return vec![tokenizer::Token::Digit(r)];
					}

					"~"|"bit-not" => {
						let vl: usize = ret.len();
						if vl != 1 {
							make_err!(argcn, i, vl, 1);
						}

						let mut tmp: Vec<tokenizer::Token> = Vec::new();
						mem::swap(&mut ret, &mut tmp);
						
						for (index, t) in tmp.into_iter().enumerate() {
							if let tokenizer::Token::Digit(d) = t {
								ret.push(tokenizer::Token::Digit(!d));
								continue;
							}

							make_err!(argt, i, t, index+1);
						}

						return ret;
					}
								
					"range"|".." => {
						let vl: usize = ret.len();
						if vl == 0 || vl > 3 {
							make_err!(argc, i, vl, 1, 3);
						}
						
						let token: tokenizer::Token = ret.remove(0);
						let mut start: u128 = 0;
						let end: u128;
						let mut step: u128 = 1;

						if let tokenizer::Token::Digit(d0) = token {
							if vl > 1 {
								start = d0;
								if vl > 2 {
									let token3: tokenizer::Token = ret.pop().unwrap();

									if let tokenizer::Token::Digit(d2) = token3 {
										step = d2;
									} else {
										make_err!(argt, i, token3, 3);
									}
								}
									
								let token2: tokenizer::Token = ret.pop().unwrap();

								if let tokenizer::Token::Digit(d1) = token2 {
									end = d1;
								}	else {
									make_err!(argt, i, token2, 2);
								}
							} else {
								end = d0;
							}
								
							ret.clear();	
							for i in (start..end).step_by(step as usize) {
								ret.push(tokenizer::Token::Digit(i));
							}

							return ret;
						}

						make_err!(argt, i, token, 1);
					}

					"load" => {
						let vl: usize = ret.len();
						if vl == 0 {
							make_err!(argcf, i, vl, 1);
						}
						
						let tok: tokenizer::Token = ret.remove(0);
						if let tokenizer::Token::Str(s) = tok {
							return run_file(s.as_str(), 0, &ret, funcs, lambdas, vars);
						}

						make_err!(argt, i, tok, 1);
					}

					"include" => {
						if ret.len() == 0 {
							make_err!(argcf, i, 0, 1);
						}

						for (index, t) in ret.into_iter().enumerate() {
							match t {
								tokenizer::Token::Str(s)|tokenizer::Token::Ident(s) => {
									run_include(s.as_str(), funcs, lambdas, vars);	
								}

								_ => {
									make_err!(argt, i, t, index+1);
								}
							}
						}

						return vec![];
					}

					"eval" => {
						if ret.len() == 0 {
							make_err!(argcf, i, 0, 1);
						}
						
						let token: tokenizer::Token = ret.remove(0);

						match token {
							tokenizer::Token::Str(s) => {
								return run_str(s.as_str(), 0, &ret, funcs, lambdas, vars);
							}

							_ => {
								make_err!(argt, i, token, 1);
							}
						}
					}

					"%%"|"args" => {
						return args.clone();
					}

					"!"|"deref" => {
						let vl: usize = ret.len();
						if vl != 1 {
							make_err!(argcn, i, vl, 1);
						}

						let tok: tokenizer::Token = ret.pop().unwrap();
						if let tokenizer::Token::Ident(s) = tok {
							if !vars.contains_key(&s) {
								make_err!(unknown_ident, s);
							}

							return vars[&s].clone();
						} else {
							make_err!(argt, i, tok, 1);
						}
					}

					"%"|"arg" => {
						if ret.len() == 0 {
							make_err!(argcf, i, 0, 1);
						}

						let mut tmp: Vec<tokenizer::Token> = Vec::new();
						mem::swap(&mut ret, &mut tmp);
						let argl = args.len();						

						for (index, t) in tmp.into_iter().enumerate() {
							if let tokenizer::Token::Digit(d) = t {
								if d as usize >= argl {
									make_err!(indexerr, i, d, argl);
								}

								ret.push(args[d as usize].clone());
								continue
							}

							make_err!(argt, i, t, index+1);
						}

						return ret;
					}

					_ => {
						if !funcs.contains_key(&i) {
							make_err!(unknown_ident, i);
						}

						return run_tokens(&funcs.get(&i).unwrap().clone(), depth+1, 
															&ret, funcs, lambdas, vars);
					}
				}
			}

			tokenizer::Token::Str(_)|tokenizer::Token::Digit(_) => {
				ret.insert(0, fun);
				return ret;
			}
			
			_ => {
				make_err!(notident, fun);
			}
		}
	}

	return ret;
}

pub fn run_str(s: &str,
							 depth: usize,
							 args: &Vec<tokenizer::Token>,
							 funcs: &mut HashMap<String, Vec<tokenizer::Token>>,
							 lambdas: &mut usize,
							 vars: &mut HashMap<String, Vec<tokenizer::Token>>
) -> Vec<tokenizer::Token> {
	let tokens: Vec<tokenizer::Token> = tokenizer::tokenize(s);
	return run_tokens(&tokens, depth, args, funcs, lambdas, vars);
}

pub fn run_file(f: &str,
								depth: usize,
								args: &Vec<tokenizer::Token>,
								funcs: &mut HashMap<String, Vec<tokenizer::Token>>,
								lambdas: &mut usize,
								vars: &mut HashMap<String, Vec<tokenizer::Token>>
) -> Vec<tokenizer::Token> {
	return run_str(
		fs::read_to_string(f)
			.expect("failed to open file")
			.as_str(), depth, args, funcs, lambdas, vars);
}

pub fn run_include(f: &str,
									 funcs: &mut HashMap<String, Vec<tokenizer::Token>>,
									 lambdas: &mut usize,
									 vars: &mut HashMap<String, Vec<tokenizer::Token>>) -> () {
	run_file(("/usr/include/jll/".to_owned()+f+".jll").as_str(), 0, &vec![],
					 funcs, lambdas, vars);
}

pub fn run_file_init(f: &str) -> Vec<tokenizer::Token> {
	let mut funcs: HashMap<String, Vec<tokenizer::Token>> = HashMap::new();
	let mut vars: HashMap<String, Vec<tokenizer::Token>> = HashMap::new();
	let mut lambdas: usize = 0;
	return run_file(f, 0, &vec![], &mut funcs, &mut lambdas, &mut vars);
}
