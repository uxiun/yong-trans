use std::{
	cmp::Ordering,
	collections::{HashMap, HashSet},
	path::Path,
};

use crate::{
	kt::{get_yong_defs, LinesBufYong, YongDictWordSpells},
	out::YongDictSpellWords,
	parser::StringStringsEntry,
};

pub fn count_chain_main<P: AsRef<Path>>(path: P, sort: bool) {
	show_count_chain(count_chain(1, path), sort);
}

pub type ByLenSta = HashMap<u32, u32>;
pub fn double_words_by_len(dict: YongDictSpellWords, count_level: u32) -> ByLenSta {
	let mut sta: ByLenSta = HashMap::new();
	dict.into_iter().for_each(|(k, v)| {
		let a = v.len() as u32;
		let k = k.len() as u32;
		if let Some(doubles) = sta.get(&k) {
			sta.insert(k, doubles + if a >= count_level { 1 } else { 0 });
		} else {
			sta.insert(k, if a > count_level { 1 } else { 0 });
		}
	});
	sta
}

pub fn double_words_by_len_vec(dict: YongDictSpellWords, count_level: u32) -> Vec<u32> {
	double_words_by_len(dict, count_level)
		.values()
		.map(|d| *d)
		.collect()
}

// pub fn double_words_by_prefix(dict: YongDictSpellWords, prefix: &str)
// -> (&str, u32)
// {

// }

pub fn display_hashmap<K, V>(h: HashMap<K, V>) -> Vec<(K, V)>
where
	K: Ord + Clone,
	V: Ord + Clone,
{
	let mut ve: Vec<_> = h.iter().map(|(k, v)| (k.clone(), v.clone())).collect();
	ve.sort();
	ve
}

pub fn count_chain<P: AsRef<Path>>(
	chain_start: usize,
	path: P,
) -> HashMap<String, HashMap<String, u32>> {
	let mut h: HashMap<String, HashMap<String, u32>> = HashMap::new();
	let defs = get_yong_defs(path);
	for d in defs {
		let (_, chain) = d.spell.split_at(chain_start);
		let s = chain.to_string();
		let target: String = s.chars().take(2).collect();
		let tar = target.clone();
		if target.len() == 2 {
			let rv = (&target).chars().rev().collect::<String>();

			if let Some(k) = [target.clone(), rv].into_iter().find(|k| h.contains_key(k)) {
				let h = h.get_mut(&k).unwrap();
				h.entry(target).and_modify(|u| *u += 1).or_insert(1);
			} else {
				h.insert(tar.clone(), [(tar, 1)].into());
			}
		}
	}

	h
}

pub fn show_count_chain(count: HashMap<String, HashMap<String, u32>>, sort: bool) {
	// dbg!(&count);

	let mut v = count
		.into_iter()
		.map(|(k, v)| (k, v.clone(), v.iter().map(|(k, v)| v).sum::<u32>()))
		.collect::<Vec<_>>();
	if sort {
		v.sort_by(|s, d| {
			if s.2 < d.2 {
				Ordering::Greater
			} else if s.2 > d.2 {
				Ordering::Less
			} else {
				Ordering::Equal
			}
		});
	}
	for (cs, h, sum) in v.into_iter() {
		let st: String = h.into_iter().fold(String::new(), |s, (chain, n)| {
			s + " " + &chain + " " + &n.to_string()
		});

		println!("{} = {}", st, sum);
	}
}
