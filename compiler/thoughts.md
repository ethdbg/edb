- The only source_map() func that should be using shortest_len is run_until. That way, it will find the opcode with the shortest 'length' (if an assignment, that occurs before the actual assignment). If line_cache does not contain a shortest_len(), then it tells the user that a breakpoint cannot be set there. This occurs on things where breakpoints shouldn't be set anyway, like function or contract type declarations.
-
- step() will find the range of the next line (w/o leading whitespace) and find the first opcode position (relative to the last opcode) that is within that range.
    - step_back() does the reverse

