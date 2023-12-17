use std::{
	collections::HashMap,
	ffi::OsStr,
	fmt::Debug,
	fs::{self, create_dir, create_dir_all, remove_dir, remove_dir_all, File, OpenOptions},
	hash::Hash,
	io::{BufRead, BufReader, Error, Write},
	marker::PhantomData,
	path::{Path, PathBuf},
	time::{Duration, Instant},
};

use chrono::Local;
use itertools::Itertools;
use num_bigint::ToBigUint;
use permutation_iterator::Permutor;

use crate::{
	kt::{get_yongdictwordspells, YongDictWordSpells},
	out::{spell_words_dict_tostring, WithMakeWordSpecifier, WordSpellsEntry, YongDictSpellWords},
	spell::{restruct_swap_dict, SwapDictChars},
	util::{now_string, sort_groupby},
};

pub struct RepeatActionConst<P> {
	pub path: P,
	pub maxstore: usize,
}

#[derive(Debug, Clone)]
pub struct RepeatAction<P, Eq, Write, Read, Readrev, Sort, Memo, A> {
	// pub pathprefix: &'a str,
	// pub pathpostfix: F,
	pub path: P,
	pub read: Read,
	pub readrev: Readrev,
	pub write: Write,
	pub sort: Sort,
	pub memo: Memo,
	pub duration: Duration,
	pub maxstore: usize,
	// pub stream: S,
	pub action: A,
	pub eq: Eq,
	// phantom: PhantomData<T>,
	// ph: PhantomData<D>,
}

impl<'a, P, T, D, Eq, Write, Read, Readrev, Sort, Memo, V, A>
	RepeatAction<P, Eq, Write, Read, Readrev, Sort, Memo, A>
