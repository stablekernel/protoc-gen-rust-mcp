//! Turning a proto leading comment into a tool or schema description, and a
//! camelCase identifier into a space-separated fallback description. Ports
//! of Go's `processCommentToString` and `camelToSpace`
//! (`cmd/protoc-gen-go-mcp/mcp.go` at commit `9e072f9`).

/// Turns a (possibly multi-line, possibly block-style) leading proto
/// comment into a single-line string with its raw text otherwise
/// untouched: callers are responsible for escaping it for whatever they
/// embed it in (a Rust string literal via [`crate::rust_literal`], a JSON
/// Schema `"description"` via `serde_json`), so this never escapes quotes
/// or backslashes itself - doing so here would corrupt the text when a
/// caller's own encoding escapes it a second time (see Go issue #100).
///
/// `comments` is the raw text from a `SourceCodeInfo.Location`'s
/// `leading_comments` (or a `leading_detached_comments` entry): for a line
/// comment (`// foo`), that's the source text between consecutive `//`
/// markers, including each continuation line's own `// ` prefix and a
/// trailing newline; for a block comment (`/* foo */`), it's the comment
/// text with the `/* ` prefix and ` */` suffix still attached, and no
/// per-line marker.
pub fn process_comment_to_string(comments: &str) -> String {
    let mut comment_text = comments;
    comment_text = comment_text.strip_prefix("// ").unwrap_or(comment_text);
    comment_text = comment_text.strip_prefix("/* ").unwrap_or(comment_text);
    comment_text = comment_text.strip_suffix(" */").unwrap_or(comment_text);

    // Join every line (dropping any remaining "// " line-comment prefix)
    // into a single space-separated line.
    let joined = comment_text
        .split('\n')
        .map(|line| line.strip_prefix("// ").unwrap_or(line))
        .collect::<Vec<_>>()
        .join(" ");

    let trimmed = joined.trim();

    // Collapse runs of whitespace (including the spaces just introduced
    // for blank comment lines) into a single space.
    collapse_whitespace_runs(trimmed)
}

/// Collapses every run of one or more whitespace characters in `s` into a
/// single ASCII space, the counterpart of Go's
/// `whitespaceRunRegexp.ReplaceAllString(s, " ")`.
fn collapse_whitespace_runs(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    let mut in_run = false;
    for c in s.chars() {
        if c.is_whitespace() {
            if !in_run {
                out.push(' ');
                in_run = true;
            }
        } else {
            out.push(c);
            in_run = false;
        }
    }
    out
}

