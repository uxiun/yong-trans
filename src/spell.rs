use std::{
	collections::HashMap,
	ffi::OsStr,
	fmt::Debug,
	fs::{self, File, OpenOptions},
	hash::Hash,
	io::{self, BufWriter, Error, Write},
	iter::zip,
	marker::PhantomData,
	path::Path,
	time::{Duration, Instant, SystemTime, UNIX_EPOCH},
};

use chrono::Local;
use humantime::format_duration;
use itertools::Itertools;
use num_bigint::ToBigUint;
use permutation_iterator::Permutor;

use crate::{
	kt::{get_yongdictwordspells, read_lines, to_swap_key_dict, YongDictWordSpells},
	out::{spell_words_dict_tostring, WithMakeWordSpecifier, YongDictSpellWords},
	parser::read_line_alpha_entry,
	repeat::word_withspecifiers,
	util::{hashmap_flip, now_string},
};

pub fn main() {
	let j = dbg!([BihuaXxyx::Shu.chain_as(BihuaXxyx::Zhe)]);
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum YongSpelling {
	Xxyx,
	Cangjie,
	Free,
}

pub enum WordSpellsLineFormat {
	AddShorter,
	Free,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct SpecifySpelling {
	pub spelling: YongSpelling,
	pub spell: String,
}
impl SpecifySpelling {}

#[derive(Debug, Clone, Copy)]
pub enum BihuaXxyx {
	Shu,  //竖 I 巾
	Dian, //点 、广
	Zhe,  //折 く 录
	Heng, //横 一
	Pie,  //撇 白
}
impl BihuaXxyx {
	pub fn from_aeuio(e: char) -> Option<Self> {
		match e {
			'a' => Some(Self::Shu),
			'e' => Some(Self::Dian),
			'u' => Some(Self::Zhe),
			'i' => Some(Self::Heng),
			'o' => Some(Self::Pie),
			_ => None,
		}
	}
	fn to_aeuio(&self) -> char {
		match self {
			Self::Shu => 'a',
			Self::Dian => 'e',
			Self::Zhe => 'u',
			Self::Heng => 'i',
			Self::Pie => 'o',
		}
	}
	pub fn chain_as(&self, s: Self) -> char {
		let lines = read_lines("spell/xxyx.txt").expect("correct xxyx spell filepath");
		let di = lines
			.filter_map(|f| {
				if let Ok(a) = f {
					read_line_alpha_entry(a)
				} else {
					None
				}
			})
			.map(|a| (a.value, a.key))
			.collect::<HashMap<_, _>>();
		// let di = dbg!(di);
		let key = self.to_aeuio().to_string() + &s.to_aeuio().to_string();
		*di.get(&key).expect(&format!(
			"
		key={},
		complete xxyx spell rule.",
			&key
		))
	}
}

// pub fn makeword_cjmain()
// -> std::io::Result<()>
// {
// 	let file = File::create("cjmain-word-rule.txt")?;
// 	let mut file = BufWriter::new(file);

// }

// fn code_rule_gen(possible_charlens: Vec<u32>)
// -> String
// {
// 	let mut charlens = possible_charlens.clone();
// 	charlens.sort();
// 	(2..5).map(|wordlen| {
// 		possible_charlens.iter().permutations(wordlen)
// 			.map(|lens| {

// 			})
// 	})
// }

fn code_py(spellen: u32, nth: u32) -> Vec<String> {
	(0..3)
		.map(|i| format!("+p{}{}", nth, spellen - i))
		.rev()
		.collect()
}

fn pformat(charnth: u32, keynth: u32) -> String {
	format!("p{}{}", charnth, keynth)
}

fn plusformat(charnth: u32, keynth: u32) -> String {
	format!("+p{}{}", charnth, keynth)
}

fn code_head(wordlen: u32) -> String {
	match wordlen {
		2 => {
			"p11+p12+p21+p22+p23+p13".to_string()
			// let h: String = (1..3).map(|i| {
			// 	(1..3).map(|j| pformat(i, j)
			// 	).collect::<String>()
			// }).collect();
		}
		_ => "".to_string(),
	}
}

fn code_mid(spellen: u32, charnth: u32) -> String {
	let i = spellen - 3;
	if spellen < 9 {
		"".to_owned()
	} else {
		(0..spellen - 8)
			.map(|f| plusformat(charnth, f + 4))
			.collect()
	}
}

#[derive(Debug, Clone, Copy)]
struct RuleKey {
	char_index: i8,
	key_index: u8,
}

enum TargetLenBoundary {
	Equal,
	AboveOrEq,
}

type RuleSpell = Vec<Result<String, RuleKey>>;
struct YongRule {
	targetlen: u8,
	boundary: TargetLenBoundary,
	spell: RuleSpell,
}

fn rulespell_tostring(rulespell: &RuleSpell) -> String {
	let s = rulespell
		.into_iter()
		.map(|item| match item {
			Err(key) => format!(
				"{}{}{}",
				if key.char_index < 0 { "p" } else { "n" },
				key.char_index.abs(),
				key.key_index
			),
			Ok(s) => s.clone(),
		})
		// .collect::<Vec<_>>()
		;
	Itertools::intersperse(s, "+".to_string()).collect()
}

impl TryInto<String> for YongRule {
	type Error = &'static str;
	fn try_into(self) -> Result<String, Self::Error> {
		let ok = self
			.spell
			.iter()
			.filter_map(|d| d.clone().err())
			.all(|rulekey| {
				(match self.boundary {
					TargetLenBoundary::AboveOrEq => true,
					TargetLenBoundary::Equal => rulekey.char_index.abs() as u8 <= self.targetlen,
				}) && (rulekey.key_index <= 9)
			});
		if ok {
			Ok(format!(
				"code_{}{}={}",
				match self.boundary {
					TargetLenBoundary::AboveOrEq => "a",
					TargetLenBoundary::Equal => "e",
				},
				self.targetlen,
				rulespell_tostring(&self.spell)
			))
		} else {
			Err("indexes in rulekey is out of range for targetlen")
		}
	}
}

pub fn perm_swap_table<P: AsRef<Path>>(
	path: P,
	takelen: Option<usize>,
) -> Vec<HashMap<String, String>> {
	let predefined = to_swap_key_dict(path);
	let allkeys = "qwertyuiopasdfghjklzxcbmnv";
	let keys = allkeys
		.chars()
		.filter(|c| !predefined.contains_key(&c.to_string()))
		.collect_vec();

	let allperms = keys.clone().into_iter().permutations(keys.len());
	let perms = if let Some(len) = takelen {
		allperms
			.take(len)
			.map(|p| {
				HashMap::from_iter(zip(keys.clone(), p).map(|(k, v)| (k.to_string(), v.to_string())))
			})
			.collect_vec()
	} else {
		allperms
			.map(|p| {
				HashMap::from_iter(zip(keys.clone(), p).map(|(k, v)| (k.to_string(), v.to_string())))
			})
			.collect_vec()
	};

	perms
}

pub fn perm_swap_table_process<P: AsRef<Path>, Fsort, Fmap, U, V>(
	path: P,
	takelen_max: Result<usize, usize>,
	sortf: Fsort,
	mapf: Fmap,
) -> Vec<(Vec<char>, U)>
where
	Fmap: Fn(HashMap<char, char>) -> U,
	Fsort: Fn(&U) -> V,
	V: Ord + Clone + Copy,
	U: Clone,
{
	let predefined: HashMap<char, char> = to_swap_key_dict(path)
		.into_iter()
		.filter_map(|(k, v)| {
			let k = k.chars().next()?;
			let v = v.chars().next()?;
			Some((k, v))
		})
		.collect();
	// dbg!(&predefined);

	let bans = predefined.values().collect_vec();

	let allkeys = "qwertyuiopasdfghjklzxcbmnv";
	let keys = allkeys.chars().filter(|c| !bans.contains(&c)).collect_vec();

	// dbg!(&keys);

	// let perms = keys
	// 	.clone()
	// 	.into_iter()
	// 	.permutations(keys.len())
	// 	.collect_vec();

	// dbg!(&perms.len());

	// let ts = perms
	// 	.into_iter()
	// 	.enumerate()
	// 	.map(|(i, p)| {
	// 		dbg!(i);

	// 		let mut h = HashMap::from_iter(
	// 			zip(keys.clone(), p.clone()), // .map(|(k, v)| (k.to_string(), v.to_string()))
	// 		);
	// 		h.extend(predefined.iter());

	// 		(
	// 			restruct_perm_items(allkeys.chars(), predefined.to_owned(), p),
	// 			mapf(h),
	// 		)
	// 	})
	// 	.collect_vec();

	let len = keys.len();
	let mapper = |perm: &Vec<char>| {
		let mut h = HashMap::from_iter(zip(keys.clone(), perm.clone()));
		h.extend(predefined.iter());

		// let h = restruct_perm_items(allkeys.chars(), predefined, perm.to_owned());

		mapf(h)
	};

	match takelen_max {
		Ok(takelen) => {
			let mut tosort = keys
				.clone()
				.into_iter()
				.permutations(len)
				.take(takelen)
				.map(|cs| (cs.clone(), mapper(&cs)))
				.collect_vec();

			tosort.sort_by_key(|(_, u)| sortf(u));
			tosort
		}
		Err(max) => {
			let ts = keys.clone().into_iter().permutations(len);
			shrink_big_iterator(max, ts, &sortf, mapper)
		}
	}

	// let mut tosort = ts;
	// tosort.sort_by_key(|(_, x)| sortf(x));
	// tosort
}

pub type PermTargetAndDict = (Vec<char>, HashMap<char, char>);

#[derive(Debug, Clone)]
pub struct SwapDictChars {
	allchars: Vec<char>,
	pub fromchars: Vec<char>,
	pub tochars: Vec<char>, // will be permutation.
	pub predefined: HashMap<char, char>,
	blacklist_to_from: HashMap<char, char>,
}

impl SwapDictChars {
	pub fn index(&self, c: char) -> Option<usize> {
		self
			.allchars
			.iter()
			.enumerate()
			.find_map(|(i, d)| if *d == c { Some(i) } else { None })
	}

	pub fn char_by_index(&self, i: usize) -> Option<char> {
		self.allchars.get(i).map(|c| *c)
	}

	pub fn new<P: AsRef<Path>>(
		allkeys: &'static str,
		path: P,
		blacklist_from: &'static str,
		blacklist_to: &'static str,
	) -> Self {
		if blacklist_from.len() != blacklist_to.len() {
			panic!("each blacklist must have same length");
		}

		let blacklist_map: HashMap<char, char> =
			zip(blacklist_to.chars(), blacklist_from.chars()).collect();

		let predefined: HashMap<char, char> = to_swap_key_dict(path)
			.into_iter()
			.filter_map(|(k, v)| {
				let k = k.chars().next()?;
				let v = v.chars().next()?;
				Some((k, v))
			})
			.collect();

		// sort! (将来次のpermが計算できるようになるかもなので順番を正規化しとく)
		let mut allchars: Vec<char> = allkeys.chars().collect();
		allchars.sort();

		// swap map is HashMap<mapto, mapfrom>
		let frombans = predefined.keys().collect_vec();
		let tobans = predefined.values().collect_vec();

		let tochars = allchars
			.clone()
			.into_iter()
			.filter(|c| !tobans.contains(&c) && !blacklist_to.contains(*c))
			.collect_vec();
		let fromchars: Vec<char> = allchars
			.clone()
			.into_iter()
			.filter(|c| !frombans.contains(&c) && !blacklist_from.contains(*c))
			.collect_vec();

		Self {
			allchars,
			tochars,
			fromchars,
			predefined,
			blacklist_to_from: blacklist_map,
		}
	}

	pub fn restruct_swap_dict(&self, perm: &Vec<char>) -> HashMap<char, char> {
		let mut selectedmap: HashMap<char, char> =
			HashMap::from_iter(zip(self.tochars.clone(), self.fromchars.clone()));

		// let trans = perm.into_iter()
		// 	.map(|c| {
		// 		self.blacklist_to_from.get(c)
		// 		.unwrap_or(c)
		// 		.to_owned()
		// 	})
		// 	;

		let mut h = HashMap::from_iter(zip(self.fromchars.clone(), perm.clone()));

		h.extend(self.predefined.clone());

		h
	}

	pub fn permutations(&self) -> itertools::Permutations<std::vec::IntoIter<char>> {
		self
			.tochars
			.clone()
			.into_iter()
			.permutations(self.tochars.len())
	}
}

struct PermutorConverter<F, T> {
	fromU64: F,
	toU64: T,
}

struct PermSpecify<It, Iu> {
	specials: It,
	positions: Iu,
}

pub fn pure_perm<T>(items: &[T]) -> Vec<&T> {
	Permutor::new(items.len() as u64)
		.map(|u| items.get(u as usize).unwrap())
		.collect()
}

impl<FromU64, ToU64, T> PermutorConverter<FromU64, ToU64>
where
	FromU64: Fn(u64) -> T,
	ToU64: Fn(T) -> u64,
	T: Clone,
{
	fn perm_specify<N>(&self, normals: &[T], specials_slots: &[(Vec<T>, Vec<N>)]) -> Vec<T>
	where
		// T: Debug,
		N: Eq + Copy + From<usize> + Into<usize> + Hash, // where I: IntoIterator<Item = T>,
	{
		let special_map: HashMap<N, T> = specials_slots
			.into_iter()
			.filter_map(|(specials, slots)| {
				if specials.len() == slots.len() {
					let specialmix = self.perm(specials.len() as u64);

					// let z: HashMap<N, T> = zip(slots.into_iter().map(|s| *s), specialmix).collect();
					// Some(z.into_iter())

					Some(zip(slots.into_iter().map(|s| *s), specialmix))
				} else {
					None
				}
			})
			.flatten()
			.collect();

		let normalmix = pure_perm(normals);
		let mut noraml_iter = normalmix.into_iter();
		// let slots = special_slots.into_iter().map(|i| )

		let mut v: Vec<T> = vec![];
		for i in 0..(normals.len() + special_map.len()) {
			let n: N = i.into();
			let k = special_map.get(&n).map(|s| s.clone());

			//この書き方だと unwrap_or しなくていいときでも noraml_iter.next() されてしまう

			// let k: T = k.unwrap_or({
			// 	println!("unwrap![{i}]");
			// 	noraml_iter.next().unwrap()
			// }
			// );

			let k = match k {
				Some(s) => s,
				None => noraml_iter.next().unwrap().clone(),
			};

			v.push(k);
		}

		v

		// (0..(normals.len() + specials.len()))
		// 	.map(|i| {
		// 		let n: N = i.into();
		// 		if special_slots.contains(&n)
		// 		{
		// 			special_iter
		// 		} else {
		// 			noraml_iter
		// 		}.next().unwrap()
		// 	}
		// ).collect()
	}

	fn action_specify<M, U, N>(
		&self,
		max: u64,
		map: M,
		normals: &[T],
		specials_slots: &[(Vec<T>, Vec<N>)],
	) -> U
	where
		N: Eq + Copy + From<usize> + Into<usize> + Hash,
		M: Fn(Vec<T>) -> U,
	{
		map(self.perm_specify(normals, specials_slots))
	}

	fn results_specify<M, U, F, N>(
		&self,
		max: u64,
		map: M,
		// accum: Fv,
		stopper: F,
		normals: &[T],
		specials_slots: &[(Vec<T>, Vec<N>)],
	) -> Vec<U>
	where
		M: Fn(Vec<T>) -> U,
		N: Eq + Copy + From<usize> + Into<usize> + Hash,
		// Fv: Fn(Vec<U>) -> V,
		F: Fn(&Vec<U>) -> bool,
	{
		let mut results = vec![];
		while !stopper(&results) {
			let u = self.action_specify(max, &map, normals, specials_slots);
			results.push(u);
		}
		results
	}

	fn repeat_specify<M, U, F, A, N>(
		&self,
		max: u64,
		map: M,
		stopper: F,
		and: A,
		normals: &[T],
		specials_slots: &[(Vec<T>, Vec<N>)],
	) where
		M: Fn(Vec<T>) -> U,
		F: Fn(&Vec<U>) -> bool,
		A: Fn(Vec<U>) -> bool,
		N: Eq + Copy + From<usize> + Into<usize> + Hash,
	{
		loop {
			let now = Instant::now();
			let res = self.results_specify(max, &map, &stopper, normals, specials_slots);
			if and(res) {
				// self.repeat(max, map, stopper, and)
			} else {
				// println!("break loop");
				break;
			}
			println!("{}", format_duration(now.elapsed()));
		}
	}
}

impl<FromU64, ToU64, T> PermutorConverter<FromU64, ToU64>
where
	FromU64: Fn(u64) -> T,
	ToU64: Fn(T) -> u64,
{
	fn perm(&self, max: u64) -> Vec<T> {
		let perms = Permutor::new(max);
		perms.map(&self.fromU64).collect_vec()
	}

	fn action<M, U>(&self, max: u64, map: M) -> U
	where
		M: Fn(Vec<T>) -> U,
	{
		map(self.perm(max))
	}

	fn results<M, U, F>(
		&self,
		max: u64,
		map: M,
		// accum: Fv,
		stopper: F,
	) -> Vec<U>
	where
		M: Fn(Vec<T>) -> U,
		// Fv: Fn(Vec<U>) -> V,
		F: Fn(&Vec<U>) -> bool,
	{
		let mut results = vec![];
		while !stopper(&results) {
			let u = self.action(max, &map);
			results.push(u);
		}
		results
	}

	fn repeat<M, U, F, A>(&self, max: u64, map: M, stopper: F, and: A)
	where
		M: Fn(Vec<T>) -> U,
		F: Fn(&Vec<U>) -> bool,
		A: Fn(Vec<U>) -> bool,
	{
		loop {
			let res = self.results(max, &map, &stopper);
			if and(res) {
				// self.repeat(max, map, stopper, and)
			} else {
				// println!("break loop");
				break;
			}
		}
	}
}

struct FileAction<P, Save, Load> {
	save: Save,
	load: Load,
	path: P,
}

impl<P, Save, Load, D> FileAction<P, Save, Load>
where
	P: AsRef<Path>,
	Save: Fn(D) -> String,
	Load: Fn(String) -> D,
{
	pub fn save(&self, data: D) -> std::io::Result<()> {
		let s = (self.save)(data);
		// let f = File::open(&self.path)?;
		let f = OpenOptions::new()
			.create(true)
			.write(true)
			.open(&self.path)?;
		let mut f = BufWriter::new(f);
		f.write_all(s.as_bytes())?;
		f.flush()
	}

	pub fn append(&self, data: D) -> std::io::Result<()> {
		let s = (self.save)(data);
		// let f = File::open(&self.path)?;
		let f = OpenOptions::new()
			.create(true)
			.append(true)
			.open(&self.path)?;
		let mut f = BufWriter::new(f);
		f.write_all(s.as_bytes())?;
		f.flush()
	}

	pub fn load(&self) -> Result<D, Error> {
		let s = fs::read_to_string(&self.path);

		s.map(&self.load)
	}
}

pub fn specifier_to_wordspells(
	j: Vec<(String, WithMakeWordSpecifier)>,
) -> Vec<(String, Vec<String>)> {
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

impl SwapDictChars {
	pub fn write_perm_dict<P, I>(
		&self,
		perm: &Vec<char>,
		shuangpin_table: P,
		table_paths: I,
		save_path: P,
		// nonchain_goal: usize,
		// process_perm_chunk: usize,
	) -> io::Result<()>
	where
		I: IntoIterator<Item = P>,
		P: AsRef<Path> + Copy + Clone + Debug,
	{
		let wordspells = get_yongdictwordspells(table_paths);
		let swap_dict = self.restruct_swap_dict(perm);
		let (score, dict) = word_withspecifiers(swap_dict, &wordspells, shuangpin_table);

		let s = spell_words_dict_tostring(specifier_to_wordspells(dict));

		let f = OpenOptions::new()
			.create(true)
			.write(true)
			.open(save_path)?;
		let mut f = BufWriter::new(f);
		f.write_all(s.as_bytes())?;
		f.flush()
	}

	fn specify<S: ToString>(
		&self,
		specials_slots: &[(S, S)],
	) -> (Vec<char>, Vec<(Vec<char>, Vec<usize>)>) {
		specials_slots.into_iter().fold(
			(vec![], vec![]),
			|(mut specialchars, mut ss), (specials, slots)| {
				specialchars.extend(specials.to_string().chars());
				let slot_indice = slots
					.to_string()
					.chars()
					.filter_map(|c| self.index(c))
					.collect_vec();
				ss.push((specials.to_string().chars().collect(), slot_indice));
				(specialchars, ss)
			},
		)
	}

	pub fn perm<P, Q, I, S>(
		&self,
		shuangpin_table: P,
		table_paths: I,
		save_path: Q,
		nonchain_goal: usize,
		process_perm_chunk: usize,
		specials_slots: &[(S, S)],
	) where
		I: IntoIterator<Item = P>,
		P: AsRef<Path> + Clone + Debug + Copy,
		Q: AsRef<Path> + Clone + Debug + Copy,
		S: ToString,
	{
		let (specialchars, specials_slots) = self.specify(specials_slots);
		let normalchars: Vec<char> = self
			.tochars
			.iter()
			.filter(|c| !specialchars.contains(c))
			.map(|c| *c)
			.collect();

		let max = self.tochars.len() as u64;

		let fromU = |u: u64| self.tochars.get(u as usize).unwrap().to_owned();

		let toU = |c: char| self.tochars.iter().find_position(|d| **d == c).unwrap().0 as u64;

		let dict = get_yongdictwordspells(table_paths);

		type ScoredResult = (usize, Vec<(String, crate::out::WithMakeWordSpecifier)>);
		type WithPerm = (Vec<char>, ScoredResult);

		let map = |d: Vec<char>| -> WithPerm {
			let swap_dict = self.restruct_swap_dict(&d);
			let scored = word_withspecifiers(swap_dict, &dict, shuangpin_table.clone());

			(d, scored)
		};

		let stopper = |results: &Vec<WithPerm>| results.len() >= process_perm_chunk;

		let and = |results: Vec<WithPerm>| {
			type ScoredPerm = (usize, Vec<char>);
			let load = |s: String| -> Vec<ScoredPerm> {
				s.lines()
					.filter_map(|l| {
						let mut cols = l.split("\t");
						let score = cols
							.next()
							.map(|s| usize::from_str_radix(s, 10).ok())
							.flatten()?;
						let perm = cols.next().map(|s| s.chars().collect())?;

						Some((score, perm))
					})
					.collect()
			};

			let save = |scoreds: Vec<ScoredPerm>| -> String {
				Itertools::intersperse(
					scoreds
						.into_iter()
						.map(|(score, perm)| score.to_string() + "\t" + &perm.into_iter().collect::<String>()),
					"\n".to_string(),
				)
				.collect::<String>()
					+ "\n"
			};

			let fa = FileAction {
				load,
				save,
				path: save_path,
			};

			let mut data: Vec<ScoredPerm> = results
				.into_iter()
				.map(|(perm, (score, dict))| (score, perm))
				.collect();

			data.sort_by_key(|(score, _)| *score);

			let (top_score, efficient_perms) = {
				let mut most_effs = vec![];
				let mut last_score = None;
				for (score, perm) in data.iter() {
					if last_score.map(|s| s == score).unwrap_or(true) {
						most_effs.push(perm);
						last_score = Some(score);
					} else {
						break;
					}
				}

				(last_score, most_effs)
			};

			// print
			if let Some(score) = top_score {
				println!("{score}");
				for perm in efficient_perms {
					let s: String = perm.into_iter().collect();
					println!("  {s}");
				}
			}
			let continu = top_score.map(|s| *s > nonchain_goal).unwrap_or(true);

			if let Err(e) = fa.append(data) {
				print!("error occured while save: {e}");
			}

			continu
		};

		let permutor = PermutorConverter {
			fromU64: fromU,
			toU64: toU,
		};

		let max = self.tochars.len() as u64;
		if specials_slots.len() > 0 {
			permutor.repeat_specify(max, map, stopper, and, &normalchars, &specials_slots)
		} else {
			println!("no specials_slots, run normal repeat");

			permutor.repeat(max, map, stopper, and);
		}
	}
}

pub fn restruct_swap_dict<P: AsRef<Path>>(
	path: P,
	perm: &Vec<char>,
	predefined: &HashMap<char, char>,
	keys: &Vec<char>,
) -> HashMap<char, char> {
	let bans = predefined.values().collect_vec();

	let allkeys = "qwertyuiopasdfghjklzxcbmnv";

	let mut h = HashMap::from_iter(zip(keys.clone(), perm.clone()));
	h.extend(predefined.iter());
	h
}

// pub fn restruct_perm_items<I, U, O, T>(allkeys: I, fixedmap: HashMap<T, T>, keys: U, selectedkeys: O) -> Vec<T>
// where
// 	I: IntoIterator<Item = T>,
// 	O: IntoIterator<Item = T>,
// 	U: IntoIterator<Item = T>,
// 	T: Hash + Eq + Clone,
// {
// 	let mut dict: HashMap<T, T> = zip(selectedkeys, keys).collect();

// 	dict.extend(fixedmap);

// 	keys
// 		.into_iter()
// 		.map(|k| {
// 			if let Some(t) = fixedmap.get(&k) {
// 				t.clone()
// 			} else {
// 				it.next().unwrap()
// 			}
// 		})
// 		.collect()
// }

pub fn shrink_big_iterator<F, U, T, I, V, G>(max: usize, i: I, sortby: F, mapf: G) -> Vec<(T, U)>
where
	I: IntoIterator<Item = T>,
	G: Fn(&T) -> U,
	F: Fn(&U) -> V,
	V: Ord + Copy,
	T: Clone,
	U: Clone,
{
	shrink_big_iterator_sortvalue(max, i, sortby, mapf)
		.into_iter()
		.map(|(_, t)| t)
		.collect()
}

pub fn shrink_big_iterator_sortvalue<F, G, U, T, I, V>(
	max: usize,
	i: I,
	sortby: F,
	mapf: G,
) -> Vec<(V, (T, U))>
where
	I: IntoIterator<Item = T>,
	G: Fn(&T) -> U,
	F: Fn(&U) -> V,
	V: Ord + Copy + Clone,
	T: Clone,
	U: Clone,
{
	println!("called shrink_big_iterator_sortvalue");

	let mut it = i.into_iter();
	let mut l = vec![];
	let mut counter = 0.to_biguint().unwrap();

	let mut v = vec![];
	let mut nowreset = Instant::now();
	let mut nowabsolute = Instant::now();
	println!("start loop");
	loop {
		if let Some(t) = it.next() {
			if v.len() < max {
				let u = mapf(&t);
				v.push((sortby(&u), (t, u)));
				counter += 1.to_biguint().unwrap();
			} else {
				v.sort_by_key(|(v, _)| *v);
				l.push(v[0].clone());

				if max <= l.len() {
					println!(
						"[{}] {} sort and reset: {}",
						counter,
						format_duration(nowabsolute.elapsed()),
						format_duration(nowreset.elapsed())
					);
					nowreset = Instant::now();

					l.sort_by_key(|(v, _)| *v);
					l = l.first().map(|d| vec![d.to_owned()]).unwrap_or(vec![]);
				}
			}
		} else {
			break;
		}
	}

	l.extend(shrink_big_iterator_sortvalue(max, it, sortby, mapf));

	l.sort_by_key(|(v, _)| *v);

	l
}

#[test]
fn time() {
	let now = Local::now();
	dbg!(now.to_string());

	let raw = SystemTime::now();
	let n = raw.duration_since(UNIX_EPOCH).unwrap();
	// dbg!(formatdu)
}

pub fn run_big_iter<F, G, U, T, I, V, P: AsRef<Path>, FnFromString, FnToString, FnPermToString>(
	commitpath: P,
	commit_duration: Duration,
	commit_read: FnFromString,
	commit_tostring: FnToString,
	perm_tostring: FnPermToString,
	max: usize,
	i: I,
	sortby: F,
	mapf: G,
) -> Vec<(V, (T, U))>
where
	I: IntoIterator<Item = T>,
	G: Fn(&T) -> U,
	F: Fn(&U) -> V,
	V: Ord + Copy + Clone,
	T: Clone,
	U: Clone,
	FnToString: Fn(T, U) -> (String, String),
	FnFromString: Fn(String) -> T,
{
	println!("called shrink_big_iterator_sortvalue");

	let mut it = i.into_iter();
	let mut l = vec![];
	let mut counter = 0.to_biguint().unwrap();

	let mut v = vec![];
	let mut nowreset = Instant::now();
	let mut nowcommit = Instant::now();
	let mut nowabsolute = Instant::now();
	println!("start loop");
	loop {
		if let Some(t) = it.next() {
			if v.len() < max {
				let u = mapf(&t);
				v.push((sortby(&u), (t, u)));
				counter += 1.to_biguint().unwrap();
			} else {
				v.sort_by_key(|(v, _)| *v);
				l.push(v[0].clone());

				if max <= l.len() {
					println!(
						"[{}] {} sort and reset: {}",
						counter,
						format_duration(nowabsolute.elapsed()),
						format_duration(nowreset.elapsed())
					);
					nowreset = Instant::now();

					l.sort_by_key(|(v, _)| *v);
					l = l.first().map(|d| vec![d.to_owned()]).unwrap_or(vec![]);
				}

				// if nowcommit.elapsed() > commit_duration {
				// 	nowcommit = Instant::now();
				// 	let file = File::options()
				// 		.read(true)
				// 		.
				// }
			}
		} else {
			break;
		}
	}

	l.extend(shrink_big_iterator_sortvalue(max, it, sortby, mapf));

	l.sort_by_key(|(v, _)| *v);

	l
}

// #[test]
// fn perm() {
// 	let ps = perm_swap_table("spell/swap_predefined.txt", None);

// 	let keys = "qwertyuiopasdfghjklzxcbmnv";
// 	let li = shrink_big_iterator_sortvalue(keys.chars().permutations(keys.len()), |x| {
// 		x.into_iter().map(|c| u32::from(*c)).sum::<u32>()
// 	});

// 	println!("length {}", li.len());
// }
pub type ScoredPerm = (usize, Vec<char>);

pub fn parse_score_perms(src: String) -> Vec<ScoredPerm> {
	src
		.lines()
		.filter_map(|l| {
			let mut cols = l.split("\t");
			let score = cols
				.next()
				.map(|s| usize::from_str_radix(s, 10).ok())
				.flatten()?;
			let perm = cols.next().map(|s| s.chars().collect())?;

			Some((score, perm))
		})
		.collect()

	// rg_sort_head_command_res.lines()
	// 	.filter_map(|s| s.split_ascii_whitespace().nth(1).map(|s| s.chars().collect()) )
	// 	.filter(|cs: &Vec<char>| cs.into_iter().all(|c| c.is_ascii_alphabetic() ) )
	// 	.collect()
}

pub fn default_swapdictchars() -> SwapDictChars {
	SwapDictChars::new("qwertyuiopasdfghjklzxcbmnv", "spell/swap_base", "z", "a")
}

pub fn swap_table_quickcheck(src: String) {
	let swap = default_swapdictchars();
	let sp = parse_score_perms(src);
	for (score, swap_table, perm) in sp
		.into_iter()
		.map(|(score, perm)| (score, swap.restruct_swap_dict(&perm), perm))
	{
		let pretty = swap_table_tostring(swap_table);
		let permstr: String = perm.into_iter().collect();
		println!("↓ {score} {}", permstr);
		println!("{}", pretty);
	}
}

fn swap_table_tostring(swap_table: HashMap<char, char>) -> String {
	let keyboard = "qwertyuiopasdfghjklzxcbmnv";
	let h: HashMap<char, char> = HashMap::from_iter([
		('q', '手'),
		('w', '田'),
		('e', '水'),
		('r', '口'),
		('t', '廿'),
		('y', '卜'),
		('u', '山'),
		('i', '戈'),
		('o', '人'),
		('p', '心'),
		('a', '日'),
		('s', '尸'),
		('d', '木'),
		('f', '火'),
		('g', '土'),
		('h', '竹'),
		('j', '十'),
		('k', '乂'),
		('l', '中'),
		('z', '！'),
		('x', '難'),
		('c', '金'),
		('b', '月'),
		('m', '一'),
		('n', '弓'),
		('v', '女'),
	]);

	// let qwerty_sorted = keyboard
	// 	.chars()
	// 	.map(|c| swap_table.get(&c).unwrap_or(&'？'));

	let leftend = "tgb";
	let y = "y";
	let z = "z";
	let rightend = "plv";

	let slot = [["qwert ", "yuiop"], ["asdfg ", "hjkl"], [" zxc b", " mnv"]];

	Itertools::intersperse(
		slot.into_iter().map(|line| {
			line
				.into_iter()
				.map(|s| {
					s.chars()
						.map(|c| {
							hashmap_flip(&swap_table) //<keyboard, cjkey> => swap_table<cjkey, keyboard>, but swapfile line: <keyboard, cjkey>
								.get(&c)
								.map(|cjkey| h.get(cjkey))
								.flatten()
								.unwrap_or(&'　')
						})
						.collect::<String>()
				})
				.collect()
		}),
		"\n".to_string(),
	)
	.collect()

	// let mut s = String::new();
	// let mut is_next_rightstart = false;
	// let mut is_next_leftstart = true;

	// for key in keyboard.chars().into_iter() {
	// 	let zi = h.get(&key).unwrap_or(&"！");
	// 	if rightend.contains(key) {
	// 		s = format!("{s}{zi}\n");
	// 		is_next_leftstart = true;
	// 	} else if leftend.contains(key) {
	// 		s = format!("{s}{zi}　");
	// 		is_next_rightstart = true;
	// 	} else if is_next_rightstart && !y.contains(key) {
	// 		s = format!("{s}　{zi}");
	// 		is_next_leftstart = false ;
	// 	} else if is_next_leftstart && !z.contains(key) {
	// 		s = format!("{s}　{zi}");
	// 		is_next_leftstart = false ;
	// 	} else {
	// 		s += zi;
	// 		is_next_rightstart = false;
	// 		is_next_leftstart = false;
	// 	}
	// }

	// s
}

pub fn swap_table_literal_tostring(keytocjchar: &HashMap<char, char>) -> String {
	let keyboard = "qwertyuiopasdfghjklzxcbmnv";
	let h: HashMap<char, char> = HashMap::from_iter([
		('q', '手'),
		('w', '田'),
		('e', '水'),
		('r', '口'),
		('t', '廿'),
		('y', '卜'),
		('u', '山'),
		('i', '戈'),
		('o', '人'),
		('p', '心'),
		('a', '日'),
		('s', '尸'),
		('d', '木'),
		('f', '火'),
		('g', '土'),
		('h', '竹'),
		('j', '十'),
		('k', '乂'),
		('l', '中'),
		('z', '！'),
		('x', '難'),
		('c', '金'),
		('b', '月'),
		('m', '一'),
		('n', '弓'),
		('v', '女'),
	]);

	let leftend = "tgb";
	let y = "y";
	let z = "z";
	let rightend = "plv";

	let slot = [["qwert ", "yuiop"], ["asdfg ", "hjkl"], [" zxc b", " mnv"]];

	Itertools::intersperse(
		slot.into_iter().map(|line| {
			line
				.into_iter()
				.map(|s| {
					s.chars()
						.map(|c| {
							keytocjchar
								.get(&c)
								.map(|cjkey| h.get(cjkey))
								.flatten()
								.unwrap_or(&'　')
						})
						.collect::<String>()
				})
				.collect()
		}),
		"\n".to_string(),
	)
	.collect()
}
