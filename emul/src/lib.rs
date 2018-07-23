// Copyright 2015-2018 Parity Technologies (UK) Ltd.
// This file is part of Parity.

// Parity is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// Parity is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with EDB. If not, see <http://www.gnu.org/licenses/>.
#![recursion_limit="512"]

#[macro_use]
extern crate log;
#[macro_use]
extern crate delegate;

extern crate vm;
extern crate evm;
extern crate parity_bytes as bytes;
extern crate ethcore;
extern crate ethcore_transaction as transaction;
extern crate ethereum_types;
extern crate keccak_hash as hash;
extern crate rustc_hex;
extern crate crossbeam;
extern crate rlp;
extern crate ethcore_io;
//extern crate parity; // should really get rid of this dependency; all of parity is not needed
extern crate kvdb; // key-value database
extern crate kvdb_rocksdb;
extern crate blooms_db;
extern crate tempdir;
extern crate journaldb;
extern crate kvdb_memorydb;
extern crate patricia_trie_ethereum;
extern crate rayon;

pub mod emulator;
mod externalities;
// mod executive;
mod extensions;
mod tests;
mod err;
mod utils;
// mod factory;
 
