// pub fn is_lianda_fluent(cs: char, cd: char,)->bool {

// }

use std::{fs::File, path::Path};

use itertools::Itertools;

use crate::{kt::read_lines, parser::StringStringsEntry};

pub fn fluent_list<P: AsRef<Path>>(path: P) -> Vec<String> {
	let lines = read_lines(path).expect("cant open file");
	lines
		.into_iter()
		.filter_map(|f| f.ok())
		.filter_map(|f| {
			let mut cs = f.chars();
			let e = cs.next()?;
			let f = cs.next()?;
			Some(e.to_string() + &f.to_string())
		})
		.collect()
}

pub fn is_left_key(key: char) -> bool {
	let d = "qwertasdfgzxcb";
	let k = "yuiophjlknmv";
	d.contains(key)
}

pub fn not_fluent_pairs() -> Vec<(String, String)> {
	let d = "qwertasdfgzxcb";
	let k = "yuiophjlknmv";
	let lianda_difficult = [("qtagb", d), ("yph", k)];
	lianda_difficult
		.into_iter()
		.map(|(s, d)| (s.to_owned(), d.to_owned()))
		.collect()
}
