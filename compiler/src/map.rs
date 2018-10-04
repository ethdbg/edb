//! Map of source file. Line numbers are zero-indexed.
//! Byte offset refers to character offset rather than actual UTF-8 bytes/codepoints
use super::err::{MapError};
use std::iter::FromIterator;
// creates a map of the source file
#[derive(Debug, Clone)]
pub struct Map {
    pub matrix: Vec<Vec<char>>, // matrix of chars.  every index of outer Vec is a line
}

/// A line number
pub type Line = usize;
/// A column number
pub type Col = usize;
/// A Byte offset into the source file
pub type ByteOffset = usize;
/// Start and end offsets for a line
pub type Range = (ByteOffset, ByteOffset);

/// An offset represented by an enum.
/// Options for where the offset should reside
pub enum LineNumber {
    /// get the start of a line without including the leading whitespace
    NoLeadingWhitespace(Line),
    /// get the canonical start of a line (including leading whitespace)
    Start(Line),
    /// get the end of a line
    End(Line),
    /// get the canonical range of a line, including leading whitespace
    Range(Line),
    /// Get the range of a line, without leading whitespace
    NoWhitespaceRange(Line),
}

impl LineNumber {
    fn get_line(&self) -> Line {
        match *self {
            LineNumber::NoLeadingWhitespace(l) => l,
            LineNumber::Start(l) => l,
            LineNumber::End(l) => l,
            LineNumber::Range(l) => l,
            LineNumber::NoWhitespaceRange(l) => l,
        }
    }
}

/// A character position in the source code
pub enum CharPosition {
    LineCol(Line, Col),
    Offset(ByteOffset),
}

#[derive(Debug, Clone, PartialEq)]
enum CharType {
    Whitespace,
    Any
}

impl PartialEq<char> for CharType {
    fn eq(&self, other: &char) -> bool {
        match self {
            CharType::Whitespace => {
                other == &' '
            },
            CharType::Any => {
                other != &' '
            }
        }
    }
}


impl PartialEq<CharType> for char {
    fn eq(&self, other: &CharType) -> bool {
        match other {
            CharType::Whitespace => {
                self == &' '
            },
            CharType::Any => {
                self != &' '
            }
        }
    }
}

impl Map {

    /// new map
    pub fn new(source: &str) -> Self {
        let mut matrix = Vec::new();

        for line in source.lines() {
            let mut vec = line.chars().collect::<Vec<char>>();
            vec.push('\n');
            matrix.push(vec);
        }
        Self { matrix }
    }

    /// Returns how many lines are contained in the source file
    pub fn len(&self) -> usize {
        self.matrix.len()
    }

    /// find the line that contains this offset
    pub fn find_line(&self, byte_offset: ByteOffset) -> Option<Line> {

        self.matrix
            .iter()
            .enumerate()
            .find(|(line, _)| {
                self.in_range(byte_offset, *line)
            })
            .map(|(idx, _)| idx)
    }

    /// find a byte offset from a line number
    pub fn find_offset(&self, line: LineNumber) -> Result<ByteOffset, MapError> {
        match line {
            LineNumber::NoLeadingWhitespace(_) => {
                Ok(self.range(line)?.0)
            },
            LineNumber::Start(_) => {
                Ok(self.range(line)?.0)
            },
            LineNumber::End(_) => {
                Ok(self.range(line)?.1)
            },
            _ => Err(MapError::CannotGetRange)
        }
    }

    /// get a line as a character slice
    pub fn line(&self, line: Line) -> Result<&[char], MapError> {
        if self.matrix.len() < line {
            Err(MapError::LineOutOfBounds)
        } else {
            Ok(self.matrix[line].as_slice())
        }
    }

    pub fn lines(&self, range: std::ops::Range<usize>) -> Result<Vec<String>, MapError>{
        if self.matrix.len() < range.end {
            Err(MapError::LineOutOfBounds)
        } else {
            Ok(self.matrix[range].iter().map(|s| String::from_iter(s.as_slice())).collect::<Vec<String>>())
        }
    }

