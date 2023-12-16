use std::{
	collections::HashMap,
	ffi::OsStr,
	fs::{self, File},
	hash::Hash,
	io::{BufWriter, Write},
	iter::zip,
	marker::PhantomData,
	path::Path,
	time::{Duration, Instant, SystemTime, UNIX_EPOCH},
};

use chrono::Local;
use humantime::format_duration;
use itertools::Itertools;
use num_bigint::ToBigUint;

use crate::{
	d,
	kt::{read_lines, to_swap_key_dict, YongDictWordSpells},
	parser::read_line_alpha_entry,
	util::now_string,
};

pub fn main() {
	let j = d!([BihuaXxyx::Shu.chain_as(BihuaXxyx::Zhe)]);
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
		// let di = d!(di);
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
	dbg!(&predefined);

	let bans = predefined.values().collect_vec();

	let allkeys = "qwertyuiopasdfghjklzxcbmnv";
	let keys = allkeys.chars().filter(|c| !bans.contains(&c)).collect_vec();

	dbg!(&keys);

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
	pub allchars: Vec<char>,
	pub fromchars: Vec<char>,
	pub tochars: Vec<char>,
	pub predefined: HashMap<char, char>,
}

impl SwapDictChars {
	pub fn new<P: AsRef<Path>>(allkeys: &'static str, path: P) -> Self {
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
			.filter(|c| !tobans.contains(&c))
			.collect_vec();
		let fromchars: Vec<char> = allchars
			.clone()
			.into_iter()
			.filter(|c| !frombans.contains(&c))
			.collect_vec();

		Self {
			allchars,
			tochars,
			fromchars,
			predefined,
		}
	}

	pub fn restruct_swap_dict(self, perm: &Vec<char>) -> HashMap<char, char> {
		let mut selectedmap: HashMap<char, char> =
			HashMap::from_iter(zip(self.tochars, self.fromchars.clone()));

		let mut h = HashMap::from_iter(zip(self.fromchars, perm.to_owned()));

		h.extend(self.predefined);

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

#[test]
fn restruct() {
	let s = SwapDictChars::new("qwertyuiopasdfghjklzxcbmnv", "spell/swap_predefined.txt");

	dbg!(&s);
	let perm = &s.permutations().next().unwrap();
	let all = s.allchars.clone();
	let len = all.len();
	let d = s.restruct_swap_dict(perm);
	dbg!(&d);
	for c in all {
		if !d.contains_key(&c) {
			dbg!(c);
		}
	}
	assert_eq!(len, d.len());
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
