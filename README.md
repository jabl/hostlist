Hostlist
========

![Rust](https://github.com/jabl/hostlist/workflows/Rust/badge.svg)

This is a Rust implementation of a hostlist library. Hostlists are a
syntax for expressing multiple hosts, commonly used in
HPC. E.g. compute[1-3] => compute1, compute2, compute3. However it
goes a bit beyond what can be done with plain bash expansions of the
type compute{1..3}.

Some other tools and applications supporting hostlists are

- [Slurm](http://slurm.schedmd.com/)
- [pdsh](https://github.com/grondo/pdsh)
- [genders](https://github.com/chaos/genders)
- [GNU FreeIPMI](https://www.gnu.org/software/freeipmi/)


## Usage

The library provides a single public function, with the signature

    pub fn expand(a_str: &str) -> Result<Vec<String>, &'static str>

This function will expand a hostlist into a list of
hostnames. E.g. "foo[1-3]" will result in a vector ["foo1", "foo2",
"foo3"].

## Command-line utility

A so far VERY rudimentary CLI app called `hostlist` is included,
allowing to use the `expand()` function from the command line.

## Building only the library

If one doesn't want the command line application, one can build the library only
with

`cargo build --no-default-features`
