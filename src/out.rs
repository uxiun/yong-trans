use itertools::Itertools;

use crate::{
	kt::{from_word_spells_dict, swap_key, to_swap_key_dict, to_yong_dict, YongDictWordSpells},
	parser::{self, StringStringsDict, StringStringsEntry},
	py::get_shuangpin_tone,
	spell::{perm_swap_table, perm_swap_table_process, BihuaXxyx, SpecifySpelling, YongSpelling},
	sta::{display_hashmap, double_words_by_len},
	util::{cmp_by_len_default, process_permutation_separately, unions_hashmap, with_elapsed_time},
	yubi::{fluent_list, is_left_key, not_fluent_pairs},
};
use std::{
	collections::{HashMap, HashSet},
	fs::File,
	io::{BufWriter, LineWriter, Write},
	iter::zip,
	path::Path,
	str::pattern::Pattern,
	time::Instant,
	u32,
};

pub fn main<P: AsRef<Path>, Q: AsRef<Path>>(save_path: P, swap_path: Q) {
	let mut m: YongDictWordSpells = HashMap::new();
	let merged = unions_hashmap(
		&mut m,
		[
			// "table/xxyx.txt"
			// , "table/xxyx-ziz.txt"
			"table/cj5-20000.txt",
			// "table/cj5-90000.txt",
		]
		.map(|path| to_yong_dict(path))
		.into_iter()
		.collect(),
	);
	let deduped: HashMap<String, Vec<SpecifySpelling>> = merged
		.clone()
		.into_iter()
		.map(|(k, v)| {
			(k, {
				let h: HashSet<SpecifySpelling> = HashSet::from_iter(v.into_iter());
				h.into_iter().collect()
			})
		})
		.collect();
	println!("complete merge dicts");
	let dict = deduped;

	let mut dict = DictTranslator::Cjmain.flip_word_spells_with_make_word_specifier(
		dict,
		// "spell/swap_refreq.txt",
		swap_path,
		"shuang/xiaoque.txt",
	);

	// let mut dict = DictTranslator::Cjmain.explore_swap_and_flip_word_spells(
	// 	dict,
	// 	"spell/swap_predefined.txt",
	// 	"shuang/xiaoque.txt",
	// 	Err(10),
	// );

	// let toadd = DictTranslator::Free.flip_word_spells(
	// 	from_word_spells_dict("spell/add-short.txt")
	// );
	// let dict = unions_hashmap(&mut dict, vec![
	// 	toadd
	// ]

	// let sta = dbg!({
	// 	let dict: YongDictSpellWords = dict.clone().into_iter().collect();
	// 	let stas = [
	// 		double_words_by_len(dict.clone(), 1),
	// 		double_words_by_len(dict.clone(), 2),
	// 		double_words_by_len(dict.clone(), 3),
	// 		double_words_by_len(dict.clone(), 4),
	// 		double_words_by_len(dict.clone(), 5),
	// 		double_words_by_len(dict.clone(), 6),
	// 	];
	// 	let mut se: HashMap<u32, Vec<u32>> = HashMap::new();
	// 	for sta in stas.iter() {
	// 		(1..10).into_iter().for_each(|n| {
	// 			if let Some(x) = sta.get(&n) {
	// 				if let Some(v) = se.get_mut(&n) {
	// 					v.push(*x);
	// 				} else {
	// 					se.insert(n, vec![*x]);
	// 				}
	// 			}
	// 		});
	// 	}
	// 	display_hashmap(se)
	// });

	if let Err(e) = write_spell_words_dict(
		// "table-custom/cjrefreq-20000.txt",
		save_path,
		dict.clone(),
	) {
		println!("write_spell_words_dict err: {}", e);
	}
}

pub type YongDictSpellWords = HashMap<String, Vec<String>>;

pub fn write_spell_words_dict<P>(path: P, dict: Vec<(String, Vec<String>)>) -> std::io::Result<()>
where
	P: AsRef<Path>,
{
	println!("start creating your table!");

	let file = File::create(path)?;
	// let mut file = LineWriter::new(file);
	let mut file = BufWriter::new(file);
	let mut v: Vec<_> = dict.into_iter().collect();
	v.sort_by(|j, k| cmp_by_len_default(&j.0, &k.0));

	let dict: Vec<(String, Vec<String>)> = v.into_iter().collect();
	dict.iter().for_each(|t| {
		let s = SpellWordsEntry {
			spell: t.0.to_owned(),
			words: t.1.to_owned(),
		}
		.to_string();
		// file.write_all(s.as_bytes());
		file.write(s.as_bytes());
	});

	Ok(())
}