    /// find the byte range of a line number (start offset and end offset)
    /// For LineNumber::Start, and LineNumber::End, returns canoncial start and end
    /// LineNumber::NoLeadingWhitespace returns start and end w/o leading whitespace
    pub fn range(&self, line: LineNumber) -> Result<Range, MapError> {
        println!("Finding byte range of line: {}: {:?}", line.get_line(), self.line(line.get_line())?);
        let line_str = self.line(line.get_line())?;
        let start = self.matrix
            .iter()
            .take(line.get_line())
            .fold(0, |acc, l| acc + l.len());
        let end = start + line_str.len();
        println!("Byte Range: {} - {}", start, end);

        match line {
            LineNumber::NoLeadingWhitespace(_) | LineNumber::NoWhitespaceRange(_) => {
                let local_idx = line_str
                    .iter()
                    .enumerate()
                    .skip_while(|(_, &c)| c == CharType::Whitespace)
                    .map(|(idx, _)| idx)
                    .take(1)
                    .fold(0, |acc, i| acc + i);
                Ok((start+local_idx, end))
            },
            LineNumber::Start(_) | LineNumber::End(_) | LineNumber::Range(_) => Ok((start, end))
        }
    }

    pub fn get_char(&self, char_pos: CharPosition) -> Result<&char, MapError> {

        match char_pos {
            CharPosition::LineCol(line, col) => {
                let line = self.line(line)?;
                if col > line.len() {
                    Err(MapError::ColOutOfBounds)
                } else {
                    Ok(&line[col])
                }
            },
            CharPosition::Offset(offset) => Ok(self.matrix.iter().flatten().nth(offset).ok_or(MapError::OutOfBounds)?)
        }
    }

    /// check if `offset` is in range of a character slice
    fn in_range(&self, offset: usize, line_num: Line) -> bool {
        let (start, end) = self.range(LineNumber::Range(line_num)).expect("Should never be out of range in internal function; qed");
        (offset >= start) && (offset <= end)
    }
}


// TODO: Benchmark
#[cfg(test)]
mod tests {
    use super::*;
    use speculate::speculate;
    use edb_test_helpers as edbtest;
    use crate::test::Bencher;
    const UNICODE_RANGE: &str = include_str!("map/utf8-test.txt");
    const LINUX_SRC:&str = include_str!("map/linux-source-c.test");
    const LARGE:&str = include_str!("map/1MB.txt");
    const TEST_STR:&str =
"pragma solidity ^0.4.22;

/// @title Voting with delegation.
contract Ballot {
    // This declares a new complex type which will
    // be used for variables later.
    // It will represent a single voter.
    struct Voter {
        uint weight; // weight is accumulated by delegation
        bool voted;  // if true, that person already voted
        address delegate; // person delegated to
        uint vote;   // index of the voted proposal
    }

    // This is a type for a single proposal.
    struct Proposal {
        bytes32 name;   // short name (up to 32 bytes)
        uint voteCount; // number of accumulated votes
    }
";

    speculate! {
        before {
            #[allow(unused_must_use)] {
                pretty_env_logger::try_init();
            }
            let map = Map::new(TEST_STR);
        }

        it "can get a line from an offset" {
            assert_eq!(map.find_line(62).unwrap(), 3);
        }

        it "can get an offset" {
            assert_eq!(map.find_offset(LineNumber::NoLeadingWhitespace(3)).unwrap(), 61);
            assert_eq!(map.find_offset(LineNumber::Start(3)).unwrap(), 61);
            assert_eq!(map.find_offset(LineNumber::End(3)).unwrap(), 79);
        }

        it "should strip leading whitespace from offset" {
            assert_eq!(map.find_offset(LineNumber::NoLeadingWhitespace(7)).unwrap(), 211);
            assert_eq!(*map.get_char(CharPosition::Offset(211)).unwrap(), 's');
        }
    }

    #[test]
    fn test_contract() {
        Map::new(edbtest::contract_path(edbtest::Contract::Voting).to_str().unwrap());
    }

    // TODO: testing the unicode range from 0 to 0x1fff. Does not yet work for unknown reasons
    #[bench]
    fn unicode_0x1fff(b: &mut Bencher) {
        b.iter(||
               Map::new(UNICODE_RANGE)
        )
    }
    #[bench]
    fn bench_contract(b: &mut Bencher) {
        b.iter(||
            Map::new(edbtest::contract_path(edbtest::Contract::Voting).to_str().unwrap())
        )
    }

    #[bench]
    fn bench_linux(b: &mut Bencher) {
        b.iter(||
               Map::new(LINUX_SRC)
        )
    }

    #[bench]
    fn bench_1mb(b: &mut Bencher) {
        b.iter(||
               Map::new(LARGE)
        )
    }
}
