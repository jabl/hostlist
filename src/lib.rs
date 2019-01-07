/*
    Copyright (C) 2016-2018  Janne Blomqvist

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



use std::str;
use nom::*;
use nom::types::CompleteByteSlice;


// A name component part of a hostlist, before the hostlist syntax begins
named!(hostname_part<CompleteByteSlice, CompleteByteSlice>, take_while!(|ch| ch != b'['));

// A hostlist list expressions, the stuff within []. E.g. 1,2,5-6,9
named!(listexpr<CompleteByteSlice,
       Vec<(CompleteByteSlice, Option<CompleteByteSlice>)> >,
       separated_nonempty_list!(
           char!(','),
           tuple!(take_while!(is_digit),
                  opt!(preceded!(char!('-'),
                                 take_while!(is_digit))
                  )
           )
       )
);

// A range (something enclosed with [])
named!(range<CompleteByteSlice,
       Vec<(CompleteByteSlice, Option<CompleteByteSlice>)> >,
       delimited!(char!('['), listexpr, char!(']'))
);

// hostname-range pair
named!(hnrangepair<CompleteByteSlice,
       (Option<CompleteByteSlice>,
        Option<Vec<(CompleteByteSlice, Option<CompleteByteSlice>)> >) >,
       tuple!(
           opt!(hostname_part), opt!(range))
);

// A complete hostlist, e.g. foo[1-3]
named!(hostlist<CompleteByteSlice,
       Vec<(Option<CompleteByteSlice>,
       Option<Vec<(CompleteByteSlice, Option<CompleteByteSlice>)> >) >>,
       many1!(hnrangepair)
);


// Expand a hostlist to a vector of hostnames
pub fn expand(a_str: &str) -> Result<Vec<String>, &'static str> {
    let p = hostlist(CompleteByteSlice(a_str.as_bytes()));
    let parsed = match p {
        Ok((_, o)) => o,
        _ => return Err("Invalid hostlist")
    };
    let mut res: Vec<String> = Vec::new();
    for e in &parsed {
        let base = match e.0 {
            None => "",
            Some(o) => str::from_utf8(&o).unwrap(),
        };
        let r = match &e.1 {
            Some(o) => o,
            None => return Err("Invalid hostrange"),
        };
        for r2 in r {
            let idx = str::from_utf8(&r2.0).unwrap();
            res.push(format!("{}{}", base, idx));
            match r2.1 {
                // An upper part of a range
                Some(u) => {
                    let idxu: i32 = str::from_utf8(&u).unwrap().parse().unwrap();
                    let idxl: i32 = idx.parse().unwrap();
                    for i in idxl .. idxu {
                        res.push(format!("{}{}", base, i + 1));
                    }
                }
                None => continue
            }
        }
    }
    Ok(res)
}


// Tests of private functions
#[test]
fn check_base() {
    let hostlist = b"foo[1-3]";
    let res = hostname_part(CompleteByteSlice(hostlist));
    let out = match res {
        Ok((_, o)) => str::from_utf8(&o).unwrap(),
        _ => panic!()
    };
    assert_eq!(out, "foo");
}

#[test]
fn listexpr_1() {
    let le = b"1";
    let res = listexpr(CompleteByteSlice(le));
    let out = match res {
        Ok((_, o)) => str::from_utf8(&o[0].0).unwrap(),
        _ => panic!()
    };
    assert_eq!(out, "1");
}

#[test]
fn listexpr_2() {
    let le = b"1,2,3-5";
    let res = listexpr(CompleteByteSlice(le));
    let out = match res {
        Ok((_, o)) => o,
        _ => panic!()
    };
    assert_eq!(str::from_utf8(&out[0].0).unwrap(), "1");
    assert_eq!(str::from_utf8(&out[1].0).unwrap(), "2");
    assert_eq!(str::from_utf8(&out[2].0).unwrap(), "3");
    assert_eq!(str::from_utf8(&out[2].1.unwrap()).unwrap(), "5");
}


#[test]
fn hostrange() {
    let hostlist = b"[1,2,3-5]";
    let res = range(CompleteByteSlice(hostlist));
    let out = match res {
        Ok((_, o)) => o,
        _ => { println!("{:?}", res);
               panic!();
        }
    };
    assert_eq!(str::from_utf8(&out[0].0).unwrap(), "1");
    assert_eq!(str::from_utf8(&out[1].0).unwrap(), "2");
    assert_eq!(str::from_utf8(&out[2].0).unwrap(), "3");
    assert_eq!(str::from_utf8(&out[2].1.unwrap()).unwrap(), "5");
}

#[test]
fn hnrangepair_1() {
    let hostlist = b"foo[1,2,3-5]";
    let res = hnrangepair(CompleteByteSlice(hostlist));
    let out = match res {
        Ok((_, o)) => o,
        _ => { println!("{:?}", res);
               panic!();
        }
    };
    assert_eq!(str::from_utf8(&out.0.unwrap()).unwrap(), "foo");
    let r = &out.1.unwrap();
    assert_eq!(str::from_utf8(&r[0].0).unwrap(), "1");
    assert_eq!(str::from_utf8(&r[1].0).unwrap(), "2");
    assert_eq!(str::from_utf8(&r[2].0).unwrap(), "3");
    assert_eq!(str::from_utf8(&r[2].1.unwrap()).unwrap(), "5");
}

#[test]
fn hnrangepair_hostonly() {
    let hostlist = b"foo";
    let res = hnrangepair(CompleteByteSlice(hostlist));
    let out = match res {
        Ok((_, o)) => str::from_utf8(&o.0.unwrap()).unwrap(),
        _ => { println!("{:?}", res);
               panic!();
        }
    };
    assert_eq!(out, "foo");
}

#[test]
fn hnrangepair_rangeonly() {
    let hostlist = b"[1,2,3-5]";
    let res = hnrangepair(CompleteByteSlice(hostlist));
    let out = match res {
        Ok((_, o)) => o,
        _ => { println!("{:?}", res);
               panic!();
        }
    };
    let r = &out.1.unwrap();
    assert_eq!(str::from_utf8(&r[0].0).unwrap(), "1");
    assert_eq!(str::from_utf8(&r[1].0).unwrap(), "2");
    assert_eq!(str::from_utf8(&r[2].0).unwrap(), "3");
    assert_eq!(str::from_utf8(&r[2].1.unwrap()).unwrap(), "5");
}

#[test]
fn hostlist_1() {
    let myhl = b"foo[1,2,3-5]";
    let res = hostlist(CompleteByteSlice(myhl));
    let out = match res {
        Ok((_, o)) => o,
        _ => { println!("{:?}", res);
               panic!();
        }
    };
    assert_eq!(str::from_utf8(&out[0].0.unwrap()).unwrap(), "foo");
    let r = &out[0].1.as_ref().unwrap();
    assert_eq!(str::from_utf8(&r[0].0).unwrap(), "1");
    assert_eq!(str::from_utf8(&r[1].0).unwrap(), "2");
    assert_eq!(str::from_utf8(&r[2].0).unwrap(), "3");
    assert_eq!(str::from_utf8(&r[2].1.unwrap()).unwrap(), "5");
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
        assert_eq!(expand("foo[1,2,3]").unwrap(),
                   vec!["foo1", "foo2", "foo3"]);
        assert_eq!(expand("foo[1-3]").unwrap(),
                   vec!["foo1", "foo2", "foo3"]);
    }
}