where
	P: AsRef<Path> + ToString + Clone + AsRef<OsStr>,
	// F: Fn(D) -> String,
	// S: IntoIterator<Item = D>,
	Eq: Fn(&D, &D) -> bool,
	Write: Fn(D, T) -> String,
	Read: Fn(String) -> (Option<V>, Option<D>),
	Readrev: Fn(V, D) -> String,
	// Sort: Fn(Vec<(D,T)>) -> Vec<Vec<(D,T)>>,
	Sort: Fn((D, T)) -> V,
	Memo: Fn(D) -> String, // used as suffix of save file path
	A: Fn(D) -> T,
	D: Debug + Clone,
	T: Debug + Clone,
	V: Debug + Ord + ToString + Hash + Clone,
{
	fn save_at<Pa: AsRef<Path> + Clone>(&self, path: Pa, d: D, t: T) -> std::io::Result<()> {
		let mut file = File::create_new(path)?;
		file.write_all((self.write)(d, t).as_bytes())?;
		file.flush()?;
		Ok(())
	}

	fn save(&self, converts: Vec<&(D, T)>) {
		let now = Local::now();
		let dts = sort_groupby(converts.into_iter().map(|dt| ((self.sort)(dt.clone()), dt)));

		// let vd = dts.clone().into_iter()
		// 	.map(|(v, vc)| (v, vc.into_iter().map(|(d,t) | d).collect_vec() ))
		// 	.collect_vec();
		// dbg!(&vd);

		if let Some((v, topdts)) = dts.first() {
			topdts.into_iter().enumerate().for_each(|(i, (d, t))| {
				let path = Path::new(&self.path).join(&(self.memo)(d.clone()));

				if let Err(e) = self.save_at(path, d.clone(), t.to_owned()) {
					println!("{e} @ {}", (self.memo)(d.to_owned()));
				}
			});
		}

		// if let Err(_) = self.save_at(&self.path, ts.clone(), d.clone()) {
		// 	let s = self.path.to_string() + &now_string();
		// 	if let Err(e) = fs::rename(&self.path, s.as_str()) {
		// 		Err(e)
		// 	} else {
		// 		self.save(ts, d)
		// 	}

		// } else {
		// 	Ok(())
		// }
	}

	fn cursor_proceed(&self, converts: &[(D, T)]) -> std::io::Result<()> {
		if let Some((d, t)) = converts.last() {
			let mut f = OpenOptions::new()
				.truncate(true)
				.write(true)
				.open(&self.cursor_path())?;

			let v = (self.sort)((d.clone(), t.clone()));
			// let s: String = v.to_string() + &(self.memo)(d.to_owned());

			f.write_all((self.write)(d.to_owned(), t.to_owned()).as_bytes())
		} else {
			Ok(())
		}
	}

	pub fn run<Stream: IntoIterator<Item = D>>(&self, stream: Stream, tops: Vec<(Option<V>, D)>) {
		{
			let (isallsame, _) =
				tops
					.clone()
					.into_iter()
					.fold((true, None), |(isallsame, last), (o, d)| match (last, o) {
						(Some(last), Some(v)) => (isallsame && last == v, Some(v)),
						(_, o) => (isallsame && true, o),
					});

			if !isallsame {
				println!("tops value is not same.");
			}
		}

		let topconverts = tops
			.clone()
			.into_iter()
			.map(|(v, d)| (d.clone(), (self.action)(d)))
			.collect_vec();

		let mut converts = vec![];
		let mut now = Instant::now();
		for d in stream {
			let t = (self.action)(d.clone());
			converts.push((d.clone(), t));

			if now.elapsed() > self.duration || self.maxstore < converts.len() {
				if let Err(e) = self.archive(tops.clone()) {
					println!("archive err");
					dbg!(e);
				}

				if let Err(e) = self.init() {
					println!("init err");
					dbg!(e);
				}

				self.cursor_proceed(&converts);

				let mut merged = vec![];
				// topconverts.clone();
				merged.extend(topconverts.iter());
				merged.extend(converts.iter());

				println!("{} perms to save", merged.len());

				self.save(merged);
				now = Instant::now();
				converts = vec![];
			}
		}
	}

	fn get_cursor_log(&self) -> Option<D> {
		if let Ok(f) = File::open(&self.log_path()) {
			let mut f = BufReader::new(f);
			if let Some(last) = f.lines().last()
				&& let Ok(log) = last
			{
				(self.read)(log).1
			} else {
				None
			}
		} else {
			None
		}
	}

	pub fn log<Stream: IntoIterator<Item = D>>(&self, stream: Stream) -> std::io::Result<()> {
		self.init_log()?;

		let mut stream = stream.into_iter().peekable();
		let mut skipped = 0.to_biguint().unwrap();
		loop {
			if let Some(d) = stream.peek()
				// && let Some((_, top)) = tops.last()
				// && !(self.eq)(&d, &top)

				&& let Some(cursor) = self.get_cursor_log()
				&& !(self.eq)(&d, &cursor)
			{
				stream.next();
				skipped += 1.to_biguint().unwrap();
			} else {
				break;
			}
		}

		println!("skipped {}", skipped);

		let mut now = Instant::now();
		let mut results = vec![];
		for d in stream {
			let t = (self.action)(d.clone());
			let v = (self.sort)((d.clone(), t));
			results.push((v, d));

			if now.elapsed() > self.duration || self.maxstore < results.len() {
				match OpenOptions::new()
					.create(true)
					.append(true)
					.open(&self.log_path())
				{
					Ok(mut f) => {
						let mut s: String = Iterator::intersperse(
							results
								.clone()
								.into_iter()
								.map(|(v, d)| (self.readrev)(v, d)),
							"\n".to_string(),
						)
						.collect();

						s += "\n";
						if let Err(e) = f.write_all(s.as_bytes()) {
							dbg!(e);
						} else {
							results = vec![];
							now = Instant::now();
						}
					}

					Err(e) => {
						println!("{}", e);
					}
				}
			}
		}

		Ok(())
	}

	fn log_path(&self) -> String {
		self.path.to_string() + ".log"
	}

	fn init(&self) -> std::io::Result<()> {
		File::create_new(&self.archive_path());
		File::create_new(&self.cursor_path());
		remove_dir_all(&self.path);
		create_dir_all(&self.path)
	}

	fn init_log(&self) -> std::io::Result<()> {
		create_dir_all(&self.path)
	}

	fn archive_path(&self) -> String {
		self.path.to_string() + ".archive"
	}

	fn cursor_path(&self) -> String {
		self.path.to_string() + ".cursor"
	}

	fn get_cursor(&self) -> Option<D> {
		let path = self.cursor_path();
		let s = fs::read_to_string(&path).ok()?;
		(self.read)(s).1
	}

	fn archive(&self, tops: Vec<(Option<V>, D)>) -> std::io::Result<()> {
		let path = &self.archive_path();
		// let mut f = File::options();
		// f.append(true);
		let mut f = OpenOptions::new().create(true).append(true).open(path)?;

		let mut s: String = Iterator::intersperse(
			tops
				.into_iter()
				.map(|(o, d)| o.map(|v| v.to_string()).unwrap_or(String::new()) + " " + &(self.memo)(d)),
			"\n".to_string(),
		)
		.collect();

		// dbg!(&s);
		s += "\n";
		f.write_all(s.as_bytes())?;
		Ok(())
	}

	fn read_tops(&self) -> Vec<(Option<V>, D)> {
		let savepaths: Vec<PathBuf> = Path::new(&self.path)
			.read_dir()
			.map(|dir| {
				dir
					.into_iter()
					.filter_map(|d| d.ok())
					.map(|e| e.path())
					.collect()
			})
			.unwrap_or(vec![]);

		savepaths
			.into_iter()
			.filter_map(|path| {
				let content = fs::read_to_string(path).ok()?;

				let (v, d) = (self.read)(content);

				d.map(|d| (v, d))
			})
			.collect()
	}

	// fn latest_from_archive(&self) -> Vec<(Option<V>, Option<D>)> {
	// 	let o: Option<_> = {
	// 		let archive = fs::read_to_string(self.archive_path()).ok()?;

	// 		let last = archive.lines().last()?;
	// 		(self.read)(last.to_string())
	// 	};

	// 	if let Some(v) { v }
	// }

	pub fn restart<Stream: IntoIterator<Item = D>>(&self, stream: Stream) -> std::io::Result<()> {
		let mut tops = self.read_tops();

		// if let Some(cursor) = self.get_cursor() {
		// 	tops.push((None, cursor));
		// }

		let is_already_processed = true;
		let mut now = Instant::now();
		let mut stream = stream.into_iter().peekable();

		let mut skipped = 0.to_biguint().unwrap();
		let mut amongtop = false;
		loop {
			if let Some(d) = stream.peek()

				// && let Some((_, top)) = tops.last()
				// && !(self.eq)(&d, &top)

				&& let Some(cursor) = self.get_cursor()
				&& !(self.eq)(&d, &cursor)
			{
				stream.next();
				skipped += 1.to_biguint().unwrap();
			} else {
				break;
			}
		}

		println!("skipped {}", skipped);
		self.run(stream, tops);

		Ok(())
	}

	pub fn start<Stream: IntoIterator<Item = D> + Clone>(&self, stream: Stream) {
		if let Err(_) = self.restart(stream.clone()) {
			self.run(stream, vec![]);
		}
	}
}

