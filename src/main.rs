#![feature(pattern, iter_intersperse, let_chains, file_create_new)]
#![allow(dead_code, unused, non_snake_case)]

use std::{collections::HashMap, time::Duration};

use spell::SwapDictChars;

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

	random_swap_perm()
}

fn random_swap_perm() {
	let s = SwapDictChars::new("qwertyuiopasdfghjklzxcbmnv", "spell/swap_base",
	"z",
	"a");

	dbg!(s);
	return;
	s.perm(
		"shuang/xiaoque.txt",
		["table/cj5-20000.txt"],
		".auto/cj20000random",
		3000,
		20,
	);
}

#[test]
fn lines() {
	let s = "aaaa\nhwllo\n".lines();
	dbg!(s.last()); // hwllo
}
