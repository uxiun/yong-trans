use std::collections::HashMap;

use crate::{out::YongDictSpellWords, parser::StringStringsEntry};

pub type ByLenSta = HashMap<u32, u32>;
pub fn double_words_by_len(dict: YongDictSpellWords, count_level: u32)-> ByLenSta {
	let mut sta: ByLenSta = HashMap::new();
	dict.into_iter().for_each(|(k, v)| {
		let a = v.len() as u32;
		let k = k.len() as u32;
		if let Some(doubles) = sta.get(&k) {
			sta.insert(k, doubles + 
				if a >= count_level {1} else {0}
			);
		} else {
			sta.insert(k, 
				if a > count_level {1} else {0}
			);
		}
	}
	);
	sta
}

pub fn double_words_by_len_vec(dict: YongDictSpellWords, count_level: u32)
-> Vec<u32>
{
	double_words_by_len(dict, count_level).values()
		.map(|d| *d)
		.collect()
}

// pub fn double_words_by_prefix(dict: YongDictSpellWords, prefix: &str)
// -> (&str, u32)
// {
	
// }

pub fn display_hashmap<K,V>(h: HashMap<K,V>)->
Vec<(K,V)>
where K: Ord + Clone,
	V: Ord + Clone,
{
	let mut ve: Vec<_> = h.iter().map(|(k,v)| (k.clone(), v.clone()))
		.collect();
	ve.sort();
	ve
}