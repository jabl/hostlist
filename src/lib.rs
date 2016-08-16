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
use std::collections::BTreeMap;

// A name component part of a hostlist, before the hostlist syntax begins
named!(hostname_part, take_until!(b"["));
// A range (something enclosed with [])
named!(range, delimited!(char!(b'['), take_until!(b"]"), char!(b']')));

// hostname-range pair
named!(hnrangepair<&[u8], (Option<&[u8]>, Option<&[u8]>) >, tuple!(
    opt!(hostname_part), opt!(range)));

// A complete hostlist, e.g. foo[1-3]
named!(hostlist<&[u8], Vec<(Option<&[u8]>, Option<&[u8]>)> >,
       many1!(hnrangepair));

// Convert a range in string format to a btreemap
// In principle one could use an interval or segment tree, but here
// we're just merging overlapping intervals instead of keeping track
// of them. So a plain btreemap is fine. The key is the lower end of
// the range, the value is the upper end.
fn range2tree(a_str: &str) -> BTreeMap<i32, i32> {
    let mut range = BTreeMap::new();
    range.insert(1, 3);
    range
}

// Expand a hostlist to a vector of hostnames
pub fn expand(a_str: &str) -> Vec<String> {


    // Is this a hostlist at all?
    /*let baseend = match  hostlist.find('[') {
        None => return vec![hostlist.to_string()],
        Some(i) => i,
    };
    vec![hostlist[0..baseend].to_string()]*/

    // New impl using Nom
    let p = hostlist(a_str.as_bytes());
    let res: Vec<(Option<&[u8]>, Option<&[u8]>)>; 
    match p {
        Done(_, o) => res = o,
        _ => { println!("Invalid hostlist: {:?}", p);
               panic!();
        }
    };
    let mut res2: Vec<(&str, BTreeMap<i32, i32>)> = Vec::new();
    for e in &res {
        let base = match e.0 {
            None => "",
            Some(o) => str::from_utf8(&o).unwrap(),
        };
        let range = range2tree(match e.1 {
            None => "",
            Some(i) => str::from_utf8(&i).unwrap(),
        });
        res2.push((base, range));
    }
    println!("res2: {:?}", res2);
    vec!["food".to_string()]
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
        Done(_, o) => str::from_utf8(&o[0].0.unwrap()).unwrap(),
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
        assert_eq!(expand("foo[1-3]"), vec!["foo1", "foo2", "foo3"]);
    }
}
