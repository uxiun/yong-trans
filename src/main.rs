#![feature(iter_intersperse)]
#![allow(dead_code, unused, non_snake_case)]

use std::collections::HashMap;

mod kt;
mod py;
mod mcr;
mod spell;
mod parser;
mod util;
mod sta;
mod out; //execute translate
fn main() {

  // let p = '略'.to_pinyin().unwrap().with_tone_num_end();
  // println!("{} ", p);
  out::main();
}

fn readfile(filename: &str) {}

pub type SpellCharsMap = HashMap<String, String>;
