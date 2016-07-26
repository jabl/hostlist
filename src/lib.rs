/*
  Copyright (C) 2016 Janne Blomqvist

  This Source Code Form is subject to the terms of the Mozilla Public
  License, v. 2.0. If a copy of the MPL was not distributed with this
  file, You can obtain one at http://mozilla.org/MPL/2.0/.
*/

// Expand a hostlist to a vector of hostnames
pub fn expand(hostlist: &str) -> Vec<String> {
    // Is this a hostlist at all?
    let baseend = match  hostlist.find('[') {
        None => return vec![hostlist.to_string()],
        Some(i) => i,
    };
    vec![hostlist[0..baseend].to_string()]
}

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
