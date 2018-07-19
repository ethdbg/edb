- want to use `finalize` function in evm.rs, not test_finalize in vm::tests because finalize in vm::tests no return data
-
-
-


- Command line arguments is parsed via Configuration::parse_cli and then passed to `start` method in  parity/lib.rs
  - 'conf' is then modified with `into_command()` which is then passed into the `execute` method in parity/lib.rs
    - it is now of 'Execute' type
  - 
  -
  - 'spec' identifies chain. ie, dev, kovan, mainnet, etc
