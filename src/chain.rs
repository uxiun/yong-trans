use std::{
	collections::HashMap,
	fmt::{Debug, Display},
	fs::{self, File, OpenOptions},
	io::{self, BufWriter, Write},
	iter::zip,
	ops::Not,
	path::Path,
};

use itertools::Itertools;
use permutation_iterator::Permutor;

use crate::{
	kt::{get_yongdictwordspells, to_swap_key_dict},
	out::{spell_words_dict_tostring, WithMakeWordSpecifier},
	repeat::word_withspecifiers_stringdict,
	spell::{specifier_to_wordspells, swap_table_literal_tostring, ScoredPerm},
	util::{concat_lines, hashmap_flip, union_hashmap, strip_dir_from_path, get_filename},
};

pub fn loopperm<I, Q>(
	i: usize,
	j: usize,
	yong_source_tables: I
) -> io::Result<()>
where
	Q: AsRef<Path> + Debug + Display + Clone,

	I: IntoIterator<Item = Q> + Clone,
{
	let yongdict = get_yongdictwordspells(yong_source_tables.clone());


	let path = "spell/stat_for_swap";
	let content = fs::read_to_string(path)?;
	let stats = get_swap_stats(&content)?;

	let predefined: HashMap<char, char> = HashMap::from_iter([('q', 'p'),('a', 'z'), ('b', 'x'), ('p', 'q')]);
	let allchars = "qwertyuiopasdfghjklzxcbmnv";
	let keys = allchars
		.chars()
		.filter(|c| !predefined.contains_key(c))
		.collect_vec();
	let vars = allchars
		.chars()
		.filter(|c| predefined.values().contains(c).not())
		.collect_vec();

	leveled_perm(
		i,
		j,
		&keys,
		&vars,
		&stats,
		&predefined,
		yong_source_tables,
		// save_path,
	)
}

fn log<P>(save_path: P, score: usize, keytocjchar: &HashMap<char, char>)
where
	P: Display,
{
	let save_path = format!("{}.log", save_path,);
	let kch = keytocjchar_tostring(keytocjchar);
}

fn leveled_perm<'a, I, Q>(
	i: usize,
	j: usize,
	keys: &Vec<char>,
	vars: &Vec<char>,
	stats: &HashMap<&'a str, usize>,
	predefined: &HashMap<char, char>,
	yong_source_tables: I,
	// save_path: P,
	// yongdict:&HashMap<String, Vec<crate::spell::SpecifySpelling>>
) -> io::Result<()>
where
	I: IntoIterator<Item = Q> + Clone,
	Q: AsRef<Path> + Debug + Clone + Display,
	// P: AsRef<Path> + Display,
{
	let yongdict = &get_yongdictwordspells(yong_source_tables.clone());
	let mut yongpaths = yong_source_tables.clone().into_iter().map(|d| get_filename(d) ).collect_vec();
	yongpaths.sort();

	let yongpath_string =yongpaths.join("_");
	let save_path = format!( ".auto/{}", yongpath_string);

	let logfile_path = format!("{}.log", save_path);
	let msg = format!("could not open logfile: {}", logfile_path);
	let logfile = OpenOptions::new()
		.create(true)
		.append(true)
		.open(&logfile_path)
		.expect(&msg);
	let mut logwriter = BufWriter::new(logfile);
	loop {
	let mut realscoreds = (0..i)
		.filter_map(|_| {
			let mut scored = (0..j)
				.map(|_| {
					let key_randvar = permloop(keys, vars);
					let keytochar = HashMap::from_iter(
						[predefined.into_iter(), key_randvar.iter()]
							.into_iter()
							.flatten()
							.map(|(key, cha)| (*key, *cha)),
					);
					let chartokey = HashMap::from_iter(keytochar.iter().map(|(k, v)| (*v, *k)));

					let minuscore = eval(stats, &chartokey);
					(minuscore, keytochar)
				})
				.collect_vec();

			scored.sort_by_key(|(score, _)| *score);
			let (score, keytochar) = scored.first()?;
			let swap_dict = charchar_to_swap_dict(keytochar);

			let (score, w) = word_withspecifiers_stringdict(&swap_dict, yongdict, "shuang/xiaoque.txt");

			let log = Log {
				keytochar: keytochar.to_owned(),
				score,
				yongpaths: yongpaths.clone(),
			};

			// let line = format!("{} {} {}", score, keytocjchar_tostring(keytochar), yongpath_string);
			let line = log.to_string();
			println!("{}", line);
			logwriter.write_all((line+"\n") .as_bytes());

			Some((log, w))
		})
		.collect_vec();

	logwriter.flush();

	realscoreds.sort_by_key(|(log, _)| log.score);

	let (log, dict) = realscoreds.first().expect("ensure that write loop i > 0");

	println!("{}", log.score);
	prettyprint_keytocjchar(&log.keytochar);

	}
}

pub fn restruct_keytocjchar_and_write(
	logline: &str,
) -> io::Result<()>
{
	match Log::try_from(logline) {
		Ok(log) => {

			let mut paths =log.yongpaths;
			paths.sort();
			let save_path = format!(
				"{}-keytochar:{}",
				paths.join("_"),
				keytocjchar_tostring(&log.keytochar)
			);

			let yongdict = get_yongdictwordspells(paths);

			let swap_dict = &charchar_to_swap_dict(&log.keytochar);

			let (score, dict) = word_withspecifiers_stringdict(swap_dict, &yongdict, "shuang/xiaoque.txt");
			write_extra_scored_withspecifier(&keytocjchar_to_file_format(&log.keytochar), dict, save_path)
		},
		Err(e) => {
			println!("{}", e);
			Ok(())
		}
	}


}

