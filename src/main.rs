#![feature(pattern, iter_intersperse, let_chains, file_create_new)]
#![allow(dead_code, unused, non_snake_case)]

use std::{
	collections::HashMap,
	env::{self, Args},
	path::Path,
	process::{Command, Stdio},
	time::Duration, fmt::Debug,
};

use itertools::Itertools;
use repeat::word_withspecifiers;
use spell::{swap_table_quickcheck, SwapDictChars};
use util::command_exe;

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
	args.next();
	match args.next() {
		None => println!("no subcommand was given"),
		Some(command) => match command.as_str() {
			"random" => random_swap_perm(),
			"rg" => rg(&mut args),
			other => {
				println!("Unknown command: {other}");
			}
		},
	}
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

fn random_swap_perm() {
	let s = SwapDictChars::new("qwertyuiopasdfghjklzxcbmnv", "spell/swap_base", "z", "a");

	let specials = [
		("mhboin", "sdfjkl")
	];


	random_perm_specialize_name(
		".auto/cj20000specify.random",
		"shuang/xiaoque.txt",
		["table/cj5-20000.txt"],
		3000,
		20,
		s,
		&specials
	);
}

fn random_perm_specialize_name<P,I>(
	save_path_base: &str,
	shuangpin_table: P,
	table_paths: I,
	nonchain_goal: usize,
	process_perm_chunk: usize,
	sdc: SwapDictChars,
	special_slots: &[(&str, &str)],
)
where
	I: IntoIterator<Item = P>,
	P: AsRef<Path> + Debug + Clone + Copy,
{
	let save_path = special_name(save_path_base, special_slots);

	sdc.perm(
		shuangpin_table,
		table_paths,
		save_path.as_str(),
		nonchain_goal,
		process_perm_chunk,
		special_slots,
	)
}

fn special_name(base: &str, specials: &[(&str, &str)]) -> String {
	let mut sorted = specials.into_iter().map(|(s,d)| {
		let mut sc: Vec<char> = s.chars().collect();
		let mut dc: Vec<char> = d.chars().collect();
		sc.sort();
		dc.sort();
		let s: String = sc.into_iter().collect();
		let d: String = dc.into_iter().collect();
		(s, d)
	}).collect::<Vec<_>>();
	sorted.sort();
	
	let s: String = sorted.into_iter()
		.map(|(cjs, keys) | format!("-{}_{}", cjs, keys) )
		.collect();
	base.to_string() + &s
}

#[test]
fn lines() {
	let s = "aaaa\nhwllo\n".lines();
	dbg!(s.last()); // hwllo
}
