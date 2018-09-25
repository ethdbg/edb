use super::err::LanguageError;
use pest::{Parser, iterators::Pair};
use pest_derive::*;
use log::*;

const _GRAMMER: &str = include_str!("map/grammar.pest");

#[derive(Parser)]
#[grammar = "map/grammar.pest"]
pub struct MapParser;

/// creates a map of the source file
#[derive(Debug, Clone)]
pub struct Map<'a> {
    lines: Vec<Pair<'a, Rule>>,
}

type Range = (usize, usize);

impl<'a> Map<'a> {
    pub fn new(source: &'a str) -> Result<Self, LanguageError> {
        let parsed = MapParser::parse(Rule::file, source)?;
        let mut lines = Vec::new();

        parsed
            .flatten()
            .for_each(|p| {
                if p.as_rule() == Rule::line {
                    lines.push(p)
                }
            });

        Ok(Self { lines: lines })
    }

    pub fn line(&self, offset: usize) -> Option<usize> {
        self.lines
            .iter()
            .find(|l| {
                l.as_span().start() <= offset && l.as_span().end() >= offset
            })
            .and_then(|l| {
                Some(l.as_span().start_pos().line_col().0)
            })
    }

    // get the byte range of a line number
    pub fn offset(&self, line: usize) -> Option<Range> {
        self.lines
            .get(line)
            .and_then(|l| {
                Some((l.as_span().start(), l.as_span().end()))
            })
    }
}


// TODO: Benchmark
#[cfg(test)]
mod tests {
    use super::*;
    use log::*;
    use crate::test::Bencher;
    // const UNICODE_RANGE: &str = include_str!("map/utf8-test.txt");
    const CONTRACT:&str = include_str!("../../tests/contracts/solidity/voting/voting.sol");
    const LINUX_SRC:&str = include_str!("map/linux_source.c");
    const LARGE:&str = include_str!("map/1MB.txt");

    #[test]
    fn print_map() {
        pretty_env_logger::try_init();
        let map = Map::new(CONTRACT).unwrap();
        info!("LENGTH: {}", map.lines.len());
        map.lines.iter().for_each(|l| {
            info!("{:?}", l);
            info!("\n\n");
        });
    }

    #[test]
    fn test_contract() {
        Map::new(CONTRACT).unwrap();
    }

    /* TODO: testing the unicode range from 0 to 0x1fff. Does not yet work for unknown reasons
    #[test]
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
    fn bench_parser(b: &mut Bencher) {
        b.iter(||
               MapParser::parse(Rule::file, LARGE).unwrap()
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
