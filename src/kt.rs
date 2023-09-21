
use std::{
  collections::HashMap,
  fs::File,
  io::{self, BufRead, BufReader, Lines}, path::Path, fmt::Debug,
};

use crate::{parser::{YongDef, read_line_cangjie_def, read_line_xxyx_def, read_line_yong_def, read_line_custom_add, StringStringsDict, StringStringsEntry}, d, util::unions_hashmap, spell::{YongSpelling, SpecifySpelling, WordSpellsLineFormat}};

pub fn main(){
  // let merged = to_yong_dict("table/cj5q-90000");
  let hs = 
    [ to_yong_dict("table/xxyx.txt")
    , to_yong_dict("table/cj5q-90000.txt")
    ];
  let mut m: YongDictWordSpells = HashMap::new();
  let merged = 
    unions_hashmap(&mut m, hs.into_iter().collect())
  ;
  let va = d!([
    merged.get("看"),
    merged.get("上"),
    merged.get("車"),
    ]);
}



pub fn to_yong_dict<P: AsRef<Path> + Copy + Debug>
  (path: P)-> YongDictWordSpells {
  let spelling: YongSpelling = if path_file_or_dict_prefix_match(path, "xxyx") {
    YongSpelling::Xxyx
  } else if path_file_or_dict_prefix_match(path, "cj") {
    YongSpelling::Cangjie
  } else {
    YongSpelling::Free
  };
  println!("start collect defs");
  // let mut dict: YongDictWordSpells = HashMap::new();

  if let Some(ly) = LinesBufYong::yong_def_part(path) {
    let defs = ly.collect_yong_defs(spelling);
    println!("collected defs");
    YongDefGroup {defs}.to_dict(spelling)
  } else {
    println!("seems the file has no def part");
    let couldnotopen = d!(path);
    HashMap::new()
  }
  
}
pub fn from_word_spells_dict<P: AsRef<Path> + Copy + Debug>
  (path: P)-> YongDictWordSpells {
  let format: WordSpellsLineFormat = if path_file_or_dict_prefix_match(path, "add-short") {
    WordSpellsLineFormat::AddShorter
  } else {
    WordSpellsLineFormat::Free
  };
  println!("start collect defs with format:");
  // let format = d!(format);
  // let mut dict: YongDictWordSpells = HashMap::new();

  if let Some(ly) = LinesBufYong::def_part_entire(path) {
    let defs = ly.collect_word_spells(format);
    println!("collected defs");
    defs.into_iter().map(|(k,v)| (k, 
      v.into_iter().map(|spell| SpecifySpelling {
        spell,
        spelling: YongSpelling::Free
      } ).collect()
    )).collect()
  } else {
    println!("seems the file has no def part");
    let couldnotopen = d!(path);
    HashMap::new()
  }
  
}

fn path_file_or_dict_prefix_match<P>(path: P, pat: &str)->bool 
where P: AsRef<Path> 
{
  path.as_ref().starts_with(pat) || {
    if let Some(name) = path.as_ref().file_name() {
      if let Some(n) =  name.to_str() {
        n.starts_with(pat)
      } else {false}
    } else {
      false
    }
  }
}

pub type YongDictWordSpells = HashMap<String, Vec<SpecifySpelling>>;


#[derive(Debug)]
pub struct YongDefGroup {
  defs: Vec<YongDef>
}
impl YongDefGroup {
  pub fn to_dict(self, spelling: YongSpelling)-> YongDictWordSpells {
    let mut dict: YongDictWordSpells = HashMap::new();
    let defs = self.defs;
    for def in defs.iter() {
      for word in def.words.iter() {
        if let Some(m) = dict.get_mut(word) {
          m.push( SpecifySpelling {
            spell: def.spell.to_owned(),
            spelling
          }
          );
        } else {
          dict.insert(word.to_owned(), vec![
            SpecifySpelling {
              spelling,
              spell: def.spell.to_owned()
            }
          ]);
        }
      }
    }
    dict
  }
} 


pub type LinesBuf = Lines<BufReader<File>>;



#[derive(Debug)]
pub struct LinesBufYong {
  x: LinesBuf
}
impl LinesBufYong {
  fn def_part_entire<P: AsRef<Path>>(path: P) -> Option<Self> {
    if let Ok(ls) = read_lines(path) {
      Some(LinesBufYong { x: ls })
    } else {None}
  }
  fn yong_def_part<P: AsRef<Path>>(path: P) -> Option<Self> {
    let mut l = read_lines(path);
    
    if let Ok(mut ls) = l {
      // let i = ls.position(|line| {
      //   if let Ok(l) = line {
      //     l.trim_end() == "[DATA]"
      //   } else {false}
      // });
      // if let Some(i) = i {
      //   ls.
      // } else {None}
  
      loop {
        if let Some(Ok(line)) = ls.next() {
          if line.trim_end() == "[DATA]" {break;}
        } else {
          break;
        }
      }
      Some(LinesBufYong { x: ls })
    } else {None}
  }
  
  fn collect_yong_defs(self, spelling: YongSpelling) -> Vec<YongDef> {
    let f = match spelling {
      YongSpelling::Cangjie => read_line_cangjie_def,
      YongSpelling::Xxyx => read_line_xxyx_def,
      YongSpelling::Free => read_line_yong_def,
    };
    self.x.into_iter()
      .filter_map(|l| if let Ok(l) = l {
        f(l)
      } else {None})
      .collect()
  }
  fn collect_word_spells(self, format: WordSpellsLineFormat) -> Vec<StringStringsEntry> {
    let f = match format {
      WordSpellsLineFormat::AddShorter=> read_line_custom_add,
      WordSpellsLineFormat::Free=> read_line_custom_add
    };
    self.x.into_iter()
      .filter_map(|l| if let Ok(l) = l {
        f(l)
      } else {None})
      .collect()
  }

  
}



pub fn read_lines<P>(path: P)
-> io::Result<LinesBuf>
where P: AsRef<Path>
 {
  let f = File::open(path)?;
  Ok(BufReader::new(f).lines())
}
