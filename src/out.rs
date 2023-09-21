
use itertools::izip;

use crate::{kt::{YongDictWordSpells, to_yong_dict, from_word_spells_dict}, spell::{SpecifySpelling, YongSpelling}, parser::{self, StringStringsDict}, py::get_shuangpin_tone, util::{unions_hashmap, cmp_by_len_default}, d, sta::{double_words_by_len, display_hashmap}};
use std::{io::{BufWriter, LineWriter, Write}, collections::HashMap, path::Path, fs::File};


pub fn main() {
	let mut m: YongDictWordSpells = HashMap::new();
	let merged = unions_hashmap(&mut m
		, [ "table/xxyx.txt"
		, "table/xxyx-ziz.txt"
		, "table/cj5q-90000.txt"
		]
		.map(|path| to_yong_dict(path))
		.into_iter()
		.collect()
	);
	println!("complete merge dicts");
	let dict = merged.clone();
	let mut dict = 
		DictTranslator::CjqYongPy.flip_word_spells(dict);
	let toadd = DictTranslator::Free.flip_word_spells(
		from_word_spells_dict("spell/add-short.txt")
	);
	let dict = unions_hashmap(&mut dict, vec![
		toadd
	]);
	let mut seed: HashMap<u32, Vec<u32>> = HashMap::new();
	let sta = d!(
		{
			let stas = [
				double_words_by_len(dict.clone(), 1),
				double_words_by_len(dict.clone(), 2),
				double_words_by_len(dict.clone(), 3),
				double_words_by_len(dict.clone(), 4),
				double_words_by_len(dict.clone(), 5),
				double_words_by_len(dict.clone(), 6),
			];
			let mut se: HashMap<u32, Vec<u32>> = HashMap::new();
			for sta in stas.iter() {
				(1..10).into_iter().for_each(|n| {
					if let Some(x) = sta.get(&n) {
						if let Some(v) = se.get_mut(&n) {
							v.push(*x);
						} else {
							se.insert(n, vec![*x]);
						}
					}
				});
			}
			display_hashmap(se)
		}
	);

	write_spell_words_dict("table/cxvpy-swap-min-spellpoly.txt", dict.clone()
	);
}

pub type YongDictSpellWords = HashMap<String, Vec<String>>;

pub fn write_spell_words_dict<P>(path: P, dict: YongDictSpellWords) 
-> std::io::Result<()>
where P: AsRef<Path>
{
	println!("start creating your table!");
	let file = File::create(path)?;
	// let mut file = LineWriter::new(file);
	let mut file = BufWriter::new(file);
	let mut v: Vec<_> = dict.into_iter().collect();
	v.sort_by(|j,k|
			cmp_by_len_default(&j.0, &k.0)
		);
	
	let dict: Vec<(String, Vec<String>)> = v.into_iter().collect();
	dict.iter()
		.for_each(|t| {
			let s = SpellWordsEntry {
				spell: t.0.to_owned(),
				words: t.1.to_owned()
			} .to_string();
			// file.write_all(s.as_bytes());
			file.write(s.as_bytes());
		});

	Ok(())
}


struct SpellWordsEntry {
	spell: String,
	words: Vec<String>
}
impl SpellWordsEntry {
	fn from_tuple(t: (&String, &Vec<String>))-> Self {
		SpellWordsEntry { spell: t.0.to_owned(), words: t.1.to_owned() }
	}
	fn to_tuple(&self)-> (String, Vec<String>) {
		(self.spell.clone(), self.words.clone())
	}
	fn to_string(&self)-> String {
		let word_clause = self.words.clone().into_iter().intersperse_with(|| " ".to_owned())
		.collect::<String>()
			+ if self.words.len() == 1 {"$SPACE"} else {""}
		;
		let s = format!("{} {}\n", self.spell, word_clause);
		s
	}
}

pub enum DictTranslator {
	CjqYongPy,
	Free
}
impl DictTranslator {
	fn flip_word_spells(&self, dict: YongDictWordSpells) -> YongDictSpellWords {
		let translater = match self {
			DictTranslator::CjqYongPy => WordSpellsEntry::to_cjq_xxyx_py ,
			Self::Free => WordSpellsEntry::just_pass_all_spells 
		};
		let j = dict.iter().filter_map(|t|{
			let spells = translater( WordSpellsEntry::from_tuple(t));
			if spells.len() > 0 {
				Some((t.0.clone(), spells) )
			} else {None}
		}
		);
		println!("src dict ok");
		let mut init: YongDictSpellWords = HashMap::new();
		for (word, spells) in j.into_iter() {
			for spell in spells.iter() {
				if let Some(mv) = init.get_mut(spell) {
					if mv.iter().all(|w| w != &word) {
						//綴の長さに応じて追加しないようにするかも
						mv.push(word.clone());
					}
				} else {
					init.insert(spell.clone(), vec![word.clone()]);
				}
			}
		}
		println!("spellwords dict ready");
		init
	}
	
}