pub fn keytocjchar_tostring(keytocjchar: &HashMap<char, char>) -> String {
	let mut kvs = keytocjchar
		.into_iter()
		.map(|(k, v)| format!("{k}{v}"))
		.collect_vec();
	kvs.sort();
	kvs.join("_")
}

pub fn keytocjchar_to_file_format(keytocjchar: &HashMap<char, char>) -> String {
	concat_lines(keytocjchar.into_iter().map(|(k, v)| format!("{k} {v}"))) + "\n"
}

pub fn file_to_keytocjchar<P: AsRef<Path>>(path: P) -> HashMap<char, char> {
	HashMap::from_iter(fs::read_to_string(&path).into_iter().flat_map(|s| {
		s.lines().filter_map(|line| {
			line
				.trim()
				.split_whitespace()
				.filter_map(|s| s.chars().next())
				.collect_tuple()
		}).collect::<Vec<_>>()
	}))
}

struct Log {
	score: usize,
	keytochar: HashMap<char,char>,
	yongpaths: Vec<String>
}



impl ToString for Log {
	fn to_string(&self) -> String {
		let y = self.yongpaths.join(" ");
		let k = keytocjchar_tostring(&self.keytochar);
		format!("{} {k} {y}", self.score)
	}
}

impl TryFrom<&str> for Log {
	type Error = &'static str;
fn try_from(value: &str) -> Result<Self, Self::Error> {

	let mut i = value.trim().split_whitespace();
	let scorestr = i.next().ok_or("no score")?;
	let score = usize::from_str_radix(scorestr, 10).ok().ok_or("could not parse to usize")?;
	let kcstr = i.next().ok_or("no underline_separeted text for keytochar")?;
	let keytochar = string_to_keytocjchar(kcstr);
	Ok(


	Log {
		score,
		keytochar,
		yongpaths: i.map(|s| s.to_string()).collect()
	}
	)
}
}


pub fn string_to_keytocjchar(underline_separeted: &str) -> HashMap<char,char> {
	HashMap::from_iter(
		underline_separeted.trim().split("_")
		.filter_map(|s| s.chars().collect_tuple())

	)
}

pub fn prettyprint_keytocjchar(keytocjchar: &HashMap<char, char>) {
	let s = swap_table_literal_tostring(keytocjchar);
	println!("{}", s);
}

pub fn write_extra_scored_withspecifier<P: AsRef<Path>>(
	extra: &str,
	dict: Vec<(String, WithMakeWordSpecifier)>,
	save_path: P,
) -> io::Result<()> {
	let s = spell_words_dict_tostring(specifier_to_wordspells(dict));

	let f = OpenOptions::new()
		.create(true)
		.truncate(true)
		.write(true)
		.open(save_path)?;
	let mut f = BufWriter::new(f);
	f.write_all((extra.to_string() + "\n" + &s).as_bytes())?;
	f.flush()
}

fn charchar_to_swap_dict(literal_swap_dict: &HashMap<char, char>) -> HashMap<String, String> {
	HashMap::from_iter(
		literal_swap_dict
			.iter()
			.map(|(k, v)| (v.to_string(), k.to_string())),
	)
}

fn get_swap_stats<'a>(content: &'a str) -> io::Result<HashMap<&'a str, usize>> {
	Ok(HashMap::from_iter(content.lines().flat_map(|line| {
		line
			.trim()
			.split_whitespace()
			.chunks(2)
			.into_iter()
			.filter_map(|chunk| {
				let (chain, count) = chunk.collect_tuple()?;
				let u = usize::from_str_radix(count, 10).ok()?;
				Some((chain, u))
			})
			.collect_vec()

		// let chainscores = vec![];
	})))
}

fn eval<'a>(
	stats: &HashMap<&'a str, usize>,
	chartokey: &HashMap<char, char>,
	// kv: &HashMap<char, char>,
) -> usize {
	// let chartokey: HashMap<char, char> = HashMap::from_iter( hashmap_flip(&di).into_iter().map(|(k,v) | (**k,**v)));

	stats.into_iter().fold(0, |minuscore, (chain, count)| {
		let isfluent = chain
			.chars()
			.filter_map(|c| chartokey.get(&c))
			.collect_tuple()
			.map(|(j, k)| is_fluent(j, k))
			.unwrap_or(false);

		minuscore + if isfluent { 0 } else { *count }
	})
}

fn is_fluent(j: &char, k: &char) -> bool {
	let s = "swz";
	let d = "dex";
	let f = "frtgcb";
	let j = "jhuym";
	let k = "kin";
	let l = "lov";
	let sdf = [s, d, f];
	let jkl = [j, k, l];

	![sdf, jkl]
		.into_iter()
		.any(|g| g.into_iter().any(|cs| cs.contains(j) && cs.contains(k)))
		|| [(sdf, j), (jkl, k)]
			.into_iter()
			.all(|(g, c)| g.into_iter().any(|cs| cs.contains(c)))
		|| [(sdf, k), (jkl, j)]
			.into_iter()
			.all(|(g, c)| g.into_iter().any(|cs| cs.contains(c)))
}

fn permloop(keys: &Vec<char>, vars: &Vec<char>) -> HashMap<char, char> {
	let perm = Permutor::new(vars.len() as u64);
	let randvars = perm.filter_map(|i| vars.get(i as usize)).collect_vec();

	if randvars.len() != keys.len() {
		panic!("different length");
	}

	let kvs = zip(keys, randvars);
	HashMap::from_iter(kvs.into_iter().map(|(k, v)| (*k, *v)))
}
