#![feature(pattern, iter_intersperse, let_chains, file_create_new)]
#![allow(dead_code, unused, non_snake_case)]

use std::{
	collections::HashMap,
	env::{self, Args},
	fmt::Debug,
	io,
	path::Path,
	process::{Command, Stdio},
	time::Duration,
};

use chain::{loopperm, restruct_keytocjchar_and_write};
use itertools::Itertools;
use repeat::word_withspecifiers;
use spell::{swap_table_quickcheck, SwapDictChars};
use util::command_exe;

mod chain;
mod kt;
mod mcr;
mod out;
mod parser;
mod py;
mod repeat;
mod spell;
mod sta;
mod util;
mod yubi; //execute translate

fn readfile(filename: &str) {}

pub type SpellCharsMap = HashMap<String, String>;

fn main() {
	// let p = '略'.to_pinyin().unwrap().with_tone_num_end();
	// println!("{} ", p);

	// out::main();

	// sta::count_chain_main("table/cj5-20000.txt", true);

	// repeat::log_swap_table_permutation(
	// 	"shuang/xiaoque.txt",
	// 	"spell/swap_predefined.txt",
	// 	".auto/cj20000z",
	// 	["table/cj5-20000.txt"],
	// 	"qwertyuiopsdfghjklzxcbmnv",
	// 	Duration::from_secs(60 * 3),
	// 	300,
	// )

	let mut args = env::args();
	args.next(); // "rust program itself"
	match args.next() {
		None => println!("no subcommand was given"),
		Some(command) => match command.as_str() {
			"out" => out(&mut args),
			"rg" => rg(&mut args),
			// "random" => random_swap_perm(&mut args),
			// "perm" => perm_restruct_cli(&mut args),
			"l" => loop_by(&mut args),
			"w" => write_from(&mut args),

			other => {
				println!("Unknown command: {other}");
			}
		},
	}
}

fn loop_by(args: &mut Args) {
	if let Some(a) = args.next() {
		match a.as_str() {
			"level" => random_leveled_perm(args),
			other => {
				println!("Unknown command: {other}");
			}
		}
	}
}

fn write_from(args: &mut Args) {
	let a = args.next();
	let s: String = args.into_iter().collect();
	if let Some(a) = a {
		match a.as_str() {
			"level" => {
				if let Err(e) = restruct_keytocjchar_and_write(&s) {
					println!("{e}");
				}
			}
			m => {
				println!("unknown : {}", m);
			}
		}

	} else {
		println!("need next command");
	}
}

fn random_leveled_perm(args: &mut Args) {
	let i = args.next().unwrap();
	let j = args.next().unwrap();
	let msg = "could not parse as usize";
	let i = usize::from_str_radix(&i, 10).expect(msg);
	let j = usize::from_str_radix(&j, 10).expect(msg);
	let sources = args.collect_vec();
	if sources.len() > 0 {

		loopperm(i, j, sources);
	} else {
		println!("no source path was given");
	}
}

fn out(args: &mut Args) {
	let swap_path = args.next().unwrap();
	let save_path = args.next().unwrap_or(format!(".table/{}", swap_path));
	out::main(&save_path, &swap_path)
}

fn rg(args: &mut Args) {
	let target_path = args.next().unwrap_or(".auto/cj20000random".to_string());

	let mut rg = Command::new("rg")
		.arg("^[0-9]")
		.arg(&target_path)
		.stdout(Stdio::piped())
		.spawn()
		.unwrap();

	let mut sort = Command::new("sort")
		.arg("-n")
		.stdin(rg.stdout.unwrap())
		.stdout(Stdio::piped())
		.spawn()
		.unwrap();

	let mut head = Command::new("head")
		.arg("-n 10")
		.stdin(sort.stdout.unwrap())
		.stdout(Stdio::piped())
		.spawn()
		.unwrap();

	let output = head.wait_with_output().unwrap();
	let output = String::from_utf8(output.stdout).unwrap();

	// let output = command_exe("rg", &["^[0-9]", &target_path, "|sort -n", "|head -n 10"]).unwrap_or("".to_string());

	swap_table_quickcheck(output);
}

#[test]
fn perm_restruct() {
	let perm = "cmntkodxhlfviwruzgyejs".chars().collect_vec();
	let swap = SwapDictChars::new("qwertyuiopasdfghjklzxcbmnv", "spell/swap_base", "z", "a");

	swap.write_perm_dict(
		&perm,
		"shuang/xiaoque.txt",
		["table/cj5-20000.txt"],
		".auto/cj20000random-cmn",
	);

	//OKOKOK saved score matches output file!
}

