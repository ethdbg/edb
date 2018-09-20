<p align="center">
  <img src="https://raw.githubusercontent.com/ethdbg/edb/master/edb_logo.png" />
</p>

# EDB
https://www.youtube.com/channel/UCaN9mu_nq7_xlFaVq3986-g for development stream.


Not built around 'Just a Hook'. This debugger is built directly onto a customized VM, and has all of it's tools at its disposal. We make the hooks.

This Debugger plans to be unopinionated and general. As first, however, it will mainly support the Ethereum Main chain and the Solidity/Vyper languages.

This Repository will include:
- A General Debug Library for Debug Functions (Stepping/CallStack/etc)using sputnikvm
- A library for sourcemappings between LLL, Solidity, Vyper

Organizations Repository will include:
- An RPC which uses the Debug library and exposes the debug functions
- A GDB-like CLI client
- A VSCode Plugin


### Supported Languages
- [ ] Solidity
- [ ] Vyper
- [ ] Serpent
- [ ] LLL

### Supported Chains
- [ ] Ethereum Main
- [ ] Ethereum Classic
- [ ] Ellaism? (Research Required)
- [ ] Ubiq? (Research Required)
- [ ] Expanse? (Research Required)
- [ ] Musicoin? (Research Required)
