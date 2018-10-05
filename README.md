# Playbot NG (eval)
[![build status](https://travis-ci.com/panicbit/playbot_ng.svg?branch=master)](https://travis-ci.com/panicbit/playbot_ng)

Playbot NG is a drop-in replacement for the retired IRC bot named playbot.
There's an instance of Playbot NG named **eval** running on
[Moznet](https://wiki.mozilla.org/IRC).

## Usage

Interaction with playbot is done either via commands
or by talking directly to it.(can add Computer Vision)

### Code evaluation

Playbot will evaluate Rust code that you give to it via private message or
by prefixing a message with its nickname (followed by a colon),
e.g. `eval: 42 + 777`.
By default the code is wrapped in a template that prints the result of the
expression via its `Debug` impl.
This behaviour can be changed (see `--bare`/`--mini` below)

There a few flags that can modify the behaviour of the evaluation.
You can select the release channel using `--stable` (default), `--beta`,
or `--nightly`.
You can pick the build profile using `--debug` (default), or `--release`.
To make playbot evaluate your code as it is, pass `--bare` or `--mini`.
To make playbot print the Rust version, pass `--version` (can be combined with the above channel flags; code is ignored).

For convenience, inner attributes at the beginning of code are treated as crate attributes, e.g. `eval: --nightly #![feature(nll)] …`.
This rule does not apply when `--bare` or `--mini` is given.

To display a link to this help you can pass `help`, `h`, `-h`, `-help`, `--help`, or `--h`.

### Commands

Commands are prefixed by `?` and can have parameters,
e.g. `?crate itertools`.
They can also be used inline by enclosing them into braces,
e.g. `You can use {?crate itertools} for that.`

#### Command `?crate <crate>`

Display information about `<crate>` from `crates.io`

Example: `?crate itertools`

#### Command `?help`

Display a link to this help

Example: `?help`
