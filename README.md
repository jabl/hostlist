Hostlist
========

[![Build Status](https://travis-ci.org/jabl/hostlist.svg?branch=master)](https://travis-ci.org/jabl/hostlist)

This is a Rust implementation of a hostlist library. Hostlists are a
syntax for expressing multiple hosts, commonly used in
HPC. E.g. compute[1-3] => compute1, compute2, compute3. However it
goes a bit beyond what can be done with plain bash expansions of the
type compute{1..3}.

Some other tools and applications supporting hostlists are

- [Slurm](http://slurm.schedmd.com/)
- [pdsh](https://github.com/grondo/pdsh)
- [genders](https://github.com/chaos/genders)