fn swap_table_permutation() {}

pub fn loop_swap_table_permutation<P, Q, I>(
	shuangpin_table: P,
	swappath: P,
	commit_path: P,
	source_table_paths: I,
	allkeys: &'static str,
	commit_duration: Duration,
	commit_store_max_size: usize,
) where
	P: AsRef<Path> + AsRef<OsStr> + Clone + ToString,
	Q: AsRef<Path> + Copy + Debug,
	I: IntoIterator<Item = Q>,
{
	let swaps = SwapDictChars::new(allkeys, swappath.clone());

	// dbg!(&swaps);

	let fromj = |j: Vec<(String, WithMakeWordSpecifier)>| {
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
	};

	let memo = |cs: Vec<char>| cs.into_iter().collect::<String>();

	let sort = |(d, t): (_, (usize, _))| t.0;

	let dict = get_yongdictwordspells(source_table_paths);

	let action = |d: Vec<char>| {
		let swap = swaps.clone().restruct_swap_dict(&d);
		// restruct_swap_dict(swappath.clone(), &d, &swaps.predefined, &keys);
		word_withspecifiers(swap, &dict, shuangpin_table.clone())
	};

	let write = |perm: Vec<char>, t: (usize, Vec<(String, WithMakeWordSpecifier)>)| {
		// let mut tosort = ts;
		// tosort.sort_by_key(|(nonchain, _)| *nonchain);

		// if let Some((_, spellwords)) = tosort.first() {
		// 	let s: String = perm.into_iter().collect();
		// 	s + "\n" + &spell_words_dict_tostring(fromj(spellwords.to_owned()))
		// } else {
		// 	String::new()
		// }

		let s: String = perm.into_iter().collect();
		t.0.to_string() + " " + &s + "\n" + &spell_words_dict_tostring(fromj(t.1))
	};

	let read = |s: String| {
		s.lines()
			.next()
			.map(|s| {
				let mut it = s.split(" ");
				(
					it.next()
						.map(|permline| usize::from_str_radix(permline, 10).ok())
						.flatten(),
					it.next()
						.map(|vline| vline.chars().filter(|c| c.is_ascii_alphabetic()).collect()),
				)
			})
			.unwrap_or((None, None))
	};

	let eq = |dj: &Vec<char>, dk: &Vec<char>| {
		let j: String = dj.into_iter().collect();
		let k: String = dk.into_iter().collect();
		j == k
	};

	let readrev = |v: usize, d: Vec<char>| v.to_string() + "\t" + &d.into_iter().collect::<String>();

	RepeatAction {
		path: commit_path,
		eq,
		write,
		action,

		read,
		readrev,
		sort,
		memo,
		duration: commit_duration,
		maxstore: commit_store_max_size,
	}
	.start(swaps.clone().permutations())
}

