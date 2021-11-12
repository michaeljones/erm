
# Erm

This project is a toy Elm interpreter written in Rust following some of the guidance from the
[Crafting Interpreters](http://craftinginterpreters.com/) book.

It is primarily a learning exercise.

## Status

It does not really work at the moment.

- [x] Explicit import
- [x] List syntax
- [ ] Runnable binary with errors
- [ ] Tuple syntax
- [ ] Let-in blocks
- [ ] Generic number handling
- [ ] Underscore to ignore
- [ ] Function type annotations
- [ ] Type alias declarations
- [ ] Type declarations
- [ ] Record syntax
- [ ] Record updatn syntax
- [ ] Case statement
- [ ] Pattern matching
- [ ] Destructuring
- [ ] Generic 'appendable' handling

No current plans to support:

- Ports
- WebGL syntax

## Notes

- Explore the idea of supporting a dhall like approach to generating config. Perhaps a 'Config'
  output type for the 'main' function and some kind of automatic support for json/yaml/toml outputs.

## Tests

Run tests with logging output:

```
RUST_LOG=trace cargo test -- --nocapture
```

## Links

### Elm Resources

- [Full syntax](https://github.com/pdamoc/elm-syntax-sscce/blob/main/src/Main.elm)
- [Prelude](https://github.com/elm/compiler/blob/770071accf791e8171440709effe71e78a9ab37c/compiler/src/Elm/Compiler/Imports.hs#L20-L33)
- [When to include prelude](https://github.com/elm/compiler/blob/770071accf791e8171440709effe71e78a9ab37c/compiler/src/Parse/Module.hs#L80)
- [Elm-in-Elm AST](https://github.com/elm-in-elm/compiler/blob/master/src/Elm/AST/Canonical.elm)
- [Elm-in-Elm Type Structure](https://github.com/elm-in-elm/compiler/blob/master/src/Elm/Data/Type.elm)

### Type Inference

- https://eli.thegreenplace.net/2018/type-inference/
- https://eli.thegreenplace.net/2018/unification/
- http://dev.stephendiehl.com/fun/index.html
- https://medium.com/@aleksandrasays/type-inference-under-the-hood-f0ebbeb005a3
- https://dev.to/dannypsnl/hindley-milner-type-system-incrementally-build-way-make-new-language-in-racket-307j
- https://www.lesswrong.com/posts/vTS8K4NBSi9iyCrPo/a-reckless-introduction-to-hindley-milner-type-inference
- https://cstheory.stackexchange.com/questions/25573/what-are-some-good-introductory-books-on-type-theory
- http://reasonableapproximation.net/2019/05/05/hindley-milner.html

### Parsing Binary Expressions

- https://eli.thegreenplace.net/2009/03/20/a-recursive-descent-parser-with-an-infix-expression-evaluator
- http://www.engr.mun.ca/~theo/Misc/exp_parsing.htm
