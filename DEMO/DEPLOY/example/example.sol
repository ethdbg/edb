pragma solidity ^0.4.5;

// Test file

contract Example {
    uint someNumber;
    uint anotherNumber;
    string stored;

    function set(string input) public {
        uint y = 0;
        anotherNumber = 20;
        someNumber = 10;
        stored = input;
        if (y < anotherNumber) {
            y = 200;
        } else {
            y = 0;
        }
        anotherNumber = someNumber + 100;
        y = anotherNumber / 2;
    }

    function get() view public returns (string) {
        return stored;
    }
}