pub struct WordSpellsEntry {
	pub word: String,
	pub spells: Vec<SpecifySpelling>
}

impl WordSpellsEntry {
	fn from_tuple(t: (&String, &Vec<SpecifySpelling>))-> Self {
		WordSpellsEntry {
			word: t.0.to_owned()
			,spells: t.1.to_owned()
		}
	}
	fn get_xxyx_bihuas(&self)-> Vec<String> {
		let xxs = self.spells.iter().filter(|s| s.spelling == YongSpelling::Xxyx);
		let bihuas : Vec<String> = xxs.filter_map(|ss| {
			let s = ss.spell.as_bytes();
			if let Some( (_, rem)) = s.split_first() {
				if rem.len() > 0 {
					Some(String::from_utf8(rem.to_vec()).unwrap() )
				} else { None }
			} else {
				None
			}
		})
			.collect();
		bihuas
	}
	fn just_pass_all_spells(self) -> Vec<String> {
		self.spells.into_iter().map(|ss| ss.spell)
			.collect()
	}
	fn to_cjq_xxyx_py(self) -> Vec<String> { 
		let xxs = self.spells.iter().filter(|s| s.spelling == YongSpelling::Xxyx);
		let cjs = self.spells.iter().filter(|s| s.spelling == YongSpelling::Cangjie);
		let bihuas = self.get_xxyx_bihuas();
		let cj_edge = cjs.filter_map(|ss| {
			let s = ss.spell.as_bytes();
			if let (Some(first), Some(last)) = (s.first(), s.last()) {
				Some((first, last))
			} else {
				None
			}
		}).next()
		;
		if bihuas.len() > 0 {
			let py = get_shuangpin_tone(
				self.word.chars().next().expect("at least one length word")
				, "shuang/xiaoque.txt");
			let cj_fix: String = if let Some((start,end)) = cj_edge {
				let o = String::from_utf8(vec![*end]);
				if let Ok(e) = o
				{
					// match e.as_str() {
					// 	"a" => "s",
					// 	"e" => "y",
					// 	"u" => "z",
					// 	"i" => "x",
					// 	"o" => "w",
					// 	s => s.clone()
					// }
					let pairs = [
						("a", "h"),
						("e", "c"),
							("c", "w"),
						("u", "x"),
						("i", "z"),
						("o", "s"),
						("s", "y")
					];
					let (_, changed) =
						pairs.iter().fold((false, e.clone()), |acc, d| {
							let (has_changed, acc) = acc;
							if !has_changed && acc.as_str() == d.0 {
								(true, d.1.to_string())
							} else {
								(has_changed, acc)
							}
						});
					changed
				} else {
					"q".to_string()
				}
			} else {
				"q".to_string()
			}
			;
			let py_fix: String = if let Some(py) = py {
				py.to_string()
			} else {
				"".to_string()
			};
			
			let mut bim = vec![];
			// bim 
			// 	.sort_by(|s,d| s.len().cmp(&d.len()));
			let toadd: Vec<String> = bihuas
				.iter()
				.flat_map(|xxyx_fix| 
					spell_polymer(0, &cj_fix, xxyx_fix, &py_fix)
				)
				.collect();
			bim.extend(toadd);
			// if xxs.into_iter().any(|xs| xs.spell.len() == 1)  {
			// 	if let Some(c) = py_fix.chars().next() {
			// 		let topush = match self.word.as_str() {
			// 			"有" => 'v'
			// 			, _ => c
			// 		};
			// 		bim.push(topush.to_string());
			// 	}
			// }
			// match self.word.as_str() {
			// 	"时" => bim.push("s".to_string()),
			// 	"出" => bim.push("i".to_string()),
			// 	"时" => bim.push("o".to_string()),
			// 	_ => ()
			// }
			
			bim
		} else {
			vec![]
		}
	}
}

fn spell_polymer(id: u32, cj_fix: &str, xxyx_fix: &str, py_fix: &str)-> Vec<String> {
	match id {
		0 => match xxyx_fix.len() {
			1 => vec![cj_fix.to_string() + xxyx_fix]  //2
		, 2 => vec![
			cj_fix.to_string() + xxyx_fix             //3
			, cj_fix.to_string() + xxyx_fix + &py_fix //6
			]
		, 3 => vec![                       
			cj_fix.to_string() + xxyx_fix,            //4
			cj_fix.to_string() + xxyx_fix + &py_fix]  //7
		, _ => vec![
			cj_fix.to_string() + xxyx_fix + &py_fix   //8+
		]
		}
		,
		_ => match xxyx_fix.len() {
			1 => vec![cj_fix.to_string() + xxyx_fix]  //2
		, 2 => vec![
			cj_fix.to_string() + xxyx_fix//3
			, cj_fix.to_string() + xxyx_fix + &py_fix]//6
		, _ => vec![                       //if 3
			cj_fix.to_string() + xxyx_fix,            //4
			cj_fix.to_string() + xxyx_fix + &py_fix]  //7
		}
		
	}
}