# m2-pylox

This is a port of the bytecode interpreter for the Lox programming language featured in the [Crafting Interpreters](https://craftinginterpreters.com/) book.

# Implemented chapters and challenges

* Chapter 14
* Chapter 15
* Chapter 16
* Chapter 17
* Chapter 18

# Implementation differences

There are some changes in comparison with the reference implementation provided in the book.

* No `ERROR` or `EOF` tokens, the `scan_token()` return is a `Result<Option<Token>>, &str>`, with `Err(&str)` replacing the error tokens and `Ok(None)` indicating the end of file.
* Instruction pointer is not a pointer, instead is an index into the current chunk.
* Bytecode is a series of enum values instead of bytes. Each enum packs the necessary information as members, which results in slightly larger bytecode due to it being fixed width, but it's easier to handle since it is integrated into the typesystem.
* Planned but not yet implemented: Compiler is a struct that only needs one instance, and instead of chaining enclosing instances it keeps two stacks of contexts, one stack for the normal Compiler data, and another for the ClassCompiler. This should lead to a simpler ownership model and easy handling of compiler nesting.
