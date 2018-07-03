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


pub mod emul;
mod externalities;
mod executive;
/**
 * args
#[derive(Clone, Debug)]
pub struct Args {
	/// Address of currently executed code.
	pub to: Option<Address>,
	/// Hash of currently executed code.
	pub code_hash: Option<H256>,
    pub from: Option<Address>,
	/// Gas paid up front for transaction execution
	pub gas: Option<U256>,
	/// Gas price.
	pub gas_price: Option<U256>,
	/// Transaction value.
	pub value: Option<ActionValue>,
	/// Code being executed.
	pub code: Option<Bytes>,
	/// Input data.
	pub data: Option<Bytes>,
}
 *
 */

#[cfg(test)]
mod tests {
    use super::emul::{Emul, Args};
    use super::rustc_hex::FromHex;
    use super::vm::{ActionParams};
    use std::sync::Arc;

    
    #[test]
    fn it_should_create_emulator_instance() {
        let mut params = ActionParams::default();
        let code = "606060405260005b620f42408112156019575b6001016007565b600081905550600680602b6000396000f3606060405200"
            .from_hex().unwrap();
        params.code = Some(Arc::new(code));
        let emul = Emul::new(params).unwrap();;
    }

    fn it_should_run_code() {
        let mut params = ActionParams::default();
        let code = "606060405260005b620f42408112156019575b6001016007565b600081905550600680602b6000396000f3606060405200"
            .from_hex().unwrap();
        params.code = Some(Arc::new(code));
        let emul = Emul::new(params).unwrap();
        emul.run_code();

    }
}
