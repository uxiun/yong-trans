use std::{
	cmp::Ordering,
	collections::HashMap,
	fmt::Debug,
	hash::Hash,
	time::{Duration, Instant}, path::Path, fs::{File, OpenOptions}, io::{Read, Write},
};

use chrono::{DateTime, Local};
use humantime::format_duration;
use itertools::Itertools;
use nom::Parser;
use num_bigint::{BigUint, ToBigUint};

pub fn hashmap_flip_flatten<K, L>(
	// base: &mut HashMap<<L as IntoIterator>::Item, K>,
	h: HashMap<K, L>,
) -> HashMap<<L as IntoIterator>::Item, K>
where
	L: IntoIterator,
	<L as IntoIterator>::Item: Hash + Eq + PartialEq + Clone,
	K: Clone,
{
	let mut base = HashMap::new();
	for (k, v) in h.into_iter() {
		for i in v.into_iter() {
			base.insert(i, k.clone());
		}
	}
	base.clone()
}

pub fn union_hashmap<L>(
	s: &mut HashMap<<L as IntoIterator>::Item, L>,
	d: HashMap<<L as IntoIterator>::Item, L>,
) -> &mut HashMap<<L as IntoIterator>::Item, L>
where
	<L as IntoIterator>::Item: Eq + PartialEq + Hash,
	L: IntoIterator + Extend<<L as IntoIterator>::Item>,
{
	for (k, v) in d.into_iter() {
		if let Some(sv) = s.get_mut(&k) {
			sv.extend(v);
		} else {
			s.insert(k, v);
		}
	}
	s
}

pub fn unions_hashmap<L, K>(base: &mut HashMap<K, L>, hs: Vec<HashMap<K, L>>) -> &mut HashMap<K, L>
where
	K: Eq + PartialEq + Hash,
	L: IntoIterator + Extend<<L as IntoIterator>::Item>,
{
	for h in hs.into_iter() {
		for (k, v) in h.into_iter() {
			if let Some(sv) = base.get_mut(&k) {
				sv.extend(v);
			// for item in v.into_iter() {
			// 	sv.push(item);
			// }
			} else {
				base.insert(k, v);
			}
		}
	}
	base
}

pub fn groupBy_run<K, V>(i: &[(K, V)]) -> HashMap<&K, Vec<&V>>
where
	K: Eq + Hash,
{
	let mut h = HashMap::new();
	for (key, value) in i.into_iter() {
		h.entry(key)
			.and_modify(|e: &mut Vec<&V>| e.push(value))
			.or_insert(vec![value]);
	}

	h
}

pub fn sort_groupby<T, V, I>(it: I) -> Vec<(V, Vec<T>)>
where
	I: IntoIterator<Item = (V, T)>,
	V: Hash + Ord + Clone,
	T: Clone,
{
	let mut h: HashMap<V, Vec<T>> = HashMap::new();
	for (v, t) in it {
		h.entry(v)
			.and_modify(|ts| ts.push(t.clone()))
			.or_insert(vec![t]);
	}

	let mut ve: Vec<(V, Vec<T>)> = h.into_iter().collect();
	ve.sort_by_key(|(v, _)| v.clone());

	ve
}

#[test]
fn sg() {
	let v = vec![
		(5, "hello"),
		(2, "ai"),
		(3, "shy"),
		(2, "hi"),
	];
	
	let sorted = sort_groupby(v.clone());
	
	let s: String = v.into_iter().map(|(i, s)| s.to_string()).collect();
	
	dbg!(s);
}

pub fn cmp_by_len_default(s: &String, d: &String) -> Ordering {
	match s.chars().count() as i32 - d.chars().count() as i32 {
		0 => s.cmp(&d),
		x => {
			if x < 0 {
				Ordering::Less
			} else {
				Ordering::Greater
			}
		}
	}
}

fn insert_at_beginning<P: AsRef<Path> + Clone>(file_path: P, content_to_insert: String) -> std::io::Result<()> {
	// // Open the file in read mode to read existing content
	// let mut file = 
	
	// File::open(file_path)?;
	// Open the file in write mode to overwrite with new content
	let mut file = OpenOptions::new()
		.write(true)
		.truncate(true)
		.open(&file_path)?;
	
	// Read existing content
	let mut existing_content = Vec::new();
	file.read_to_end(&mut existing_content)?;
	
	
	// Write new content at the beginning
	file.write_all(content_to_insert.as_bytes())?;

	// Write back the existing content after the new content
	file.write_all(&existing_content)?;

	Ok(())
}

pub fn now_string() -> String {
	let now = Local::now();
	now.to_rfc3339_opts(chrono::SecondsFormat::AutoSi, false)
}

#[test]
fn nowst() {
	dbg!(now_string());
}

pub fn with_elapsed_time<'a, F, R>(message: &'a str, f: F) -> R
where
	F: Fn() -> R,
{
	let now = Instant::now();
	let r = f();
	let elapsed = now.elapsed();
	println!("elapsed time: {} -- {}", format_duration(elapsed), message);
	r
}

pub fn combination_size(n: usize, k: usize) -> usize {
	if n < k {
		0
	} else {
		combination_size_inner(n, k)
	}
}

fn combination_size_inner(n: usize, k: usize) -> usize {
	if k == 1 {
		n
	} else {
		n * combination_size_inner(n - 1, k - 1) / k
	}
}

