pragma solidity ^0.4.0;

contract SimpleStorage {
    uint storedData;
    uint someNumber;
    uint anotherNumber;
    string hello;
    uint y;
    function set(uint x) {
        storedData = x;
        someNumber = 0;
        while (someNumber < (10/2)) {
            someNumber = someNumber + 1;
        }
        hello = "Greetings from Simple Storage";
        anotherNumber = someNumber + 100;
        y = x / 2;
        if (y > anotherNumber) {
            hello = "Hello Greater Than";
        }
    }

    function get() constant returns (uint) {
        return storedData;
    }
}
