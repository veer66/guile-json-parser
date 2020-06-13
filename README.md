# guile-json-parser

guile-json-paser is a JSON parser for GNU Guile. guile-json-paser is written in Rust and uses [serde\_json](https://github.com/serde-rs/json) for parsing JSON.

I didn't test it yet whether it is faster or slower than JSON parser written in pure Guile.

## Require

* GNU Guile 3.X
* Rust build tool

## How to try

```
$ cargo build --release
$ guile guile-example/test1.scm
```

## Example

````Scheme
(use-modules (json parser))
(display (read-string "[1, \"xxx\", {\"a\":30}]"))
````