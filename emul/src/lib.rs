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

#[macro_use]
extern crate log;
extern crate vm;
extern crate evm;
extern crate ethcore_bytes as bytes;
extern crate ethcore;
extern crate ethcore_transaction as transaction;
extern crate ethereum_types;
extern crate keccak_hash as hash;
extern crate rustc_hex;
extern crate crossbeam;
extern crate rlp;
extern crate ethcore_io;


pub mod emulator;
mod instruction_manager;
mod externalities;
mod extensions;
mod tests;
// mod factory;
 
