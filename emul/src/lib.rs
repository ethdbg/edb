extern crate vm;
extern crate evm;
extern crate ethcore_bytes as bytes;
extern crate ethcore;
extern crate ethereum_types;
pub mod emul;
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
    use super::emul::Emul;
    #[test]
    fn it_should_create_emulator_instance() {
        let args: Emul::Args = 
        Emul::new() 
    }
}
