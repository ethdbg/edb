pragma solidity ^0.4.0;

import "./another/importing.sol";

contract SimpleStorage {
    uint storedData;
    uint from_import;
    SimpleStorageTwo some_import;

    function set(uint x) {
        storedData = x;
        some_import = SimpleStorageTwo(x);
    }

    function get() constant returns (uint) {
        from_import = some_import.get();
        return storedData;
    }
}
