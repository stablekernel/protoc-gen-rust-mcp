//! Emitting arbitrary text (tool and schema descriptions, the embedded JSON
//! Schema, proto comments...) as a valid Rust string literal. The
//! counterpart of Go's `quoteBacktickString`, which prefers a
//! backtick-quoted raw string unless the text itself contains a backtick.
//! Rust raw strings (`` r#"…"# ``) are the equivalent: readable for
//! embedded JSON and proto comments, and they don't need any escaping
//! except for extra `#`s when the text contains `"` followed by as many or
//! more `#`s than the string's delimiter uses.
//!
//! Unlike Go backtick strings, Rust raw strings do allow any character
//! except a bare (non-`\n`-preceded) `\r`, which `rustc` rejects outright
//! (<https://doc.rust-lang.org/reference/tokens.html#raw-string-literals>).
//! That one case falls back to an escaped (non-raw) string literal.

/// Returns a Rust string literal for `s` that evaluates back to exactly
/// `s`: a raw string `r#"…"#` with just enough `#`s to not be confused by
/// any `"` in `s`, or, for the one case a raw string can never express (a
/// bare `\r` not immediately followed by `\n`), a normal escaped string
/// literal.
///
/// Round-trips through `syn::parse_str::<syn::LitStr>` (see the
/// `rust_literal_round_trips_through_syn` test below and its property-test
/// variant), and through `rustc` itself: every literal it emits is valid
/// and semantically equivalent Rust source.
pub fn rust_string_literal(s: &str) -> String {
    if has_bare_carriage_return(s) {
        return escaped_string_literal(s);
    }
    let hashes = "#".repeat(raw_string_hash_count(s));
    format!("r{hashes}\"{s}\"{hashes}")
}

/// A bare `\r` (one not immediately followed by `\n`) is the one character
/// a Rust raw string literal can never contain: `rustc` rejects it even
/// though that same byte is allowed, unescaped, inside a normal string
/// literal written directly in source.
fn has_bare_carriage_return(s: &str) -> bool {
    let mut chars = s.chars().peekable();
    while let Some(c) = chars.next() {
        if c == '\r' && chars.peek() != Some(&'\n') {
            return true;
        }
    }
    false
}

/// The number of `#`s a raw string needs to safely enclose `s`: one more
/// than the longest run of consecutive `#` that immediately follows a `"`
/// in `s` (a raw string `r###"…"###` is only terminated by `"` followed by
/// *at least* three `#`, so three is enough whenever no such run reaches
/// three; zero `#`s are enough whenever `s` has no `"` at all).
fn raw_string_hash_count(s: &str) -> usize {
    let bytes = s.as_bytes();
    let mut max_run = 0usize;
    let mut i = 0usize;
    while i < bytes.len() {
        if bytes[i] == b'"' {
            let mut run = 0usize;
            let mut j = i + 1;
            while j < bytes.len() && bytes[j] == b'#' {
                run += 1;
                j += 1;
            }
            max_run = max_run.max(run);
        }
        i += 1;
    }
    if max_run == 0 && !s.contains('"') {
        0
    } else {
        max_run + 1
    }
}

/// A normal (non-raw) Rust string literal for `s`, escaping only what Rust
/// requires: `"`, `\`, and control characters that aren't allowed to
/// appear literally in source.
fn escaped_string_literal(s: &str) -> String {
    let mut out = String::with_capacity(s.len() + 2);
    out.push('"');
    for c in s.chars() {
        match c {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            '\0' => out.push_str("\\0"),
            c if (c as u32) < 0x20 => out.push_str(&format!("\\u{{{:x}}}", c as u32)),
            c => out.push(c),
        }
    }
    out.push('"');
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn round_trips(s: &str) {
        let literal = rust_string_literal(s);
        let parsed: syn::LitStr = syn::parse_str(&literal)
            .unwrap_or_else(|e| panic!("{literal:?} (for input {s:?}) failed to parse: {e}"));
        assert_eq!(&parsed.value(), s, "round-trip mismatch for input {s:?}");
    }

    #[test]
    fn plain_text_uses_a_raw_string() {
        assert_eq!(rust_string_literal("abc"), "r\"abc\"");
    }

    #[test]
    fn empty_string() {
        assert_eq!(rust_string_literal(""), "r\"\"");
        round_trips("");
    }

    #[test]
    fn backtick_stays_in_a_raw_string() {
        // Go's quoteBacktickString falls back to strconv.Quote whenever the
        // text contains a backtick (it uses backticks as its own raw-string
        // delimiter); a Rust raw string has no such problem, since its
        // delimiter is `"`, not `` ` ``, so a backtick needs no special
        // handling at all.
        assert_eq!(rust_string_literal("a`b"), "r\"a`b\"");
        round_trips("a`b");
    }

    #[test]
    fn quote_needs_one_hash() {
        assert_eq!(rust_string_literal("a\"b"), "r#\"a\"b\"#");
        round_trips("a\"b");
    }

    #[test]
    fn quote_hash_needs_two_hashes() {
        assert_eq!(rust_string_literal("a\"#b"), "r##\"a\"#b\"##");
        round_trips("a\"#b");
    }

    #[test]
    fn quote_followed_by_more_hashes_elsewhere_takes_the_max() {
        let s = "first \"# then \"## then a lone \" ";
        round_trips(s);
    }

    #[test]
    fn backslash_and_path_like_text() {
        round_trips("C:\\vibes\\new");
    }

    #[test]
    fn multiline_text() {
        round_trips("first line\nsecond line\n");
    }

    #[test]
    fn crlf_is_fine_in_a_raw_string() {
        assert_eq!(rust_string_literal("a\r\nb"), "r\"a\r\nb\"");
        round_trips("a\r\nb");
    }

    #[test]
    fn bare_carriage_return_falls_back_to_an_escaped_literal() {
        let literal = rust_string_literal("a\rb");
        assert!(
            !literal.starts_with('r'),
            "expected an escaped literal, got {literal:?}"
        );
        round_trips("a\rb");
    }

    #[test]
    fn unicode_text() {
        round_trips("emoji \u{1F389} and unicode \u{e9}");
    }

    #[test]
    fn the_example_protos_set_vibe_block_comment() {
        // The exact text processed from SetVibe's block comment in
        // examples/protos/example.proto (after process_comment_to_string):
        // quotes, a backtick and a Windows-style path with backslashes.
        round_trips(
            "This is a block comment with multiple lines to test block handling \"Hello World\", a `backtick`, and a path like C:\\vibes\\new",
        );
    }

    // Property test: for 2,000 pseudo-random strings built out of the
    // characters most likely to matter (quotes, hashes, backslashes,
    // backticks, control characters, above-ASCII and empty runs), the
    // literal rust_string_literal emits always round-trips through
    // syn::parse_str::<syn::LitStr> back to the original string.
    #[test]
    fn rust_literal_round_trips_through_syn() {
        let alphabet: Vec<char> = vec![
            '"', '#', '\\', '`', 'a', 'b', ' ', '\n', '\r', '\t', '\0', '\u{1}', '🎉', 'é',
        ];
        let mut state: u64 = 0x2545_F491_4F6C_DD1D;
        let mut next = move || {
            // xorshift64*, good enough for a deterministic test fixture.
            state ^= state >> 12;
            state ^= state << 25;
            state ^= state >> 27;
            state.wrapping_mul(0x2545_F491_4F6C_DD1D)
        };

        for _case in 0..2000 {
            let len = (next() % 12) as usize;
            let s: String = (0..len)
                .map(|_| alphabet[(next() as usize) % alphabet.len()])
                .collect();
            round_trips(&s);
        }
    }
}