pub fn spell_words_dict_tostring(dict: Vec<(String, Vec<String>)>) -> String {
	let mut v = dict;
	v.sort_by(|j, k| cmp_by_len_default(&j.0, &k.0));

	v.into_iter().map(|t| {
		SpellWordsEntry {
			spell: t.0.to_owned(),
			words: t.1.to_owned(),
		}
		.to_string()
	})
	.join("")
}

struct SpellWordsEntry {
	spell: String,
	words: Vec<String>,
}
impl SpellWordsEntry {
	fn from_tuple(t: (&String, &Vec<String>)) -> Self {
		SpellWordsEntry {
			spell: t.0.to_owned(),
			words: t.1.to_owned(),
		}
	}
	fn to_tuple(&self) -> (String, Vec<String>) {
		(self.spell.clone(), self.words.clone())
	}
	fn to_string(&self) -> String {
		let word_clause = Iterator::intersperse_with(self.words.clone().into_iter(), || " ".to_owned())
			.collect::<String>()
			+ if self.spell.chars().count() < 1 && self.words.len() == 1 {
				"$SPACE"
			} else {
				""
			};
		let s = format!("{} {}\n", self.spell, word_clause);
		s
	}
}

pub enum DictTranslator {
	CjqYongPy,
	CjheadYongcomposedPy,
	// XxchainCjPy,
	Cjmain,
	Free,
}

#[derive(Debug, Clone)]
pub struct WithMakeWordSpecifier {
	pub specified_for_make_word_spells: Vec<String>,
	pub other_spells: Vec<String>,
}

impl DictTranslator {
	fn flip_word_spells_with_make_word_specifier<P: AsRef<Path>, Q: AsRef<Path>>(
		&self,
		dict: YongDictWordSpells,
		swap_dict: P,
		shuangpin_table: Q,
	) -> Vec<(String, Vec<String>)> {
		let translator = match self {
			Self::Cjmain => WordSpellsEntry::to_cjmain_with_specifier,
			_ => WordSpellsEntry::to_cjmain_with_specifier,
		};
		let swapdict = to_swap_key_dict(swap_dict);
		let j = dict.iter().map(|t| {
			let withspecifiers = translator(WordSpellsEntry::from_tuple(t), &swapdict, &shuangpin_table);
			(t.0.to_owned(), withspecifiers)
		});

		let mut init: YongDictSpellWords = HashMap::new();
		let mut wordpsells = Vec::new();
		for (word, withspecifieds) in j.into_iter() {
			let both = [
				withspecifieds.other_spells,
				withspecifieds.specified_for_make_word_spells.clone(),
			]
			.into_iter()
			.flatten();
			for spell in both {
				if let Some(mv) = init.get_mut(&spell) {
					// if mv.iter().all(|w| w != &word) {
					if !mv.contains(&word) {
						//綴の長さに応じて追加しないようにするかも
						mv.push(word.clone());
					}
				} else {
					init.insert(spell.clone(), vec![word.clone()]);
				}
			}
			for spell in withspecifieds.specified_for_make_word_spells {
				wordpsells.push(("^".to_string() + &spell, vec![word.clone()]));
			}
		}
		wordpsells.extend(init.into_iter());

		wordpsells
	}