fn perm_restruct_cli(args: &mut Args) {
	// let perm = "cmntkodxhlfviwruzgyejs".chars().collect_vec();
	let permarg = args.next().unwrap();
	let perm = permarg.chars().collect_vec();
	let swap = SwapDictChars::new("qwertyuiopasdfghjklzxcbmnv", "spell/swap_base", "z", "a");

	let save_path = format!(".auto/cj20000perm/{}", permarg);

	swap.write_perm_dict(
		&perm,
		"shuang/xiaoque.txt",
		["table/cj5-20000.txt"],
		// ".auto/cj20000random-cmn",
		&save_path,
	);

	//OKOKOK saved score matches output file!
}

// fn cj_perm_write_dict<P,I>(
// 	perm: &'static str,
// 	table_paths: I,
// )
// where
// 	P: AsRef<Path>,
// 	I: IntoIterator<Item = P>,
// {
// 	let perm = perm.chars().collect_vec();
// 	let swap = SwapDictChars::new("qwertyuiopasdfghjklzxcbmnv", "spell/swap_base", "z", "a");

// 	let paths = table_paths.into_iter().collect::<Vec<_>>();
// 	swap.write_perm_dict(&perm,
// 		"shuang/xiaoque.txt",
// 		&paths,
// 		".auto/cj20000random-cmn", );

// 	//OKOKOK saved score matches output file!
// }

fn get_specials(args: &mut Args) -> Vec<(String, String)> {
	args
		.map(|a| {
			let mut s = a.split([',', '-', '.']);
			let special = s.next().expect("alphabet pair separetedby [-,.]");
			let positions = s.next().expect("alphabet pair separetedby [-,.]");
			(special.to_string(), positions.to_string())
		})
		.collect()
}

fn random_swap_perm(args: &mut Args) {
	let p = "spell/swap_base";
	let swap_path = if let Some(s) = args.next() {
		s
	} else {
		println!("using default swap_path: {p}");
		p.to_string()
	};

	let s = SwapDictChars::new("qwertyuiopasdfghjklzxcbmnv", &swap_path, "z", "a");


	// let specials = [("mhboin", "sdfjkl")];
	let spec = get_specials(args);

	random_perm_specialize_name(
		".auto/cj20000perm.random",
		"shuang/xiaoque.txt",
		["table/cj5-20000.txt"],
		3000,
		20,
		s,
		spec.as_slice(),
	);
}

fn random_perm_specialize_name<P, I>(
	save_path_base: &str,
	shuangpin_table: P,
	table_paths: I,
	nonchain_goal: usize,
	process_perm_chunk: usize,
	sdc: SwapDictChars,
	special_slots: &[(String, String)],
) where
	I: IntoIterator<Item = P>,
	P: AsRef<Path> + Debug + Clone + Copy,
{
	let save_path = special_name(save_path_base, &sdc, special_slots);

	sdc.perm(
		shuangpin_table,
		table_paths,
		save_path.as_str(),
		nonchain_goal,
		process_perm_chunk,
		special_slots,
	)
}

fn special_name<S: ToString>(base: &str, sdc: &SwapDictChars, specials: &[(S, S)]) -> String {
	let mut predefined = sdc.predefined.iter().collect_vec();
	predefined.sort();
	let prede: String = predefined
		.into_iter()
		.map(|(s, d)| format!("-{s}{d}"))
		.collect();

	let mut sorted = specials
		.into_iter()
		.map(|(s, d)| {
			let s = s.to_string();
			let d = d.to_string();
			let mut sc: Vec<char> = s.chars().collect();
			let mut dc: Vec<char> = d.chars().collect();
			sc.sort();
			dc.sort();
			let s: String = sc.into_iter().collect();
			let d: String = dc.into_iter().collect();
			(s, d)
		})
		.collect::<Vec<_>>();
	sorted.sort();

	let s: String = sorted
		.into_iter()
		.map(|(cjs, keys)| format!("-{}_{}", cjs, keys))
		.collect();
	base.to_string() + &prede + &s
}

#[test]
fn lines() {
	let s = "aaaa\nhwllo\n".lines();
	dbg!(s.last()); // hwllo
}
