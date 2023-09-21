use std::collections::HashMap;

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
	fn from_aeuio(e: char)-> Option<Self> {
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
	fn chain_as(&self, s: Self)-> char {
		let lines = read_lines("spell/xxyx.txt").expect("correct xxyx spell filepath");
		let di = lines.filter_map(|f| {
			if let Ok(a) = f {
				read_line_alpha_entry(a)
			} else {None}
		})
			.map(|a| (a.value, a.key))
			.collect::<HashMap<_,_>>();
		let di = d!(di);
		let key = self.to_aeuio().to_string() + &s.to_aeuio().to_string();
		*di.get(&key).expect(&format!("
		key={},
		complete xxyx spell rule.", &key))
	}
}