	fn flip_word_spells_with_make_word_specifier_byswapdict<P: AsRef<Path>, Q: AsRef<Path>>(
		&self,
		dict: YongDictWordSpells,
		swapdict: &HashMap<String, String>,
		shuangpin_table: Q,
	) -> Vec<(String, Vec<String>)> {
		let translator = match self {
			Self::Cjmain => WordSpellsEntry::to_cjmain_with_specifier,
			_ => WordSpellsEntry::to_cjmain_with_specifier,
		};
		let j = dict.iter().map(|t| {
			let withspecifiers = translator(WordSpellsEntry::from_tuple(t), &swapdict, &shuangpin_table);
			(t.0.to_owned(), withspecifiers)
		});

		let mut init: YongDictSpellWords = HashMap::new();
		let mut wordpsells = Vec::new();
		for (word, withspecifieds) in j.into_iter() {
			let both = [
				withspecifieds.other_spells,
				withspecifieds.specified_for_make_word_spells.clone(),
			]
			.into_iter()
			.flatten();
			for spell in both {
				if let Some(mv) = init.get_mut(&spell) {
					// if mv.iter().all(|w| w != &word) {
					if !mv.contains(&word) {
						//綴の長さに応じて追加しないようにするかも
						mv.push(word.clone());
					}
				} else {
					init.insert(spell.clone(), vec![word.clone()]);
				}
			}
			for spell in withspecifieds.specified_for_make_word_spells {
				wordpsells.push(("^".to_string() + &spell, vec![word.clone()]));
			}
		}
		wordpsells.extend(init.into_iter());

		wordpsells
	}

	fn explore_swap_and_flip_word_spells<P: AsRef<Path>>(
		&self,
		dict: YongDictWordSpells,
		swap_dict: P,
		shuangpin_table: P,
		takelen_max: Result<usize, usize>,
	) -> Vec<(String, Vec<String>)> {
		let translator = match self {
			Self::Cjmain => WordSpellsEntry::to_cjmain_with_specifier,
			_ => WordSpellsEntry::to_cjmain_with_specifier,
		};

		let mapf = |swap_dict: HashMap<char, char>| {
			let stringdict: HashMap<String, String> = HashMap::from_iter(
				swap_dict
					.into_iter()
					.map(|(k, v)| (k.to_string(), v.to_string())),
			);
			dict.iter().fold(
				(0, vec![]),
				|(mut count, mut word_withspecifiers), (word, spells)| {
					let withspecifiers = translator(
						WordSpellsEntry {
							word: word.to_string(),
							spells: spells.clone(),
						},
						&stringdict,
						&shuangpin_table,
					);
					count += withspecifiers
						.specified_for_make_word_spells
						.iter()
						.fold(0, |count, s| {
							if let Some(c) = s.chars().nth(3) {
								if ['.', ','].contains(&c) {
									count + 1
								} else {
									count
								}
							} else {
								count
							}
						});
					word_withspecifiers.push((word.to_owned(), withspecifiers));
					(count, word_withspecifiers)
				},
			)
		};

		let sorted = with_elapsed_time("sorted", || {
			// let sort_key = |(nonchain, _)| nonchain;

			// process_permutation_separately(permsetclone, process_perm, sort_key)

			perm_swap_table_process(&swap_dict, takelen_max, |(count, _)| *count, mapf)
		});

		// let perms = with_elapsed_time("get_permutation",
		// || perm_swap_table(&swap_dict, swap_perm_len)
		// );
		// let _sorted = with_elapsed_time("sort permutations", || {

		// 	let mut tosort = perms.iter()
		// 		.map(|swap_dict| {
		// 			dict.iter().fold((0, vec![]), |(mut count, mut word_withspecifiers), (word, spells)| {
		// 				let withspecifiers = translator(WordSpellsEntry { word: word.to_string(), spells: spells.clone() }, &swap_dict, &shuangpin_table);
		// 				count += withspecifiers.specified_for_make_word_spells
		// 					.iter()
		// 					.fold(0, |count, s| if s.contains(['.', ',']) {count + 1} else {count} );
		// 				word_withspecifiers.push((word.to_owned(), withspecifiers));
		// 				(count, word_withspecifiers)
		// 			})

		// 		})
		// 		.collect_vec();
		// 	tosort.sort_by_key(|(nonchain, _)| *nonchain);
		// 	tosort
		// });

		for (i, (notchain, js)) in &sorted[0..5] {
			let allkeys = "qwertyuiopasdfghjklzxcbmnv";
			let h: HashMap<char, char> = HashMap::from_iter(zip(allkeys.chars(), i.to_owned()));

			let leftend = "tgb";
			let y = "y";
			let z = "z";
			let rightend = "plv";

			let mut s = String::new();
			let mut is_next_rightstart = false;
			let mut is_next_leftstart = true;
			for (k, v) in h.into_iter() {
				if rightend.contains(k) {
					s = format!("{s}{k}\n");
					is_next_leftstart = true;
				} else if leftend.contains(k) {
					s = format!("{s}{k} ");
					is_next_rightstart = true;
				} else if is_next_rightstart && !y.contains(k) {
					s = format!("{s} {k}");
				} else if is_next_leftstart && !z.contains(k) {
					s = format!("{s} {k}");
				} else {
					s += k.to_string().as_str();
					is_next_rightstart = false;
					is_next_leftstart = false;
				}
			}

			println!("notchain: {notchain}");
			println!("{}", s);
		}

		let (_, (notchain, j)) = sorted[0].to_owned();

		let mut init: YongDictSpellWords = HashMap::new();
		let mut wordpsells = Vec::new();
		for (word, withspecifieds) in j.into_iter() {
			let both = [
				&withspecifieds.other_spells,
				&withspecifieds.specified_for_make_word_spells,
			]
			.into_iter()
			.flatten();
			for spell in both {
				if let Some(mv) = init.get_mut(spell) {
					// if mv.iter().all(|w| w != &word) {
					if !mv.contains(&word) {
						//綴の長さに応じて追加しないようにするかも
						mv.push(word.clone());
					}
				} else {
					init.insert(spell.clone(), vec![word.clone()]);
				}
			}
			for spell in &withspecifieds.specified_for_make_word_spells {
				wordpsells.push(("^".to_string() + &spell, vec![word.clone()]));
			}
		}
		wordpsells.extend(init.into_iter());

		wordpsells
	}

