/*
MIT License

Copyright (c) 2016-2019 Janne Blomqvist

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

use nom::*;
use nom::character::is_digit;
use nom::bytes::complete::{take_while, take_while1};
use nom::bytes::complete::tag;
use nom::sequence::delimited;
use nom::sequence::preceded;
use nom::sequence::tuple;
use nom::sequence::pair;
use nom::combinator::opt;
use nom::multi::many0;
use nom::multi::separated_nonempty_list;
use std::str;

// A name component part of a hostlist, before the hostlist syntax begins
//named!(hostname_part<&str, &str>, take_while!(|ch| ch != '['));
fn hostname_part(input: &[u8]) -> IResult<&[u8], &[u8]>
{
    let hpart = take_while1(|ch| ch != b'[');
    hpart(input)
}

// take_digits taken from https://github.com/badboy/iso8601 (MIT
// licensed)
fn take_digits(i: &[u8]) -> IResult<&[u8], u32> {
    let (i, digits) = take_while(is_digit)(i)?;

    if digits.is_empty() {
        return Err(nom::Err::Error((i, nom::error::ErrorKind::Eof)));
    }

    let s = str::from_utf8(digits).expect("Invalid data, expected UTF-8 string");
    let res = s
        .parse()
        .expect("Invalid string, expected ASCII representation of a number");

    Ok((i, res))
}

// A hostlist list expressions, the stuff within []. E.g. 1,2,5-6,9
/*
named!(listexpr<&str,
       Vec<(&str, Option<&str>)> >,
       separated_nonempty_list!(
           char!(','),
           tuple!(take_while!(is_digit),
                  opt!(preceded!(char!('-'),
                                 take_while!(is_digit))
                  )
           )
       )
);
 */
fn listexpr(input: &[u8]) -> IResult<&[u8], Vec<(u32, Option<u32>)> >
{
    let digits = take_digits;
    let range = tuple((&digits, opt(preceded(tag("-"), &digits))));
    let snl = separated_nonempty_list(tag(","), range);
    snl(input)
}

// A range (something enclosed with [])
/*named!(range<&str,
       Vec<(&str, Option<&str>)> >,
       delimited!(char!('['), listexpr, char!(']'))
); */
fn range(input: &[u8]) -> IResult<&[u8], Vec<(u32, Option<u32>)> >
{
    let r = delimited(tag("["), listexpr, tag("]"));
    r(input)
}

// hostname-range pair
/*named!(hnrangepair<&str,
       (Option<&str>,
        Option<Vec<(&str, Option<&str>)> >) >,
       tuple!(
           opt!(hostname_part), opt!(range))
);*/
fn hnrangepair(input: &[u8]) -> IResult<&[u8],
                                        (&[u8],
                                         Option<Vec<(u32, Option<u32>)>>)>
{
    let t = pair(hostname_part, opt(range));
    t(input)
}

// A complete hostlist, e.g. foo[1-3]
/*named!(hostlist<&str,
       Vec<(Option<&str>,
       Option<Vec<(&str, Option<&str>)> >) >>,
       many1!(hnrangepair)
);*/
fn hostlist(input: &[u8]) -> IResult<&[u8],
                                     Vec<Vec<(&[u8],
                                          Option<Vec<(u32, Option<u32>)>>)>>>
{
    let m = many0(hnrangepair);
    let snl = separated_nonempty_list(tag(","), m);
    snl(input)
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
    let mut res: Vec<String> = Vec::new();
    for e in &parsed {
        let base = str::from_utf8(&e[0].0).unwrap();
        let r = match &e[0].1 {
            Some(o) => o,
            None => {
                res.push(base.to_string());
                continue;
            }
        };
        for r2 in r {
            let idx = r2.0;
            res.push(format!("{}{}", base, idx));
            match r2.1 {
                // An upper part of a range
                Some(u) => {
                    let idxu = u;
                    for i in idx..idxu {
                        res.push(format!("{}{}", base, i + 1));
                    }
                }
                None => continue,
            }
        }
    }
    Ok(res)
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
        Ok((_, o)) => o[0].0,
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
    assert_eq!(out[0].0, 1);
    assert_eq!(out[1].0, 2);
    assert_eq!(out[2].0, 3);
    assert_eq!(out[2].1.unwrap(), 5);
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
    assert_eq!(out[0].0, 1);
    assert_eq!(out[1].0, 2);
    assert_eq!(out[2].0, 3);
    assert_eq!(out[2].1.unwrap(), 5);
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
    let r = &out.1.unwrap();
    assert_eq!(r[0].0, 1);
    assert_eq!(r[1].0, 2);
    assert_eq!(r[2].0, 3);
    assert_eq!(r[2].1.unwrap(), 5);
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
    let r = &out.1.unwrap();
    assert_eq!(r[0].0, 1);
    assert_eq!(r[1].0, 2);
    assert_eq!(r[2].0, 3);
    assert_eq!(r[2].1.unwrap(), 5);
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
    let r = &out[0][0].1.as_ref().unwrap();
    assert_eq!(r[0].0, 1);
    assert_eq!(r[1].0, 2);
    assert_eq!(r[2].0, 3);
    assert_eq!(r[2].1.unwrap(), 5);
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
    assert_eq!(out[0].0, b"");
}
*/

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