pub fn permutation_size(n: usize, k: usize) -> BigUint {
	if n < k {
		0u32.into()
	} else {
		permutation_size_inner(0, n, k)
	}
}

fn permutation_size_inner(i: usize, n: usize, k: usize) -> BigUint {
	if i < k {
		n * permutation_size_inner(i + 1, n - 1, k)
	} else {
		1u32.into()
	}
}

#[test]
fn permsize() {
	let keys = "qwertyuiopasdfghjklzxcbmnv";
	// let i = permutation_size(, 2);
	// assert_eq!("0123456789".chars().permutations(3).count(), permutation_size(10, 3));

	let year = (60 * 60 * 24 * 365).to_biguint().unwrap();
	println!("{}", usize::MAX);

	let i = permutation_size(keys.len(), keys.len());
	dbg!(&i);
	dbg!(&year);
	dbg!(i / year);
	// dbg!(format_duration(Duration::from))

	let n = 8;
	let k = 2;
	let j = 3;

	let c: BigUint = 2u32.pow((j + k) as u32).into();
	assert_eq!(
		combination_size(n, j + k).to_biguint().unwrap()
			* permutation_size(j, j)
			* permutation_size(k, k)
			* combination_size(j + k, j),
		permutation_size(n, k + j)
	)
}

#[test]
fn calc_max_item_number() {
	let keys = "qwertyuiopasdfghjklzxcbmnv";
	let mut i = keys.len();
	loop {
		let s = permutation_size(i, i);
		dbg!(&s);
		if s < usize::MAX.into() {
			dbg!(usize::MAX);
			break;
		}
		i -= 1;
	}

	dbg!(i);
}

pub fn process_permutation_separately<Ip, P, Fp, Fs, V, T>(
	perm_set: Ip,
	process_perm: Fp,
	sort_key: Fs,
) -> Vec<(Vec<P>, T)>
where
	Ip: IntoIterator<Item = P>,
	Fp: Fn(Vec<P>) -> T,
	Fs: Fn(T) -> V,
	V: Ord,
	P: Clone + Eq + Debug,
	T: Clone,
{
	// process_permutation_separately_inner(perm_set, process_perm, sort_key, vec![])

	let (items, permitem_count) =
		perm_set
			.into_iter()
			.fold((vec![], 0), |(mut set, mut count), item| {
				set.push(item);
				(set, count + 1)
			});

	let mut max_item_number = permitem_count;
	loop {
		let size = permutation_size(max_item_number, max_item_number);
		if size < usize::MAX.into() {
			break;
		} else {
			max_item_number -= 1;
		}
	}

	vec![]
}

// fn process_permutation_separately_inner<Ip, P, Fp, Fs, V, T>(
// 	perm_set: Ip,
// 	process_perm: Fp,
// 	sort_key: Fs,
// 	// banned_indexes: Vec<usize>
// ) -> Vec<(Vec<P>, T)>
// where
// 	Ip: IntoIterator<Item = P>,
// 	Fp: Fn(Vec<P>) -> T,
// 	Fs: Fn(T) -> V,
// 	V: Ord,
// 	P: Clone + Eq + Debug,
// 	T: Clone,
// {

// 	let mut banned = vec![];

// 	let (items, permitem_count) =
// 		perm_set
// 			.into_iter()
// 			.filter(|d| !banned.contains(d))
// 			.fold((vec![], 0), |(mut set, mut count), item| {
// 				set.push(item);
// 				(set, count + 1)
// 			});

// 	println!("{:?}", items);

// 	let size = permutation_size(permitem_count, permitem_count);
// 	if size > u32::MAX.into() {

// 		for (j,item ) in items.into_iter().enumerate() {
// 			banned.push(j);

// 		}

// 		let pairs = items
// 			.clone()
// 			.into_iter()
// 			.enumerate()
// 			.map(|(j,item)| {
// 				let mut banned = banned_indexes.clone();
// 				banned.push(j);
// 				let child =
// 					process_permutation_separately_inner(items.clone(), |x| process_perm(x), |x| sort_key(x), banned);
// 				let mut inserted = (0..child.len())
// 					.flat_map(|i| {
// 						let mut v = child.clone();
// 						let mut perm = items.clone().into_iter()
// 							.enumerate()
// 							.filter_map(|(j, item)| if banned_indexes.contains(&j) { None } else { Some(item)})
// 							.collect_vec();
// 						perm.insert(i, item.clone());
// 						v.push((perm.clone(), process_perm(perm)));
// 						v
// 					})
// 					.collect::<Vec<_>>();
// 				inserted.sort_by_key(|(_, t)| sort_key(t.clone()));
// 				inserted[0].clone()
// 			})
// 			.collect::<Vec<_>>();
// 		pairs
// 	} else {
// 		let mut tosort: Vec<_> = items
// 			.into_iter()
// 			.enumerate()
// 			.filter_map(|(j, item)| if banned_indexes.contains(&j) { None } else { Some(item)})
// 			.permutations(permitem_count)
// 			.map(|perm| (perm.clone(), process_perm(perm)))
// 			.collect();
// 		tosort.sort_by_key(|(_, t)| sort_key(t.clone()));
// 		tosort
// 	}
// }
