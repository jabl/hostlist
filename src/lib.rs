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
