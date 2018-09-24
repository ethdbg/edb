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
    use log::*;
    use crate::test::Bencher;
//    const test_str: &str = include_str!("map/file.txt");
    const UNICODE_RANGE: &str = include_str!("map/utf8-test.txt");
    const CONTRACT:&str = include_str!("../../tests/contracts/solidity/voting/voting.sol");
    const LINUX_SRC:&str = include_str!("map/linux_source.c");
    const LARGE:&str = include_str!("map/1MB.txt");

    #[test]
    fn print_map() {
        pretty_env_logger::try_init();
        let map = Map::new(CONTRACT).unwrap();
        map.lines.iter().for_each(|l| {
            println!("{:?}", l);
            println!("\n\n");
        });
    }

    #[test]
    fn test_contract() {
        Map::new(CONTRACT).unwrap();
    }
/*
    #[bench]
    fn unicode_0x1fff(b: &mut Bencher) {
        b.iter(||
               Map::new(UNICODE_RANGE).unwrap()
        )
    }
*/
    #[bench]
    fn bench_contract(b: &mut Bencher) {
        b.iter(||
               Map::new(CONTRACT).unwrap()
        )
    }

    #[bench]
    fn bench_linux(b: &mut Bencher) {
        b.iter(||
               Map::new(LINUX_SRC).unwrap()
        )
    }

    #[bench]
    fn bench_1MB(b: &mut Bencher) {
        b.iter(||
               Map::new(LARGE).unwrap()
        )
    }
}
