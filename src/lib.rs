/*
    Copyright (C) 2016  Janne Blomqvist

    This program is free software: you can redistribute it and/or modify
    it under the terms of the GNU General Public License as published by
    the Free Software Foundation, either version 3 of the License, or
    (at your option) any later version.

    This program is distributed in the hope that it will be useful,
    but WITHOUT ANY WARRANTY; without even the implied warranty of
    MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
    GNU General Public License for more details.

    You should have received a copy of the GNU General Public License
    along with this program.  If not, see <http://www.gnu.org/licenses/>.
 */

#[macro_use]
extern crate nom;

use std::str;
use nom::*;
use nom::IResult::*;

// A name component part of a hostlist, before the hostlist syntax begins
named!(hostname_part, take_until!(b"["));
// A range (something enclosed with [])
named!(range, delimited!(char!(b'['), take_until!(b"]"), char!(b']')));

// hostname-range pair
//named!(hnrangepair<&[u8], (&[u8], Option<&[u8]>) >, tuple!(basehn, opt!(hostrange)));

// A complete hostlist, e.g. foo[1-3]
named!(hostlist<&[u8], &[u8] >, chain!(
    name: opt!(hostname_part) ~
        range: opt!(range),
    || { name.unwrap()
    }));


// Expand a hostlist to a vector of hostnames
pub fn expand(hostlist: &str) -> Vec<String> {


    // Is this a hostlist at all?
    let baseend = match  hostlist.find('[') {
        None => return vec![hostlist.to_string()],
        Some(i) => i,
    };
    vec![hostlist[0..baseend].to_string()]
}


// Tests of private functions
#[test]
fn check_base() {
    let hostlist = b"foo[1-3]";
    let res = hostname_part(hostlist);
    let out = match res {
        Done(_, o) => str::from_utf8(&o).unwrap(),
        _ => panic!()
    };
    assert_eq!(out, "foo");
}


#[test]
fn simple_hostrange() {
    let hostlist = b"[1-3]";
    let res = range(hostlist);
    let mut out = "";
    match res {
        Done(_, o) => out = str::from_utf8(&o).unwrap(),
        _ => println!("{:?}", res)
    }
    assert_eq!(out, "1-3");
}

#[test]
fn simple_hostlist() {
    let myhl = b"foo[1-3]";
    let res = hostlist(myhl);
    let out = match res {
        Done(_, o) => str::from_utf8(&o).unwrap(),
        _ => panic!()
    };
    assert_eq!(out, "foo");
}

// Tests of public functions
#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn it_works() {
    }

    #[test]
    fn test_expand() {
        assert_eq!(expand("foo"), vec!["foo"]);
    }
}
