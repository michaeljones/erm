
# Erm

This project is a toy Elm interpreter written in Rust following some of the guidance from the
[Crafting Interpreters](http://craftinginterpreters.com/) book.

It is primarily a learning exercise.

## Status

It does not really work at the moment.

## Notes

- Explore the idea of supporting a dhall like approach to generating config. Perhaps a 'Config'
  output type for the 'main' function and some kind of automatic support for json/yaml/toml outputs.

## Tests

Run tests with logging output:

```
RUST_LOG=trace cargo test -- --nocapture
```

## Links

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


