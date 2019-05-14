// Copyright (c) 2019 King's College London created by the Software Development Team
// <http://soft-dev.org/>
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0>, or the MIT license <LICENSE-MIT or
// http://opensource.org/licenses/MIT>, or the UPL-1.0 license <http://opensource.org/licenses/UPL>
// at your option. This file may not be copied, modified, or distributed except according to those
// terms.

const WILDCARD: &'static str = "...";

/// Does `s` conform to the fuzzy pattern `pattern`? Note that `plines` is expected not to start or
/// end with blank lines, and each line is expected to be `trim`ed.
pub(crate) fn match_vec(plines: &Vec<&str>, s: &str) -> bool {
    let slines = s.trim().lines().map(|x| x.trim()).collect::<Vec<_>>();

    let mut pi = 0;
    let mut si = 0;

    while pi < plines.len() && si < slines.len() {
        if plines[pi] == WILDCARD {
            pi += 1;
            if pi == plines.len() {
                return true;
            }
            if plines[pi] == WILDCARD {
                panic!("Can't have '{}' on two consecutive lines.", WILDCARD);
            }
            while si < slines.len() {
                if match_line(plines[pi], slines[si]) {
                    break;
                }
                si += 1;
            }
            if si == slines.len() {
                return false;
            }
        } else if match_line(plines[pi], slines[si]) {
            pi += 1;
            si += 1;
        } else {
            return false;
        }
    }
    true
}

fn match_line(p: &str, s: &str) -> bool {
    (p.starts_with(WILDCARD) && s.ends_with(&p[WILDCARD.len()..])) || p == s
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_match_vec() {
        fn match_vec_helper(p: &str, s: &str) -> bool {
            match_vec(&p.lines().collect::<Vec<_>>(), s)
        }
        assert!(match_vec_helper("", ""));
        assert!(match_vec_helper("", "\n"));
        assert!(match_vec_helper("\n", "\n"));
        assert!(match_vec_helper("a", "a"));
        assert!(match_vec_helper("a", "a"));
        assert!(match_vec_helper("...\na", "a"));
        assert!(match_vec_helper("...\na\n...", "a"));
        assert!(match_vec_helper("a\n...", "a"));
        assert!(match_vec_helper("a\n...\nd", "a\nd"));
        assert!(match_vec_helper("a\n...\nd", "a\nb\nc\nd"));
        assert!(!match_vec_helper("a\n...\nd", "a\nb\nc"));
        assert!(match_vec_helper("a\n...\nc\n...\ne", "a\nb\nc\nd\ne"));
        assert!(match_vec_helper("a\n...\n...b", "a\nb"));
    }
}
