use std::{collections::HashMap, io::BufWriter, fs::File};

use itertools::Itertools;

use crate::{kt::{read_lines, YongDictWordSpells}, parser::read_line_alpha_entry, d};

pub fn main(){
	let j = d!([
		BihuaXxyx::Shu.chain_as(BihuaXxyx::Zhe)
	]);
}

#[derive(Debug,Clone, Copy, PartialEq, Eq)]
pub enum YongSpelling {
	Xxyx,
	Cangjie,
	Free
}

pub enum WordSpellsLineFormat {
	AddShorter,
	Free
}

#[derive(Debug, Clone)]
pub struct SpecifySpelling {
	pub spelling: YongSpelling,
	pub spell: String,
}
impl SpecifySpelling {
}


#[derive(Debug, Clone, Copy)]
pub enum BihuaXxyx {
	Shu, //竖 I 巾
	Dian, //点 、广
	Zhe, //折 く 录
	Heng, //横 一
	Pie, //撇 白
}
impl BihuaXxyx {
	pub fn from_aeuio(e: char)-> Option<Self> {
		match e {
			'a' => Some(Self::Shu),
			'e' => Some(Self::Dian),
			'u' => Some(Self::Zhe),
			'i' => Some(Self::Heng),
			'o' => Some(Self::Pie),
			_ => None
		}
	}
	fn to_aeuio(&self)-> char {
		match self {
			Self::Shu => 'a',
			Self::Dian => 'e',
			Self::Zhe => 'u',
			Self::Heng => 'i',
			Self::Pie => 'o',
		}
	}
	pub fn chain_as(&self, s: Self)-> char {
		let lines = read_lines("spell/xxyx.txt").expect("correct xxyx spell filepath");
		let di = lines.filter_map(|f| {
			if let Ok(a) = f {
				read_line_alpha_entry(a)
			} else {None}
		})
			.map(|a| (a.value, a.key))
			.collect::<HashMap<_,_>>();
		// let di = d!(di);
		let key = self.to_aeuio().to_string() + &s.to_aeuio().to_string();
		*di.get(&key).expect(&format!("
		key={},
		complete xxyx spell rule.", &key))
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

fn code_py(spellen: u32, nth: u32)
-> Vec<String>
{
	(0..3).map(|i| 
		format!("+p{}{}", nth, spellen-i)
	).rev()
	.collect()
}

fn pformat(charnth: u32, keynth: u32)
-> String
{
	format!("p{}{}", charnth, keynth)
}

fn plusformat(charnth: u32, keynth: u32)
-> String
{
	format!("+p{}{}", charnth, keynth)
}



fn code_head(wordlen: u32)
-> String
{
	match wordlen {
		2 => {
			"p11+p12+p21+p22+p23+p13".to_string()
			// let h: String = (1..3).map(|i| {
			// 	(1..3).map(|j| pformat(i, j)
			// 	).collect::<String>()
			// }).collect();

		}
		_ => {
			"".to_string()
		}
	}
}

fn code_mid(spellen: u32, charnth: u32)
-> String 
{
	let i = spellen - 3;
	if spellen < 9 {
		"".to_owned()
	} else {
		(0..spellen-8).map(|f| {
			plusformat(charnth, f+4)
		}).collect()
	}
}