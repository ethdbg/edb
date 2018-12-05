//! Codefile represents one source code file and all of the files it imports

use super::{Line, CompiledFiles, OpcodeOffset, CharOffset, LineNo, contract::{Contract, Find}, err::{LanguageError, NotFoundError}};
use failure::Error;
use std::path::PathBuf;

// every CodeFile is associated with a language

pub struct CodeFile {
    name: String,
    files: CompiledFiles,
}

// TODO: Assumes all contracts that are being debugged have unique names. Possible research
// required to make sure this assumption is safe to make
// Language compilers may do automatic namespacing
impl CodeFile {

    // TODO: make path a reference (Path, not PathBuf)
    /// Create a new instance of Code File
    pub fn new(files: CompiledFiles, path: PathBuf) -> Result<Self, Error> {
        let name = path.file_name()
            .ok_or(LanguageError::NotFound(NotFoundError::File))?
            .to_str()
            .ok_or(LanguageError::InvalidPath)?
            .to_owned();

        if path.is_dir() {
            return Err(LanguageError::NotFound(NotFoundError::File)).map_err(|e| e.into());
        }

        Ok(Self { files, name })
    }

    /// Find the root contract that is being debugged
    pub fn root_name(&self) -> &str {
        &self.name
    }

    pub fn unique_exists(&self, lineno: LineNo, contract: &str) -> Result<bool, Error> {
        Ok(self.files.contracts()
            .find(contract)?
            .source_map()
            .unique_exists(lineno))
    }

    pub fn unique_opcode_pos(&self, lineno: LineNo, contract: &str) -> Result<OpcodeOffset, Error> {
        self.files.contracts()
            .find(contract)?
            .source_map()
            .unique_opcode_pos(lineno)
    }

    // passthrough for Source Map Trait
    /// Get a byte offset in the bytecode from a line number
    pub fn opcode_pos_from_lineno(&self, lineno: LineNo, from: OpcodeOffset, contract: &str) -> Result<OpcodeOffset, Error> {
        self.files.contracts()
            .find(contract)?
            .source_map()
            .opcode_pos_from_lineno(lineno, from)
    }

    pub fn char_pos_from_lineno(&self, lineno: LineNo, contract: &str) -> Result<CharOffset, Error> {
        self.files.contracts()
            .find(contract)?
            .source_map()
            .char_pos_from_lineno(lineno)
    }

    pub fn lineno_from_char_pos(&self, offset: CharOffset, contract: &str) -> Result<LineNo, Error> {
        self.files.contracts()
            .find(contract)?
            .source_map()
            .lineno_from_char_pos(offset)
    }

    pub fn lineno_from_opcode_pos(&self, offset: OpcodeOffset, contract: &str) -> Result<LineNo, Error> {
        self.files.contracts()
            .find(contract)?
            .source_map()
            .lineno_from_opcode_pos(offset)
    }

    pub fn current_range(&self, offset: usize, contract: &str) -> Result<String, Error> {
        self.files.contracts()
            .find(contract)?
            .source_map()
            .current_range(offset)
            .map_err(|e| e.into())
    }

    pub fn current_line(&self, offset: usize, contract: &str) -> Result<Line, Error> {
        self.files.contracts()
            .find(contract)?
            .source_map()
            .current_line(offset)
            .map_err(|e| e.into())
    }

    pub fn last_lines(&self, offset: usize, count: usize, contract: &str) -> Result<Vec<Line>, Error> {
        self.files.contracts()
            .find(contract)?
            .source_map()
            .last_lines(offset, count)
            .map_err(|e| e.into())
    }

    pub fn next_lines(&self, offset: usize, count: usize, contract: &str) -> Result<Vec<Line>, Error> {
        self.files.contracts()
            .find(contract)?
            .source_map()
            .next_lines(offset, count)
            .map_err(|e| e.into())
    }
}
