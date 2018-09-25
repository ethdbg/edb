## Compiler

A layer built above the traditional Solidity/Vyper/Serpent/LLL Compilers built in rust especially for edb
in order to package bytecode with AST

For each language, this will resolve imports, and package everything with meta-info included
  - this is where source-mapping happens and can be queried



Map Performance Bencharks:
```
test map::tests::bench_1MB      ... bench:   3,657,547 ns/iter (+/- 120,671)
test map::tests::bench_contract ... bench:      29,382 ns/iter (+/- 465)
test map::tests::bench_linux    ... bench:     834,450 ns/iter (+/- 6,876)
test map::tests::unicode_0x1fff ... bench:     449,309 ns/iter (+/- 5,614)
```


