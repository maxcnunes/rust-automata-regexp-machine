# rust-automata-regexp-machine

TLTR: This is an incomplete and broken regex engine implementation.

I got interested how Regex works internally. The [Automata Theory: inside a RegExp machine course
](https://www.udemy.com/course/automata-theory-building-a-regexp-machine) by Dmitry Soshnikov, was a great
resource to understand it.

I decided to implement it, but it was taking way longer than I expected, getting confused in some implementation details, so I eventually aborted it.

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
