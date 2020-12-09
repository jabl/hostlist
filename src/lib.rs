/*
MIT License

Copyright (c) 2016-2020 Janne Blomqvist

Permission is hereby granted, free of charge, to any person obtaining
a copy of this software and associated documentation files (the
"Software"), to deal in the Software without restriction, including
without limitation the rights to use, copy, modify, merge, publish,
distribute, sublicense, and/or sell copies of the Software, and to
permit persons to whom the Software is furnished to do so, subject to
the following conditions:

The above copyright notice and this permission notice shall be
included in all copies or substantial portions of the Software.

THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND,
EXPRESS OR IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF
MERCHANTABILITY, FITNESS FOR A PARTICULAR PURPOSE AND
NONINFRINGEMENT. IN NO EVENT SHALL THE AUTHORS OR COPYRIGHT HOLDERS BE
LIABLE FOR ANY CLAIM, DAMAGES OR OTHER LIABILITY, WHETHER IN AN ACTION
OF CONTRACT, TORT OR OTHERWISE, ARISING FROM, OUT OF OR IN CONNECTION
WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE SOFTWARE.

 */

use nom::bytes::complete::tag;
use nom::bytes::complete::{take_while, take_while1};
use nom::character::is_digit;
use nom::combinator::opt;
use nom::multi::many0;
use nom::multi::separated_list1;
use nom::sequence::delimited;
use nom::sequence::pair;
use nom::sequence::preceded;
use nom::sequence::tuple;
use nom::*;
use std::str;

struct DigitInfo {
    value: u32,
    leading_zeros: usize,
    num_digits: usize,
}

#[derive(Debug)]
struct RangeList {
    ranges: Vec<(u32, u32)>,
    num_digits: usize,
}

// A name component part of a hostlist, before the hostlist syntax begins
fn hostname_part(input: &[u8]) -> IResult<&[u8], &[u8]> {
    let hpart = take_while1(|ch| (ch != b'[' && ch != b','));
    hpart(input)
}

// take_digits taken from https://github.com/badboy/iso8601 (MIT
// licensed)
fn take_digits(i: &[u8]) -> IResult<&[u8], DigitInfo> {
    let (i, digits) = take_while(is_digit)(i)?;

    if digits.is_empty() {
        return Err(nom::Err::Error(nom::error::Error {
            input: i,
            code: nom::error::ErrorKind::Eof,
        }));
    }

    let s = str::from_utf8(digits).expect("Invalid data, expected UTF-8 string");
    let res = s
        .parse()
        .expect("Invalid string, expected ASCII representation of a number");

    let mut clz = 0;
    for &c in digits {
        if c == b'0' {
            clz += 1;
        } else {
            break;
        }
    }
    let di = DigitInfo {
        value: res,
        leading_zeros: clz,
        num_digits: digits.len(),
    };
    Ok((i, di))
}

// A hostlist list expressions, the stuff within []. E.g. 1,2,5-6,9
fn listexpr(input: &[u8]) -> IResult<&[u8], RangeList> {
    let digits = take_digits;
    let range = tuple((&digits, opt(preceded(tag("-"), &digits))));
    let mut snl = separated_list1(tag(","), range);
    let (i, les) = snl(input)?;
    let mut ri = RangeList {
        ranges: Vec::new(),
        num_digits: 0,
    };
    let mut max_lz = 0;
    for le in les {
        if le.0.leading_zeros > max_lz {
            max_lz = le.0.leading_zeros;
            ri.num_digits = le.0.num_digits;
        }
        let mut vals = (le.0.value, le.0.value);
        match le.1 {
            Some(u) => {
                if u.value >= le.0.value {
                    vals.1 = u.value;
                } else {
                    vals = (u.value, le.0.value);
                }
                if u.leading_zeros > max_lz {
                    max_lz = u.leading_zeros;
                    ri.num_digits = u.num_digits;
                }
            }
            None => {}
        }
        ri.ranges.push(vals);
    }
    Ok((i, ri))
}

// A range (something enclosed with [])
fn range(input: &[u8]) -> IResult<&[u8], RangeList> {
    let mut r = delimited(tag("["), listexpr, tag("]"));
    r(input)
}

// hostname-ranges pair, e.g. foo[N-M][NN-MM]
fn hnrangepair(input: &[u8]) -> IResult<&[u8], (&[u8], Vec<RangeList>)> {
    let mut t = pair(hostname_part, many0(range));
    t(input)
}