	fn flip_word_spells(&self, dict: YongDictWordSpells) -> YongDictSpellWords {
		let translater = match self {
			Self::CjqYongPy => WordSpellsEntry::to_cjq_xxyx_py,
			Self::CjheadYongcomposedPy => WordSpellsEntry::to_cangjie1_xxyx_py,
			Self::Cjmain => WordSpellsEntry::to_cjmain,
			Self::Free => WordSpellsEntry::just_pass_all_spells,
		};
		let j = dict.iter().filter_map(|t| {
			let spells = translater(WordSpellsEntry::from_tuple(t));
			if spells.len() > 0 {
				Some((t.0.clone(), spells))
			} else {
				None
			}
		});
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
	pub spells: Vec<SpecifySpelling>,
}

pub struct SpellsForMakeWord {
	shorts: Vec<String>,
	wordsrc: Vec<String>,
}

impl WordSpellsEntry {
	fn from_tuple(t: (&String, &Vec<SpecifySpelling>)) -> Self {
		WordSpellsEntry {
			word: t.0.to_owned(),
			spells: t.1.to_owned(),
		}
	}
	fn get_xxyx_bihuas(&self) -> Vec<String> {
		let xxs = self
			.spells
			.iter()
			.filter(|s| s.spelling == YongSpelling::Xxyx);
		let bihuas: Vec<String> = xxs
			.filter_map(|ss| {
				let s = ss.spell.as_bytes();
				if let Some((_, rem)) = s.split_first() {
					if rem.len() > 0 {
						Some(String::from_utf8(rem.to_vec()).unwrap())
					} else {
						None
					}
				} else {
					None
				}
			})
			.collect();
		bihuas
	}
	fn just_pass_all_spells(self) -> Vec<String> {
		self.spells.into_iter().map(|ss| ss.spell).collect()
	}

