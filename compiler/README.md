## Compiler

A layer built above the traditional Solidity/Vyper/Serpent/LLL Compilers built in rust especially for edb
in order to package bytecode with AST

For each language, this will resolve imports, and package everything with meta-info included
  - this is where source-mapping happens and can be queried



Map Performance Bencharks:
```
Finished release [optimized] target(s) in 0.14s
     Running /home/insi/Projects/EDB/edb/target/release/deps/edb_compiler-daeb0722640bd0bd
running 5 tests
test map::tests::bench_1MB      ... bench:     928,016 ns/iter (+/- 7,065)
test map::tests::bench_contract ... bench:       8,448 ns/iter (+/- 68)
test map::tests::bench_linux    ... bench:     193,695 ns/iter (+/- 1,311)

test result: ok. 0 passed; 0 failed; 2 ignored; 3 measured; 0 filtered out

```