// A complete hostlist, e.g. foo[1-3]bar[4-5][5-6],baz[1-3]
fn hostlist(input: &[u8]) -> IResult<&[u8], Vec<Vec<(&[u8], Vec<RangeList>)>>> {
    let m = many0(hnrangepair);
    let mut snl = separated_list1(tag(","), m);
    snl(input)
}

// Cartesian multiplication of strings
fn cartesian<T: AsRef<str> + ToString>(v1: &[T], v2: &[T]) -> Vec<String> {
    let oldsz = v1.len();
    let mut res = Vec::with_capacity(oldsz * v2.len());
    for e1 in v1 {
        for e2 in v2 {
            // TODO: Is this dance really needed to concatenate two &[T]'s?
            let mut t: String = e1.to_string();
            t.push_str(e2.to_string().as_str());
            res.push(t);
        }
    }
    res
}

/// Expand a hostlist to a vector of hostnames
///
/// # Examples
///
/// ```
/// extern crate hostlist;
/// assert_eq!(hostlist::expand("foo[1-3]").unwrap(),
///            vec!["foo1", "foo2", "foo3"]);
/// ```
pub fn expand(a_str: &str) -> Result<Vec<String>, &'static str> {
    let p = hostlist(a_str.as_bytes());
    let parsed = match p {
        Ok((_, o)) => o,
        _ => return Err("Invalid hostlist"),
    };
    let mut allres = Vec::new();
    for e in &parsed {
        let mut res: Vec<String> = vec!["".to_string()];
        for rangepair in e {
            let base = str::from_utf8(&rangepair.0).unwrap();
            let mut res2 = vec![base.to_string()];
            for range in &rangepair.1 {
                let mut res3: Vec<String> = Vec::new();
                for r2 in &range.ranges {
                    for i in r2.0..(r2.1 + 1) {
                        // {:08} - field width 8, pad with zeros at front
                        res3.push(format!("{:0width$}", i, width = range.num_digits));
                    }
                }
                res2 = cartesian(&res2, &res3);
            }
            res = cartesian(&res, &res2);
        }
        for host in res {
            allres.push(host);
        }
    }
    Ok(allres)
}

// Tests of private functions
#[test]
fn check_base() {
    let hostlist = b"foo[1-3]";
    let res = hostname_part(hostlist);
    let out = match res {
        Ok((_, o)) => str::from_utf8(&o).unwrap(),
        _ => panic!(),
    };
    assert_eq!(out, "foo");
}

#[test]
fn listexpr_1() {
    let le = b"1";
    let res = listexpr(le);
    let out = match res {
        Ok((_, o)) => o.ranges[0].0,
        _ => panic!(),
    };
    assert_eq!(out, 1);
}

#[test]
fn listexpr_2() {
    let le = b"1,2,3-5";
    let res = listexpr(le);
    let out = match res {
        Ok((_, o)) => o,
        _ => panic!(),
    };
    assert_eq!(out.ranges[0].0, 1);
    assert_eq!(out.ranges[1].0, 2);
    assert_eq!(out.ranges[2].0, 3);
    assert_eq!(out.ranges[2].1, 5);
}

#[test]
fn hostrange() {
    let hostlist = b"[1,2,3-5]";
    let res = range(hostlist);
    let out = match res {
        Ok((_, o)) => o,
        _ => {
            println!("{:?}", res);
            panic!();
        }
    };
    assert_eq!(out.ranges[0].0, 1);
    assert_eq!(out.ranges[1].0, 2);
    assert_eq!(out.ranges[2].0, 3);
    assert_eq!(out.ranges[2].1, 5);
}

/*
#[test]
fn hnrangepair_empty() {
    let hostlist = b"";
    let res = hnrangepair(hostlist);
    let out = match res {
        Ok((_, o)) => o,
        _ => {
            println!("{:?}", res);
            panic!();
        }
    };
    assert_eq!(str::from_utf8(&out.0).unwrap(), "");
}
*/