	pub fn to_cjmain_with_specifier<P: AsRef<Path>>(
		self,
		swap_dict: &HashMap<String, String>,
		shuangpin_table: P,
	) -> WithMakeWordSpecifier {
		let xxs = self
			.spells
			.iter()
			.filter(|s| s.spelling == YongSpelling::Xxyx);
		let cjs //: Vec<&SpecifySpelling>
			= self.spells.iter().filter(|s| s.spelling == YongSpelling::Cangjie
		);
		// .collect();
		let bihuas = self.get_xxyx_bihuas();
		let py = get_shuangpin_tone(
			self.word.chars().next().expect("at least one length word"),
			shuangpin_table,
		);
		let py_fix: String = if let Some(py) = py {
			py.to_string()
		} else {
			"vva".to_string()
		};

		// let swapdict = to_swap_key_dict(swap_dict);

		let cjs_len = cjs.clone().count();
		let specified_require = |spell: &str| {
			if let Some(head) = spell.chars().next() {
				head != 'x' || cjs_len == 1 || spell.len() == 1
			} else {
				false
			}
		};

		let cjfixes: Vec<WithMakeWordSpecifier> = cjs
			.map(|cj| {
				let ok_to_add_specified = specified_require(&cj.spell);
				let spe: String = cj
					.spell
					.chars()
					.map(|c| swap_key(c.to_string(), &swap_dict))
					.collect();
				let fls = fluent_list("spell/fluent.txt");
				// let nfs = not_fluent_pairs();
				let mut chs = spe.chars();
				let first = chs.next().expect("empty spell?");

				let spellfor = if let Some(second) = chs.next() {
					if let Some(third) = chs.next() {
						let mut spell = spe.clone();
						let rem = spell.split_off(3);

						let head: String = if is_left_key(second) != is_left_key(third)
							|| fls.iter().any(|fluent| {
								(second.to_string() + &third.to_string()).is_prefix_of(&fluent)
								// || (third.to_string() + &second.to_string()).is_prefix_of(&fluent)
								// 滑らかな運指は対称ではない。gc:fluent, but cg
							}) {
							spell.clone() + "+"
						} else {
							spell.clone() + if is_left_key(third) { "," } else { "." }
						};

						WithMakeWordSpecifier {
							specified_for_make_word_spells: if ok_to_add_specified {
								vec![head.clone() + &rem + &py_fix]
							} else {
								vec![]
							},
							other_spells: if &rem.len() > &0 {
								vec![head.clone() + &rem, spell.clone() + "a" + &py_fix]
							} else {
								vec![spe.clone(), head.clone()]
							},
						}
					} else {
						WithMakeWordSpecifier {
							specified_for_make_word_spells: if ok_to_add_specified {
								vec![spe.clone() + "a" + &py_fix]
							} else {
								vec![]
							},
							other_spells: vec![spe.clone()],
						}
					}
				} else {
					WithMakeWordSpecifier {
						specified_for_make_word_spells: if ok_to_add_specified {
							vec![spe.clone() + "a" + &py_fix]
						} else {
							vec![]
						},
						other_spells: vec![spe.clone()],
					}
				};

				let mut other_spells = spellfor.other_spells;
				other_spells.push(format!("ah{}...{}", py_fix, spe.clone()));
				WithMakeWordSpecifier {
					specified_for_make_word_spells: spellfor.specified_for_make_word_spells,
					other_spells,
				}
			})
			.collect();

		let (specified, other_spells): (Vec<_>, Vec<_>) = cjfixes
			.into_iter()
			.map(|w| (w.specified_for_make_word_spells, w.other_spells))
			.unzip();

		WithMakeWordSpecifier {
			specified_for_make_word_spells: specified.into_iter().flatten().collect(),
			other_spells: other_spells.into_iter().flatten().collect(),
		}

		// cjfixes.iter().flat_map(|spells| {
		// 	let mut withpy: Vec<String> = spells.wordsrc.iter().map(|w| {
		// 		w.to_owned() + &py_fix
		// 	}).collect();
		// 	withpy.extend(spells.shorts.clone());
		// 	withpy
		// }).collect()
	}
	fn to_cjmain(self) -> Vec<String> {
		let xxs = self
			.spells
			.iter()
			.filter(|s| s.spelling == YongSpelling::Xxyx);
		let cjs = self
			.spells
			.iter()
			.filter(|s| s.spelling == YongSpelling::Cangjie);
		let bihuas = self.get_xxyx_bihuas();
		let py = get_shuangpin_tone(
			self.word.chars().next().expect("at least one length word"),
			"shuang/xiaoque.txt",
		);
		let py_fix: String = if let Some(py) = py {
			py.to_string()
		} else {
			"vva".to_string()
		};

		let swapdict = to_swap_key_dict("spell/swap-start.txt");
		let cjfixes: Vec<SpellsForMakeWord> = cjs
			.map(|cj| {
				let spe: String = cj
					.spell
					.chars()
					.map(|c| swap_key(c.to_string(), &swapdict))
					.collect();
				let fls = fluent_list("spell/fluent.txt");
				// let nfs = not_fluent_pairs();
				let mut chs = spe.chars();
				let first = chs.next().expect("empty spell?");

				let spellfor = if let Some(second) = chs.next() {
					if let Some(third) = chs.next() {
						let mut spell = spe.clone();
						let rem = spell.split_off(3);

						let head: String = if is_left_key(second) != is_left_key(third)
							|| fls.iter().any(|fluent| {
								(second.to_string() + &third.to_string()).is_prefix_of(&fluent)
									|| (third.to_string() + &second.to_string()).is_prefix_of(&fluent)
							}) {
							spell.clone() + "+"
						} else {
							spell.clone() + if is_left_key(third) { "," } else { "." }
						};

						SpellsForMakeWord {
							wordsrc: if &rem.len() > &0 {
								vec![head.clone() + &rem, spell + "a"]
							} else {
								vec![head.clone()]
							},
							shorts: if &rem.len() > &0 {
								vec![head.clone() + &rem]
							} else {
								vec![spe.clone()]
							},
						}
					} else {
						SpellsForMakeWord {
							wordsrc: vec![
								// spe.clone() + if is_left_key(second) {","} else {"."} + "+a"
								spe.clone() + "a",
							],
							shorts: vec![spe.clone()],
						}
					}
				} else {
					SpellsForMakeWord {
						wordsrc: vec![
							// spe.clone() + if is_left_key(first) {"vd"} else {"zk"} + "+a"
							spe.clone() + "a",
						],
						shorts: vec![spe.clone()],
					}
				};

				let mut shorts = spellfor.shorts;
				shorts.push(format!("apy{}...{}", py_fix, spe.clone()));
				SpellsForMakeWord {
					wordsrc: spellfor.wordsrc,
					shorts,
				}
			})
			.collect();

		cjfixes
			.iter()
			.flat_map(|spells| {
				let mut withpy: Vec<String> = spells
					.wordsrc
					.iter()
					.map(|w| w.to_owned() + &py_fix)
					.collect();
				withpy.extend(spells.shorts.clone());
				withpy
			})
			.collect()
	}

