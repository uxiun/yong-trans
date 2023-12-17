#![feature(pattern, iter_intersperse, let_chains, file_create_new)]
#![allow(dead_code, unused, non_snake_case)]

use std::{collections::HashMap, time::Duration};

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
	
	repeat::loop_swap_table_permutation(
		"shuang/xiaoque.txt",
		"spell/swap_predefined.txt",
		".auto/cj20000z",
		["table/cj5-20000.txt"],
		"qwertyuiopsdfghjklzxcbmnv",
		Duration::from_secs(60 * 3),
		300,
	)
}
