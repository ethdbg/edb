## Compiler

A layer built above the traditional Solidity/Vyper/Serpent/LLL Compilers built in rust especially for edb
in order to package bytecode with AST

For each language, this will resolve imports, and package everything with meta-info included
  - this is where source-mapping happens and can be queried
