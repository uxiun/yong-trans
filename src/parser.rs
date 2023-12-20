use crate::out::WordSpellsEntry;
use nom::{
	bytes::complete::{is_not, tag, take_while1},
	character::{
		complete::{alpha1, anychar, char as onechar, multispace0, multispace1, one_of},
		is_alphabetic, is_space,
	},
	error::{Error, ParseError},
	multi::{fold_many1, many_m_n, separated_list1},
	sequence::{delimited, preceded, separated_pair, terminated, tuple},
	AsChar, Err, IResult, InputLength, InputTakeAtPosition, Parser,
};
use std::{
	collections::HashMap,
	str::{self, Chars},
};

pub fn main() {
	parsetameshi();
}

fn parsetameshi() {
	//OK
	let x = dbg!([
		// xxyx_spell("h 好 word1  词 ".as_bytes()),
		read_line_custom_add("可 h rh".to_owned()),
		read_line_custom_add("vlai invalid num  ".to_owned()),
		read_line_custom_add("语 sj k dj;".to_owned()),
		read_line_custom_add("izai 语 无".to_owned()),
		// xxyx_spell(b"x"),
		// xxyx_spell(b"xj"),
		// xxyx_spell(b" x"),
	]);
}

#[derive(Debug)]
pub struct AlphaEntry {
	pub key: char,
	pub value: String,
}

pub fn read_line_alpha_entry(line: String) -> Option<AlphaEntry> {
	if let Ok((value, (key, space))) = tuple((alpha_char, spaces))(line.as_bytes()) {
		Some(AlphaEntry {
			key,
			value: String::from_utf8(value.to_vec()).expect("valid utf8 bytes"),
		})
	} else {
		None
	}
}

fn alpha_char(i: &[u8]) -> IResult<&[u8], char> {
	one_of("qwertyuiopasdfghjklzxcvbmn")(i)
}

#[derive(Debug, Clone)]
pub struct YongDef {
	pub spell: String,
	pub words: Vec<String>,
}

pub fn read_line_yong_def(line: String) -> Option<YongDef> {
	let t = tuple((yong_spell, spaces, separated_list_by_space))(line.as_bytes());
	if let Ok(t) = t {
		Some(YongDef {
			spell: String::from_utf8(t.1 .0).expect("valid utf8 bytes"),
			words: t
				.1
				 .2
				.into_iter()
				.map(|b| String::from_utf8(b.to_vec()).expect("valid utf8 bytes"))
				.collect(),
		})
	} else {
		None
	}
}

pub fn read_line_xxyx_def(line: String) -> Option<YongDef> {
	let t = tuple((xxyx_spell, spaces, separated_list_by_space))(line.as_bytes());
	if let Ok(t) = t {
		Some(YongDef {
			spell: String::from_utf8(t.1 .0).expect("valid utf8 bytes"),
			words: t
				.1
				 .2
				.into_iter()
				.map(|b| String::from_utf8(b.to_vec()).expect("valid utf8 bytes"))
				.collect(),
		})
	} else {
		// let k = dbg!(&t);
		None
	}
}

pub fn read_line_cangjie_def(line: String) -> Option<YongDef> {
	let t = tuple((alpha1, spaces, separated_list_by_space))(line.as_bytes());
	if let Ok(t) = t {
		Some(YongDef {
			spell: String::from_utf8(t.1 .0.to_vec()).expect("valid utf8 bytes"),
			words: t
				.1
				 .2
				.into_iter()
				.map(|b| String::from_utf8(b.to_vec()).expect("valid utf8 bytes"))
				.collect(),
		})
	} else {
		None
	}
}

pub type StringStringsEntry = (String, Vec<String>);
pub type StringStringsDict = HashMap<String, Vec<String>>;

pub fn read_line_custom_add(line: String) -> Option<StringStringsEntry> {
	if let Ok((rem, result)) = tuple((not_space, spaces, separated_list_by_space))(line.as_bytes()) {
		let mut spells: Vec<String> = result
			.2
			.into_iter()
			.map(|e| String::from_utf8(e.to_vec()).unwrap())
			.filter(|s| s.chars().all(|c| c.is_ascii_alphabetic()))
			.collect();
		let hanzi = String::from_utf8(result.0.to_vec()).unwrap();
		// if spells.len() > 1 &&
		if hanzi.chars().count() == 1 {
			spells.pop();
			Some((String::from_utf8(result.0.to_vec()).unwrap(), spells))
		} else {
			None
		}
	} else {
		None
	}
}

pub fn read_line_swap_key(line: String) -> Option<StringStringsEntry> {
	let (rem, ma) = tuple((not_space, spaces, separated_list_by_space))(line.as_bytes()).ok()?;
	let key = String::from_utf8(ma.0.to_vec()).ok()?;
	if key.chars().count() > 1 {
		None
	} else {
		let target_key = key.chars().next()?.to_string();
		let range_keys = ma
			.2
			.into_iter()
			.filter_map(|s| String::from_utf8(s.to_vec()).ok())
			.filter(|s| {
				s.chars().count() == 1 && {
					if let Some(k) = s.chars().next() {
						k.is_ascii_alphabetic()
					} else {
						false
					}
				}
			})
			.collect();
		Some((target_key, range_keys))
	}
}