	fn to_cangjie1_xxyx_py(self) -> Vec<String> {
		let xxs = self
			.spells
			.iter()
			.filter(|s| s.spelling == YongSpelling::Xxyx);
		let cjs = self
			.spells
			.iter()
			.filter(|s| s.spelling == YongSpelling::Cangjie);
		let bihuas = self.get_xxyx_bihuas();
		let cj_edge = cjs
			.filter_map(|ss| {
				let s = ss.spell.as_bytes();
				if let (Some(first), Some(last)) = (s.first(), s.last()) {
					Some((first, last))
				} else {
					None
				}
			})
			.next();
		let bihuas_composed: Vec<(&String, String)> = {
			let mut ordered = bihuas.clone();
			ordered.sort_by(|s, d| s.chars().count().cmp(&d.chars().count()));
			if let Some(longest) = ordered.last() {
				let composed: Option<char> = {
					let x3 = longest.chars().nth(1);
					let x4 = longest.chars().nth(2);
					let c3 = BihuaXxyx::from_aeuio(x3.unwrap_or('q'));
					let c4 = BihuaXxyx::from_aeuio(x4.unwrap_or('q'));
					if let (Some(c3), Some(c4)) = (c3, c4) {
						Some(c3.chain_as(c4))
					} else {
						None
					}
				};

				let com = composed.unwrap_or('q');
				bihuas
					.iter()
					.map(|h| {
						let v: Vec<char> = h.chars().collect();
						let mut body = h.get(3..).unwrap_or("").to_string();
						// let mut v = body.to_vec();
						body.insert(0, com);
						(h, body)
					})
					.collect()
			} else {
				vec![]
			}
		};
		if bihuas.len() > 0 {
			let py = get_shuangpin_tone(
				self.word.chars().next().expect("at least one length word"),
				"shuang/xiaoque.txt",
			);
			let cj_fix: String = if let Some((start, end)) = cj_edge {
				let o = String::from_utf8(vec![*start]);
				if let Ok(e) = o {
					let pairs = to_swap_key_dict("spell/swap-start.txt");
					let (_, changed) = pairs.iter().fold((false, e.clone()), |acc, d| {
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
			};
			let py_fix: String = if let Some(py) = py {
				py.to_string()
			} else {
				"".to_string()
			};

			let mut bim = vec![];
			let bihua_lens: Vec<_> = bihuas.iter().map(|d| d.len() as u32).collect();
			// 	.sort_by(|s,d| s.len().cmp(&d.len()));
			// let debugcheck = dbg!((self.word, &bihuas_composed));
			let toadd: Vec<String> = bihuas_composed
				.iter()
				.flat_map(|(bihua, xxyx_fix)| {
					spell_polymer_dependent_length_factor(0, &bihua_lens, &cj_fix, xxyx_fix, &py_fix)
				})
				.collect();
			bim.extend(toadd);

			bim
		} else {
			vec![]
		}
	}

	fn to_cjq_xxyx_py(self) -> Vec<String> {
		let xxs = self
			.spells
			.iter()
			.filter(|s| s.spelling == YongSpelling::Xxyx);
		let cjs = self
			.spells
			.iter()
			.filter(|s| s.spelling == YongSpelling::Cangjie);
		let bihuas = self.get_xxyx_bihuas();
		let cj_edge = cjs
			.filter_map(|ss| {
				let s = ss.spell.as_bytes();
				if let (Some(first), Some(last)) = (s.first(), s.last()) {
					Some((first, last))
				} else {
					None
				}
			})
			.next();
		if bihuas.len() > 0 {
			let py = get_shuangpin_tone(
				self.word.chars().next().expect("at least one length word"),
				"shuang/xiaoque.txt",
			);
			let cj_fix: String = if let Some((start, end)) = cj_edge {
				let o = String::from_utf8(vec![*end]);
				if let Ok(e) = o {
					// match e.as_str() {
					// 	"a" => "s",
					// 	"e" => "y",
					// 	"u" => "z",
					// 	"i" => "x",
					// 	"o" => "w",
					// 	s => s.clone()
					// }
					// let pairs = [
					// 	("a", "h"),
					// 	("e", "c"),
					// 		("c", "w"),
					// 	("u", "x"),
					// 	("i", "z"),
					// 	("o", "s"),
					// 	("s", "y"),
					// 	("b", "d")
					// ];
					let pairs = to_swap_key_dict("spell/swap.txt");
					let (_, changed) = pairs.iter().fold((false, e.clone()), |acc, d| {
						let (has_changed, acc) = acc;
						if !has_changed && acc.as_str() == d.0 {
							(true, d.1.to_string())
						} else {
							(has_changed, acc)
						}
					});
					changed
				} else {
					"j".to_string()
				}
			} else {
				"j".to_string()
			};
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
				.flat_map(|xxyx_fix| spell_polymer(0, &cj_fix, xxyx_fix, &py_fix))
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

fn spell_polymer_dependent_length_factor(
	id: u32,
	length_factor: &Vec<u32>,
	cj_fix: &str,
	xxyx_fix: &str,
	py_fix: &str,
) -> Vec<String> {
	match id {
		_ => match xxyx_fix.len() {
			1 => {
				if length_factor.contains(&1) {
					vec![cj_fix.to_string() + xxyx_fix] //2
				} else {
					vec![]
				}
			}
			2 => vec![
				cj_fix.to_string() + xxyx_fix,           //3
				cj_fix.to_string() + xxyx_fix + &py_fix, //6
			],
			_ => vec![
			// , cj_fix.to_string() + xxyx_fix + &py_fix //5
			],
		},
	}
}
fn spell_polymer(id: u32, cj_fix: &str, xxyx_fix: &str, py_fix: &str) -> Vec<String> {
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
		// 1 => match xxyx_fix.len() {
		// 	1 => vec![cj_fix.to_string() + xxyx_fix]  //2
		// , 2 => vec![
		// 	cj_fix.to_string() + xxyx_fix             //3
		// 	, cj_fix.to_string() + xxyx_fix + &py_fix //6
		// 	]
		// , 3 => vec![
		// 	cj_fix.to_string() + xxyx_fix,            //4
		// 	cj_fix.to_string() + xxyx_fix + &py_fix]  //7
		// , _ => vec![
		// 	cj_fix.to_string() + xxyx_fix + &py_fix   //8+
		// ]
		// }
		// ,
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
