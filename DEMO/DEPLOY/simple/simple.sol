pragma solidity ^0.4.5;

// Test file

contract SimpleStorage {
    uint storedData;
    uint someNumber;
    uint anotherNumber;
    string hello;

    function set(uint x) public {
        uint y = 0;
        storedData = x;
        while (y < storedData) {
            y = y + 1;
        }
        hello = "Greetings from Simple Storage";
        anotherNumber = someNumber + 100;
        y = x / 2;
        if (y > anotherNumber) {
            hello = "Hello Greater Than";
        }
        // storedData = 4919;
        storedData = x;
    }

    function get() view public returns (uint) {
        return storedData;
    }
}