pub fn log_swap_table_permutation<P, Q, I>(
	shuangpin_table: P,
	swappath: P,
	commit_path: P,
	source_table_paths: I,
	allkeys: &'static str,
	commit_duration: Duration,
	commit_store_max_size: usize,
) where
	P: AsRef<Path> + AsRef<OsStr> + Clone + ToString,
	Q: AsRef<Path> + Copy + Debug,
	I: IntoIterator<Item = Q>,
{
	let swaps = SwapDictChars::new(allkeys, swappath.clone());

	// dbg!(&swaps);

	let fromj = |j: Vec<(String, WithMakeWordSpecifier)>| {
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
	};

	let memo = |cs: Vec<char>| cs.into_iter().collect::<String>();

	let sort = |(d, t): (_, (usize, _))| t.0;

	let dict = get_yongdictwordspells(source_table_paths);

	let action = |d: Vec<char>| {
		let swap = swaps.clone().restruct_swap_dict(&d);
		// restruct_swap_dict(swappath.clone(), &d, &swaps.predefined, &keys);
		word_withspecifiers(swap, &dict, shuangpin_table.clone())
	};

	let write = |perm: Vec<char>, t: (usize, Vec<(String, WithMakeWordSpecifier)>)| {
		// let mut tosort = ts;
		// tosort.sort_by_key(|(nonchain, _)| *nonchain);

		// if let Some((_, spellwords)) = tosort.first() {
		// 	let s: String = perm.into_iter().collect();
		// 	s + "\n" + &spell_words_dict_tostring(fromj(spellwords.to_owned()))
		// } else {
		// 	String::new()
		// }

		let s: String = perm.into_iter().collect();
		t.0.to_string() + " " + &s + "\n" + &spell_words_dict_tostring(fromj(t.1))
	};

	let read = |s: String| {
		s.lines()
			.next()
			.map(|s| {
				let mut it = s.split(" ");
				(
					it.next()
						.map(|permline| usize::from_str_radix(permline, 10).ok())
						.flatten(),
					it.next()
						.map(|vline| vline.chars().filter(|c| c.is_ascii_alphabetic()).collect()),
				)
			})
			.unwrap_or((None, None))
	};

	let eq = |dj: &Vec<char>, dk: &Vec<char>| {
		let j: String = dj.into_iter().collect();
		let k: String = dk.into_iter().collect();
		j == k
	};

	let readrev = |v: usize, d: Vec<char>| v.to_string() + "\t" + &d.into_iter().collect::<String>();

	RepeatAction {
		path: commit_path,
		eq,
		write,
		action,

		read,
		readrev,
		sort,
		memo,
		duration: commit_duration,
		maxstore: commit_store_max_size,
	}
	.log(swaps.clone().permutations());
}

pub fn word_withspecifiers<P: AsRef<Path>>(
	swap_dict: HashMap<char, char>,
	dict: &YongDictWordSpells,
	shuangpin_table: P,
) -> (usize, Vec<(String, WithMakeWordSpecifier)>) {
	let translator = WordSpellsEntry::to_cjmain_with_specifier;
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
}

#[test]
fn perm() {
	let max = 9;
	let key: [u8; 32] = [1; 32];
	for _ in 0..10 {
		let perms = Permutor::new_with_slice_key(max, key);
		let s: String = perms.map(|n| n.to_string()).collect();
		dbg!(s);
	}
}