#[test]
fn hnrangepair_1() {
    let hostlist = b"foo[1,2,3-5]";
    let res = hnrangepair(hostlist);
    let out = match res {
        Ok((_, o)) => o,
        _ => {
            println!("{:?}", res);
            panic!();
        }
    };
    assert_eq!(str::from_utf8(&out.0).unwrap(), "foo");
    let r = &out.1[0];
    assert_eq!(r.ranges[0].0, 1);
    assert_eq!(r.ranges[1].0, 2);
    assert_eq!(r.ranges[2].0, 3);
    assert_eq!(r.ranges[2].1, 5);
}

#[test]
fn hnrangepair_hostonly() {
    let hostlist = b"foo";
    let res = hnrangepair(hostlist);
    let out = match res {
        Ok((_, o)) => str::from_utf8(&o.0).unwrap(),
        _ => {
            println!("{:?}", res);
            panic!();
        }
    };
    assert_eq!(out, "foo");
}

/*
#[test]
fn hnrangepair_rangeonly() {
    let hostlist = b"[1,2,3-5]";
    let res = hnrangepair(hostlist);
    let out = match res {
        Ok((_, o)) => o,
        _ => {
            println!("{:?}", res);
            panic!();
        }
    };
    let r = &out.1;
    assert_eq!(r[0].ranges[0].0, 1);
    //assert_eq!(r[1].0, 2);
    //assert_eq!(r[2].0, 3);
    //assert_eq!(r[2].1.unwrap(), 5);
}
*/

#[test]
fn hostlist_1() {
    let myhl = b"foo[1,2,3-5]";
    let res = hostlist(myhl);
    let out = match res {
        Ok((_, o)) => o,
        _ => {
            println!("{:?}", res);
            panic!();
        }
    };
    assert_eq!(str::from_utf8(&out[0][0].0).unwrap(), "foo");
    let r = &out[0][0].1[0];
    assert_eq!(r.ranges[0].0, 1);
    assert_eq!(r.ranges[1].0, 2);
    assert_eq!(r.ranges[2].0, 3);
    assert_eq!(r.ranges[2].1, 5);
}

/*
#[test]
fn hostlist_empty() {
    let res = hostlist(b"");
    let out = match res {
        Ok((_, o)) => o,
        _ => {
            println!("{:?}", res);
            panic!();
        }
    };
    assert_eq!(out[0][0].0, b"");
}
*/

#[test]
fn test_cartesian() {
    let a = vec!["ab", "c"];
    let b = vec!["1", "23"];
    let r = cartesian(&a, &b);
    assert_eq!("ab1", r[0]);
    assert_eq!("ab23", r[1]);
    assert_eq!("c1", r[2]);
    assert_eq!("c23", r[3]);
    assert_eq!(4, r.len());
}

// Tests of public functions
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {}

    #[test]
    fn test_expand() {
        assert_eq!(expand("foo[1,2,3]").unwrap(), vec!["foo1", "foo2", "foo3"]);
        assert_eq!(expand("foo[1-3]").unwrap(), vec!["foo1", "foo2", "foo3"]);
        assert_eq!(expand("foo").unwrap(), vec!["foo"]);
    }

    #[test]
    fn test_full_name_with_comma_works() {
        assert_eq!(
            expand("hostname1.foo.com,hostname2.foo.com").unwrap(),
            vec!["hostname1.foo.com", "hostname2.foo.com"]
        );
    }

    #[test]
    fn test_trailing_parts() {
        assert_eq!(
            expand("hostname1.foo.com").unwrap(),
            vec!["hostname1.foo.com"]
        );
    }

    #[test]
    fn test_single_host_expansion() {
        assert_eq!(
            expand("hostname[6].foo.com").unwrap(),
            vec!["hostname6.foo.com"]
        )
    }

    #[test]
    fn test_prefix_expansion() {
        assert_eq!(
            expand("hostname[009-011]").unwrap(),
            vec!["hostname009", "hostname010", "hostname011"]
        );
    }

    #[test]
    fn test_reverse_order() {
        assert_eq!(
            expand("hostname[7-5]").unwrap(),
            vec!["hostname5", "hostname6", "hostname7"],
        );
    }

    #[test]
    fn test_single_item_two_ranges() {
        assert_eq!(
            expand("hostname[6,7]-[9-11].foo.com").unwrap(),
            vec![
                "hostname6-9.foo.com",
                "hostname6-10.foo.com",
                "hostname6-11.foo.com",
                "hostname7-9.foo.com",
                "hostname7-10.foo.com",
                "hostname7-11.foo.com"
            ]
        );
    }
}
