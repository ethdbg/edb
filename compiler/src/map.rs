use pest::{Parser, Token};
use super::err::LanguageError;
use std::{
    collections::HashMap,
    iter::FromIterator,
};
use pest_derive::*;
// O(2n)
const _GRAMMER: &str = include_str!("map/grammar.pest");

#[derive(Parser)]
#[grammar = "map/grammar.pest"]
pub struct MapParser;

#[derive(Debug, Clone)]
struct Line(HashMap<usize, Symbol>, String);

type LineCol = (usize, usize);

// Byte Offset - Line-Column number
#[derive(Debug, Clone, PartialEq)]
enum Symbol {
    Char(LineCol),
    LeadingWhitespace(LineCol),
    NewLine(LineCol),
}

#[derive(Debug, Clone)]
pub struct Map {
    lines: Vec<Line>,
}

impl Map{
    pub fn new(source: &str) -> Result<Self, LanguageError> {
        let parsed = MapParser::parse(Rule::file, source)?;
        let mut lines: Vec<Line> = Vec::new();
        let mut curr_linestr = String::from("");

        parsed
            .flatten()
            .tokens()
            .fold(HashMap::new(), |mut acc, t| {
                match t {
                    Token::Start{rule, pos} => {
                        match rule {
                            Rule::newline => {
                                acc.insert(pos.pos(), Symbol::NewLine(pos.line_col()));
                                curr_linestr = pos.line_of().to_string();
                                acc
                            },
                            Rule::characters => {
                                acc.insert(pos.pos(), Symbol::Char(pos.line_col()));
                                acc
                            },
                            Rule::whitespace => {
                                acc.insert(pos.pos(), Symbol::LeadingWhitespace(pos.line_col()));
                                acc
                            },
                            _ => acc
                        }
                    },
                    Token::End{rule, ..} => {
                        match rule {
                            Rule::line => {
                                lines.push(Line(HashMap::from_iter(acc.drain()), curr_linestr.clone()));
                                acc
                            },
                            _ => acc
                        }
                    }
                }
            });

        Ok(Self { lines: lines })
    }
}


// TODO: Benchmark
#[cfg(test)]
mod tests {
    use super::*;
    use crate::test::Bencher;
    const test_str: &str = include_str!("map/file.txt");
    const small_str: &str = include_str!("map/small_file.txt");

    #[test]
    fn print_map() {
        let unparsed: &str =
"
Hello
    this is some random code
the first line has a space and then a \n
  lets see how this does :)
";
        /*
        let parsed = MapParser::parse(Rule::file, unparsed).unwrap();
        parsed.flatten().tokens().for_each(|t| {
            println!("{:?}", t);
        });
         */
        let map = Map::new(small_str).unwrap();
        map.lines.iter().for_each(|l| {
            println!("{:?}", l);
            println!("\n\n");
        });
    }
    /*
    #[test]
    fn test_big() {
        pretty_env_logger::try_init();
        Map::new(small_str).unwrap();
    }
     */
    #[bench]
    fn bench_parse(b: &mut Bencher) {
        b.iter(||
               Map::new(small_str).unwrap()
        )
    }
}
