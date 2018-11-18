<p align="center">
  <img src="https://raw.githubusercontent.com/ethdbg/edb/master/edb_logo.png" />
</p>

# EDB


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
- [x] Solidity
    - [x] Geth/Parity/etc
    - [x] Single File
    - [ ] imports/libraries
    - [ ] multiple file
    - [ ] automatic deployment
- [ ] Vyper
- [ ] Serpent
- [ ] LLL
- [ ] ASM

### Daemon
- [ ] Headless RPC
- [ ] Unix Socket/IPC daemon
- [ ] RLS (Remote Language Server)

### Supported Chains
- [x] Ethereum Main
- [ ] Ethereum Classic
- [ ] Ellaism? (Research Required)
- [ ] Ubiq? (Research Required)
- [ ] Expanse? (Research Required)
- [ ] Musicoin? (Research Required)
_these networks are listed because they are supported by [sputnikvm](https://github.com/ETCDEVTeam/sputnikvm), the VM behind EDB_

