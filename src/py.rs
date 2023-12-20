use std::collections::HashMap;
use std::fs::{self, File};
use std::path::Path;
use std::{default, slice};

use pinyin::ToPinyin;

use crate::kt::read_lines;

pub fn main() {
	let filepath = "shuang/xiaoque.txt";
	let py = dbg!(['見', '的', '得'].map(|c| get_shuangpin_tone(c, filepath)));
}

pub fn get_shuangpin_tone<P: AsRef<Path>>(c: char, filepath: P) -> Option<ShuangPinTone> {
	// let charforpinyin = dbg!(c);
	if let Some(c) = c.to_pinyin() {
		Some(ShuangPinTone::from(
			&PinyinWithToneNumEnd(c.with_tone_num_end().to_owned()),
			&shuangpin_table_parser(filepath),
		))
	} else {
		None
	}
}

#[derive(Debug, PartialEq, Eq)]
pub enum Tone {
	Tone1,
	Tone2,
	Tone3,
	Tone4,
	Tone0,
}
impl Tone {
	fn fromU32(u: &u32) -> Option<Self> {
		match u {
			0 => Some(Tone::Tone0),
			1 => Some(Tone::Tone1),
			2 => Some(Tone::Tone2),
			3 => Some(Tone::Tone3),
			4 => Some(Tone::Tone4),
			_ => None,
		}
	}
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct PinyinWithToneNumEnd(String);
impl PinyinWithToneNumEnd {
	pub fn split_last(&self) -> (String, String) {
		let (y, n) = self.0.split_at(self.0.len() - 1);
		(y.to_owned(), n.to_owned())
	}

	pub fn split_tone(&self) -> (String, Tone) {
		let (y, n) = self.0.split_at(self.0.len() - 1);
		let defaultvalue = (self.0.clone(), Tone::Tone0);
		if let Some(n) = n.chars().next() {
			if let Some(d) = n.to_digit(10) {
				if let Some(tone) = Tone::fromU32(&d) {
					(y.to_owned(), tone)
				} else {
					defaultvalue
				}
			} else {
				defaultvalue
			}
		} else {
			defaultvalue
		}
	}

	pub fn tone(&self) -> Tone {
		let (y, n) = self.split_last();
		match &n.chars().next() {
			None => Tone::Tone0,
			Some(c) => match c.to_digit(10) {
				None => Tone::Tone0,
				Some(u) => Tone::fromU32(&u).unwrap(),
			},
		}
	}

	fn split_cv(&self) -> (String, String) {
		// let dd = dbg!(self);
		let (sheng, tone) = self.split_tone();
		let mui = &sheng.chars().position(|c| "aeuioü".contains(c));
		// .unwrap() //としたいところだけど 嗯 n2 とかいう拼音もあるらしい
		if let Some(i) = mui {
			let s = sheng.split_at(*i);
			(s.0.to_owned(), s.1.to_owned())
		} else {
			(sheng, "ia".to_owned()) //nのあとに続かないこの音に割り当てさせてもらう
		}
	}
}

#[derive(Debug, PartialEq, Eq)]
pub struct ShuangPinTone {
	zi: char,
	mu: char,
	tone: Tone,
}
impl ShuangPinTone {
	pub fn to_string(&self) -> String {
		let s = self.zi.to_string()
			+ self.mu.to_string().as_str()
			+ match self.tone {
				crate::py::Tone::Tone0 => "a",
				crate::py::Tone::Tone4 => "e",
				crate::py::Tone::Tone3 => "u",
				crate::py::Tone::Tone1 => "i",
				crate::py::Tone::Tone2 => "o",
			};
		s
	}
	fn from(py: &PinyinWithToneNumEnd, tables: &ShuangPinTables) -> Self {
		let dmac = tables;
		let (z, m) = py.split_cv();
		let cs = z.chars().next();
		let zi: char = match z.len() {
			0 => {
				if let Some(ac) = tables.c.get("V") {
					ac.clone()
				} else {
					'v'
				}
			}
			_ => {
				if let Some(ac) = tables.c.get(&z) {
					ac.clone()
				} else {
					cs.expect("z should have at least one length")
				}
			}
		};
		let f =
      // dbg!(
        m.replace("ü", "v")
      // )
      ;
		let mu = if let Some(r) = tables.v.get(&f) {
			r.clone()
		} else {
			f.chars().next().expect("at least one length")
		};
		ShuangPinTone {
			zi,
			mu,
			tone: py.tone(),
		}
	}
}

type ShuangPinTable = HashMap<String, char>;

#[derive(Debug, Clone)]
struct ShuangPinTables {
	c: ShuangPinTable,
	v: ShuangPinTable,
}
impl ShuangPinTables {
	fn new() -> Self {
		ShuangPinTables {
			c: HashMap::new(),
			v: HashMap::new(),
		}
	}
}
fn shuangpin_table_parser<P: AsRef<Path>>(filepath: P) -> ShuangPinTables {
	let lines = read_lines(filepath).expect("existing filepath");
	let mut tables = ShuangPinTables::new();
	let res = lines.into_iter().fold(&mut tables, |s, d| {
		if let Ok(o) = d {
			let mut w = o.trim().split_whitespace();
			let key = w.next();
			let yin = w.next();
			if let (Some(key), Some(yin)) = (key, yin) {
				if key.len() == 1 {
					if let Some(key) = key.chars().next() {
						if key.is_ascii_alphabetic() {
							let vs = ['a', 'e', 'u', 'i', 'o', 'v'];
							if yin.starts_with(vs) {
								s.v.insert(yin.to_owned(), key);
							} else {
								s.c.insert(yin.to_owned(), key);
							}
							s
						} else {
							s
						}
					} else {
						s
					}
				} else {
					s
				}
			} else {
				s
			}
		} else {
			s
		}
	});
	res.clone()
}