/// Turns a camelCase or PascalCase identifier into a space-separated
/// string, the fallback description for a method with no leading comment
/// (e.g. `SetVibeDetails` -> `Set Vibe Details`). A port of Go's
/// `camelToSpace`, including its exact (regex-`FindAll`-style)
/// non-overlapping matching: it inserts a space at every
/// lowercase-to-uppercase letter boundary and every letter-to-digit or
/// digit-to-letter boundary, but once two characters are consumed as one
/// boundary match, the second of the two is never re-examined as the
/// start of the next match. That means, for example, `Vibe2Array` (a
/// letter-digit boundary immediately followed by a digit-letter boundary)
/// only gets one space, not two: `Vibe 2Array`, matching Go's behavior
/// byte for byte.
pub fn camel_to_space(s: &str) -> String {
    let chars: Vec<char> = s.chars().collect();
    let mut out = String::with_capacity(s.len() + 4);
    let mut i = 0usize;
    while i + 1 < chars.len() {
        let a = chars[i];
        let b = chars[i + 1];
        let boundary = (a.is_ascii_lowercase() && b.is_ascii_uppercase())
            || (a.is_ascii_alphabetic() && b.is_ascii_digit())
            || (a.is_ascii_digit() && b.is_ascii_alphabetic());
        if boundary {
            out.push(a);
            out.push(' ');
            out.push(b);
            i += 2;
        } else {
            out.push(a);
            i += 1;
        }
    }
    if i < chars.len() {
        out.push(chars[i]);
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    // Ports of Go's TestProcessCommentToString
    // (`cmd/protoc-gen-go-mcp/mcp_test.go`), covering the two bugs in Go
    // issue #100: a backslash or backtick in a proto comment must come
    // through completely untouched, and a double quote must likewise come
    // through unescaped rather than backslash-escaped here (which used to
    // make json.Marshal double-escape it when schema.go embedded the
    // result in a JSON Schema "description").
    #[test]
    fn process_comment_to_string_quote() {
        assert_eq!(
            process_comment_to_string(" Must match \"chill\" exactly.\n"),
            "Must match \"chill\" exactly."
        );
    }

    #[test]
    fn process_comment_to_string_backslash() {
        assert_eq!(
            process_comment_to_string(" Must match \\d+.\n"),
            "Must match \\d+."
        );
    }

    #[test]
    fn process_comment_to_string_backtick() {
        assert_eq!(
            process_comment_to_string(" Use the `vibe` field.\n"),
            "Use the `vibe` field."
        );
    }

    #[test]
    fn process_comment_to_string_multiline_joins_with_a_single_space() {
        assert_eq!(
            process_comment_to_string(" First line.\n Second line.\n"),
            "First line. Second line."
        );
    }

    #[test]
    fn process_comment_to_string_block_comment() {
        assert_eq!(
            process_comment_to_string("/* Block \"comment\" with \\backslash\\. */"),
            "Block \"comment\" with \\backslash\\."
        );
    }

    // The example proto's tricky comments (examples/protos/example.proto),
    // checked against the descriptions in Go's committed
    // examples/gen/example/v1/example_mcp.pb.go. `raw` below is exactly
    // what protoc's SourceCodeInfo.leading_comments contains for SetVibe's
    // `/** ... */`-style block comment (confirmed against real protoc
    // output via protox; see also the end-to-end parity test in
    // tests/example_proto_descriptions.rs, which compiles the real
    // example.proto and exercises this same comment through this same
    // function): protoc itself already strips the per-line `* ` markers,
    // leaving only the outer `/* `/` */` delimiters for this function to
    // strip.

    #[test]
    fn process_comment_to_string_set_vibe_block_comment() {
        let raw = "\n This is a block comment\n with multiple lines\n to test block handling\n \"Hello World\", a `backtick`, and a path like C:\\vibes\\new\n";
        assert_eq!(
            process_comment_to_string(raw),
            "This is a block comment with multiple lines to test block handling \"Hello World\", a `backtick`, and a path like C:\\vibes\\new"
        );
    }

    #[test]
    fn process_comment_to_string_get_vibe_multiline_with_trailing_empty_line() {
        let raw = " Get Vibe\n of the server\n\n";
        assert_eq!(process_comment_to_string(raw), "Get Vibe of the server");
    }

    #[test]
    fn process_comment_to_string_set_vibe_details_single_line() {
        assert_eq!(
            process_comment_to_string(" Set vibe details\n"),
            "Set vibe details"
        );
    }

    #[test]
    fn process_comment_to_string_empty_is_empty() {
        assert_eq!(process_comment_to_string(""), "");
    }

    // Ports of Go's camelToSpace cases exercised through the fallback
    // description.
    #[test]
    fn camel_to_space_pascal_case_method_name() {
        assert_eq!(camel_to_space("SetVibeDetails"), "Set Vibe Details");
    }

    #[test]
    fn camel_to_space_single_word() {
        assert_eq!(camel_to_space("Vibe"), "Vibe");
    }

    #[test]
    fn camel_to_space_lower_to_upper_boundary() {
        assert_eq!(camel_to_space("myValue"), "my Value");
    }

    #[test]
    fn camel_to_space_letter_to_digit_boundary() {
        assert_eq!(camel_to_space("item123"), "item 123");
    }

    #[test]
    fn camel_to_space_digit_to_letter_boundary() {
        assert_eq!(camel_to_space("123item"), "123 item");
    }

    #[test]
    fn camel_to_space_mixed_boundaries() {
        // Go's underlying regexp scans non-overlapping 2-character windows
        // left to right: once "tV" is consumed as one boundary match, the
        // next window starts at "ib", so the "e2" boundary further along
        // still gets its own space, but a would-be "2A" boundary
        // immediately after it does not (its first character was already
        // consumed by the "e2" match).
        assert_eq!(camel_to_space("GetVibe2Array"), "Get Vibe 2Array");
    }

    #[test]
    fn camel_to_space_empty() {
        assert_eq!(camel_to_space(""), "");
    }
}
