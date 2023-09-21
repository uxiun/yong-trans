use std::{collections::HashMap, hash::Hash, cmp::Ordering};

use nom::Parser;


pub fn union_hashmap<L>(
	s: &mut HashMap<<L as IntoIterator>::Item,L>, 
	d: HashMap<<L as IntoIterator>::Item,L>
)-> &mut HashMap<<L as IntoIterator>::Item,L> 
where 
	<L as IntoIterator>::Item: Eq + PartialEq + Hash,
	L: IntoIterator + Extend<<L as IntoIterator>::Item>,
{
	for (k,v) in d.into_iter() {
		if let Some(sv) = s.get_mut(&k) {
			sv.extend(v);
		} else {
			s.insert(k, v);
		}
	}
	s
}


pub fn unions_hashmap<L,K>(
	base: &mut HashMap<K, L>, 
	hs: Vec<HashMap<K,L>>)
-> &mut HashMap<K,L> 
where
	K: Eq + PartialEq + Hash,
	L: IntoIterator + Extend<<L as IntoIterator>::Item>,
{
	for h in hs.into_iter() {
		for (k,v) in h.into_iter() {
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

pub fn cmp_by_len_default(s: &String, d: &String)-> Ordering 
{
	
	match s.chars().count() as i32 - d.chars().count() as i32 {
		0 => s.cmp(&d),
		x => if x < 0 {Ordering::Less} else {Ordering::Greater}
	}
}