pub fn filter_vowels(cs: Chars) -> String {
	cs.filter(|c| is_vowel_char(*c)).collect()
}

fn yong_spell(i: &[u8]) -> IResult<&[u8], Vec<u8>> {
	fold_many1(
		alpha
			.or(tag("+"))
			.or(tag("."))
			.or(tag(","))
			.or(tag(";"))
			.or(tag("^"))
			.or(tag("'"))
			.or(tag("-")),
		Vec::new,
		|mut acc: Vec<_>, d| {
			acc.extend_from_slice(d);
			acc
		},
	)(i)
}

fn is_vowel(u: u8) -> bool {
	if let Ok(s) = &String::from_utf8(vec![u]) {
		"aeuio".contains(s)
	} else {
		false
	}
}
fn is_vowel_char(c: char) -> bool {
	"aeuio".contains(c)
}
fn is_consonant_char(c: char) -> bool {
	c.is_ascii_alphabetic() && !is_vowel_char(c)
}

fn xxyx_spell(i: &[u8]) -> IResult<&[u8], Vec<u8>> {
	let (rem, mut js) = alpha1(i)?;
	match js.len() {
		l => {
			let c = js[0].as_char();
			if is_consonant_char(c) {
				match l {
					1 => Ok((rem, js.to_vec())),
					// , 2=> Ok((rem, js.to_vec())) ,
					l => {
						if js[1].as_char() == 'z' {
							Err(nom::Err::Error(Error {
								input: i,
								code: nom::error::ErrorKind::IsNot,
							}))
						} else {
							let (left, shouldbe_vowels) = js.split_at(2);
							if shouldbe_vowels.iter().all(|d| is_vowel_char(d.as_char())) {
								Ok((rem, js.to_vec()))
							} else {
								// let msg = dbg!("should be vowels");
								Err(nom::Err::Error(Error {
									input: i,
									code: nom::error::ErrorKind::IsNot,
								}))
							}
						}
					}
				}
			} else {
				Err(nom::Err::Error(Error {
					input: i,
					code: nom::error::ErrorKind::IsNot,
				}))
			}
		}
	}
}
fn xxyx_spellold(i: &[u8]) -> IResult<&[u8], Vec<u8>> {
	let (rem, first) = anychar(i)?;
	if is_consonant_char(first) {
		if let Ok((rem, second)) = anychar::<_, ()>(rem) {
			if second == 'z' {
				println!("second char should not be z");
				Err(nom::Err::Error(Error {
					code: nom::error::ErrorKind::Char,
					input: rem,
				}))
			} else if second.is_ascii_alphabetic() {
				let f = first.to_string();
				let s = second.to_string();
				let k = f + &s;
				let fs = k.as_bytes();
				if let Ok((rem, m)) = take_while1::<_, _, ()>(is_vowel)(rem) {
					Ok((rem, [fs, m].concat()))
				} else {
					Ok((rem, fs.to_vec()))
				}
			} else {
				Ok((rem, first.to_string().as_bytes().to_vec()))
			}
		} else {
			println!("one spell? ok");
			Ok((rem, first.to_string().as_bytes().to_vec()))
		}
	} else {
		Err(nom::Err::Error(Error {
			code: nom::error::ErrorKind::Char,
			input: i,
		}))
	}
}

fn alpha(s: &[u8]) -> IResult<&[u8], &[u8]> {
	take_while1(is_alphabetic)(s)
}

fn separated_list_by_space(i: &[u8]) -> IResult<&[u8], Vec<&[u8]>> {
	separated_list1(multispace1, not_space)(i)
}

fn spaces(i: &[u8]) -> IResult<&[u8], &[u8]> {
	take_while1(is_space)(i)
}

fn not_space(s: &[u8]) -> IResult<&[u8], &[u8]> {
	is_not(" \t\r\n")(s)
}

/// A combinator that takes a parser `inner` and produces a parser that also consumes both leading and
/// trailing whitespace, returning the output of `inner`.
fn trim<'a, I: 'a, F: 'a, O, E: ParseError<&'a I>>(
	inner: F,
) -> impl FnMut(&'a I) -> IResult<&'a I, O, E>
where
	F: Fn(&'a I) -> IResult<&'a I, O, E>,
	&'a I: InputLength + InputTakeAtPosition,
	<&'a I as InputTakeAtPosition>::Item: AsChar + Clone,
{
	delimited(multispace0, inner, multispace0)
}
fn trim_end<'a, I: 'a, F: 'a, O, E: ParseError<&'a I>>(
	inner: F,
) -> impl FnMut(&'a I) -> IResult<&'a I, O, E>
where
	F: Fn(&'a I) -> IResult<&'a I, O, E>,
	&'a I: InputLength + InputTakeAtPosition,
	<&'a I as InputTakeAtPosition>::Item: AsChar + Clone,
{
	terminated(inner, multispace0)
}
fn trim_start<'a, I: 'a, F: 'a, O, E: ParseError<&'a I>>(
	inner: F,
) -> impl FnMut(&'a I) -> IResult<&'a I, O, E>
where
	F: Fn(&'a I) -> IResult<&'a I, O, E>,
	&'a I: InputLength + InputTakeAtPosition,
	<&'a I as InputTakeAtPosition>::Item: AsChar + Clone,
{
	preceded(multispace0, inner)
}
