# rust-automata-regexp-machine

This is the output of my journey learning the Automata theory (NFA and DFA) to understand how a Regex machine works internally.

For the Regex pattern parser to an Abstract Syntax Tree (AST), I basically copied Rust's own regex AST implementation: https://github.com/rust-lang/regex.
I recommend checking out the original source code, I found it an elegant implementation and very well documented. It was a great resource
as Rust learning material.

For the regex machine I followed the [Automata Theory: inside a RegExp machine course
](https://www.udemy.com/course/automata-theory-building-a-regexp-machine) by Dmitry Soshnikov who I am very grateful for such incredible course.

## Online Regex Tools

- https://www.debuggex.com/
- http://regviz.org/
- https://regexper.com
- https://regex101.com/
- https://regexr.com/

## References

- https://github.com/rust-lang/regex
- https://developer.mozilla.org/en-US/docs/Web/JavaScript/Guide/Regular_expressions/Cheatsheet
- https://en.wikipedia.org/wiki/Regular_expression
- https://github.com/DmitrySoshnikov/regexp-tree
- https://www.udemy.com/course/automata-theory-building-a-regexp-machine

## TODO

- Generate graph for Regex AST and NFA/DFA tables
- Add basic documentation about Automata
