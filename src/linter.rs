use crate::iter::Iter;
use cni_format::Opts;
use std::io::Read;

// character classification

/// implements Perl's / Raku's "\v", i.e. vertical white space
pub fn is_vertical_ws(c: &char) -> bool {
    matches!(
        c,
        '\n' | '\u{B}' | '\u{C}' | '\r' | '\u{85}' | '\u{2028}' | '\u{2029}'
    )
}

fn is_key(c: &char, opts: &Opts) -> bool {
    if opts.more_keys {
        !(matches!(c, '[' | ']' | '=' | '`' | '#') || (opts.ini && *c == ';') || c.is_whitespace())
    } else {
        matches!(c, '0'..='9'|'a'..='z'|'A'..='Z'|'-'|'_'|'.')
    }
}

fn is_value(c: &char, opts: &Opts) -> bool {
    !(*c == '#' || (opts.ini && *c == ';') || is_vertical_ws(c))
}

// tokens

fn skip_comment(iter: &mut Iter) {
    while matches!(iter.peek(), Some(c) if !is_vertical_ws(c)) {
        iter.next();
    }
    // also skip over the linebreak
    iter.next();
}

fn skip_ws(iter: &mut Iter) {
    while matches!(iter.peek(), Some(c) if c.is_whitespace()) {
        iter.next();
    }
}

fn check_key(iter: &mut Iter, opts: &Opts) {
    let mut pseudo_raw = None;

    if iter.peek() == Some(&'.') {
        println!(
            "{0}:{1}-{0}:{2} syntax error: A key or section heading can not start with a dot.",
            iter.line,
            iter.col,
            iter.col + 1
        );
    } else if iter.peek() == Some(&'`') {
        pseudo_raw = Some((iter.line, iter.col));
        iter.next();
    }

    while let Some(c) = iter.peek().copied() {
        if !is_key(&c, opts) {
            break;
        }

        iter.next();
        if matches!(iter.peek(), Some(x) if !is_key(x, opts)) && c == '.' {
            println!(
                "{0}:{1}-{0}:{2} syntax error: A key or section heading can not end with a dot.",
                iter.line,
                iter.col,
                iter.col + 1,
            );
        }
    }

    if let Some((line, col)) = pseudo_raw {
        if iter.peek() == Some(&'`') {
            iter.next();
        }
        println!(
            "{}:{}-{}:{} syntax error: A key or section heading can not be a raw value.",
            line, col, iter.line, iter.col
        );
    } else if iter.peek() == Some(&'`') {
        iter.next();
        println!(
            "{0}:{1}-{0}:{2} syntax error: A key or section heading can not be a raw value.",
            iter.line,
            iter.col,
            iter.col + 1
        );
    }
}

// main linter parser

pub fn lint(opts: &Opts, path: &str) {
    let src = if path == "-" {
        let mut buffer = String::new();
        match std::io::stdin().read_to_string(&mut buffer) {
            Ok(_bytes) => buffer,
            Err(e) => {
                eprintln!("cannot process stdin: {}", e);
                return;
            }
        }
    } else {
        match std::fs::read_to_string(path) {
            Ok(src) => src,
            Err(e) => {
                eprintln!("cannot process {}: {}", path, e);
                return;
            }
        }
    }
    // because we do not have to faithfully represent the result, its easier
    // to replace CRLF with just LF, than dealing with CRLF everywhere
    .replace("\r\n", "\n");

    let mut iter = Iter::new(&src);

    loop {
        match iter.peek() {
            None => break,
            Some(c) if c.is_whitespace() => {
                // don't report empty lines as unnecessary whitespace
                while matches!(iter.peek(), Some(c) if is_vertical_ws(c)) {
                    iter.next();
                }

                let (line, col) = (iter.line, iter.col);
                while let Some(c) = iter.peek() {
                    if is_vertical_ws(c) {
                        iter.next();
                        // maybe this is the last line of the whitespace
                        if matches!(iter.peek(), Some(c) if !c.is_whitespace()) {
                            // before advancing the position, show the end here
                            println!(
                                "{}:{}-{}:{} info: unnecessary whitespace",
                                line, col, iter.line, iter.col
                            );
                        }
                    } else if !c.is_whitespace() {
                        break;
                    }
                    iter.next();
                }
            }
            Some('#') => skip_comment(&mut iter),
            Some(';') if opts.ini => skip_comment(&mut iter),
            Some(']') => {
                iter.next();
                println!(
                    "{0}:{1}-{0}:{2} syntax error: Unexpected closing square bracket.",
                    iter.line,
                    iter.col,
                    iter.col + 1
                )
            }
            Some('[') => {
                iter.next();
                let start = (iter.line, iter.col);
                // ending locations of various possible items
                let mut whitespace_before = None; // also the start of the comment before
                let mut comment_before = None;
                let mut word = None;
                let mut whitespace_after = None; // also the start of the comment after
                let mut comment_after = None;

                skip_ws(&mut iter);
                if start != (iter.line, iter.col) {
                    whitespace_before = Some((iter.line, iter.col));
                }

                // leading comment(s)
                while iter.peek() == Some(&'#') || (opts.ini && iter.peek() == Some(&';')) {
                    // skip over comment symbol
                    iter.next();

                    // skip to end of line
                    while matches!(iter.peek(), Some(c) if !is_vertical_ws(c)) {
                        iter.next();
                    }
                    comment_before = Some((iter.line, iter.col));

                    // skip over any whitespace (linebreak and at the start of the next line)
                    skip_ws(&mut iter);
                }
                // do not report on the comment yet, maybe the heading is broken

                // this must be the start of the actual section header
                check_key(&mut iter, opts);

                if comment_before.or(whitespace_before).unwrap_or(start) != (iter.line, iter.col) {
                    word = Some((iter.line, iter.col));
                }

                // trailing whitespace
                skip_ws(&mut iter);
                if word
                    .or(comment_before)
                    .or(whitespace_before)
                    .unwrap_or(start)
                    != (iter.line, iter.col)
                {
                    whitespace_after = Some((iter.line, iter.col));
                }

                // trailing comments
                while iter.peek() == Some(&'#') || (opts.ini && iter.peek() == Some(&';')) {
                    // skip over comment symbol
                    iter.next();

                    // skip to end of line
                    while matches!(iter.peek(), Some(c) if !is_vertical_ws(c)) {
                        iter.next();
                    }
                    comment_after = Some((iter.line, iter.col));

                    // skip over any whitespace (linebreak and at the start of the next line)
                    skip_ws(&mut iter);
                }
                // do not report on the comment yet, maybe the heading is broken

                if iter.next() == Some(']') {
                    // heading terminated properly
                    // now output warnings

                    if word.is_none() {
                        // comment_after and whitespace_after must also be None

                        if comment_before.is_none() {
                            println!(
                                "{}:{}-{}:{} info: This section heading only contains a comment, is this intentional?",
                                start.0,
                                start.1,
                                iter.line,
                                iter.col,
                            );
                        } else {
                            let start = whitespace_before.unwrap_or(start);
                            println!(
                                "{}:{}-{}:{} info: This section heading is empty. You can avoid empty section headings by putting items in this section at the start of the file.",
                                start.0,
                                start.1,
                                iter.line,
                                iter.col,
                            );
                        }
                    }

                    if let Some(end) = comment_before {
                        // maybe this was commented by mistake
                        let start = whitespace_before.unwrap_or(start);
                        println!(
                            "{}:{}-{}:{} info: This is not a good place to put a comment, consider putting it before the section heading.",
                            start.0,
                            start.1,
                            end.0,
                            end.1,
                        );
                    } else if let Some(end) = whitespace_before {
                        if end.0 != start.0 {
                            // there is a linebreak at the start of the section heading
                            println!(
                                "{}:{}-{}:{} info: A line break here may be confusing.",
                                start.0, start.1, end.0, end.1,
                            );
                        }
                    }

                    if let Some(end) = comment_after {
                        let start = whitespace_after
                            .or(word)
                            .expect("Detected a comment after a nonexistent section heading.");
                        println!(
                            "{}:{}-{}:{} info: This is not a good place to put a comment, consider putting it after the section heading.",
                            start.0,
                            start.1,
                            end.0,
                            end.1,
                        );
                    } else if let Some(end) = whitespace_after {
                        let start =
                            word.expect("Detected whitespace afer a nonexisten section heading.");
                        if end.0 != start.0 {
                            // there is a linebreak at the end of the section heading
                            println!(
                                "{}:{}-{}:{} info: A line break here may be confusing.",
                                start.0, start.1, end.0, end.1,
                            );
                        }
                    }
                } else {
                    println!(
                        "{0}:{1}-{0}:{2} syntax error: Expected ']' for end of section heading.",
                        iter.line,
                        iter.col,
                        iter.col + 1
                    );
                }
            }
            // backtick is not actually a key, but looks like someone tried to
            // put a raw value for a key so this path will produce the appropriate error messages
            Some(c) if is_key(&c, opts) || c == &'`' => {
                check_key(&mut iter, opts);

                {
                    let end_key = (iter.line, iter.col);
                    skip_ws(&mut iter);

                    if iter.peek() != Some(&'=') {
                        println!(
                            "{0}:{1}-{0}:{2} syntax error: Expected '=' after key.",
                            end_key.0,
                            end_key.1,
                            end_key.1 + 1,
                        );
                    }
                    iter.next(); // skip over equal sign
                }

                skip_ws(&mut iter);

                if iter.peek() == Some(&'`') {
                    // raw value
                    let start = (iter.line, iter.col);

                    iter.next(); // skip over backtick

                    // trying to detect where a missing backtick could be
                    let mut last_key = None;
                    let mut detected_stmt = None;

                    while let Some(c) = iter.peek() {
                        if c == &'=' {
                            // do not overwrite previous detected statements
                            if detected_stmt.is_none() && last_key.is_some() {
                                detected_stmt = last_key;
                            }
                            last_key = None;
                        } else if c == &'`' {
                            iter.next();
                            last_key = None;
                            if iter.peek() != Some(&'`') {
                                // not an escaped backtick
                                if matches!(iter.peek(), Some(c) if is_value(c, opts) && !c.is_whitespace())
                                {
                                    println!(
                                        "{0}:{1}-{0}:{2} syntax error: Unescaped backtick inside raw value. Use '``' to represent a backtick in a raw value.",
                                        iter.line, iter.col, iter.col+1
                                    );
                                } else {
                                    // this backtick terminates the raw value
                                    break;
                                }
                            }
                        } else if is_key(c, opts) {
                            // keep the start position of this key
                            if last_key.is_none() {
                                last_key = Some((iter.line, iter.col));
                            }
                        } else if c.is_whitespace() {
                            // this could be the whitespace between the key and
                            // the equal sign
                            skip_ws(&mut iter);
                            if iter.peek() != Some(&'=') {
                                // it was not
                                last_key = None;
                            }
                            // do not advance iterator at end of loop
                            continue;
                        } else if !is_key(c, opts) {
                            // this cant be the last key
                            last_key = None;
                        }
                        iter.next();
                    }

                    if iter.peek().is_none() {
                        println!(
                            "{}:{}-{}:{} syntax error: Expected '`' at end of raw value.",
                            start.0, start.1, iter.line, iter.col
                        );
                        if let Some((line, col)) = detected_stmt {
                            println!(
                                "{0}:{1}-{0}:{2} info: This looks like a new statement, did you forget to put a backtick here?",
                                line, col, col+1
                            );
                        }
                    }
                } else {
                    // non-raw value
                    while matches!(iter.peek(), Some(c) if is_value(c, &opts)) {
                        iter.next();
                    }
                }
            }
            Some('=') => {
                println!(
                    "{0}:{1}-{0}:{2} syntax error: Expected key before '='.",
                    iter.line,
                    iter.col,
                    iter.col + 1,
                );
                iter.next();
            }
            _ => {
                println!(
                    "{0}:{1}-{0}:{2} syntax error: Expected key and '=' before value.",
                    iter.line,
                    iter.col,
                    iter.col + 1,
                );
                // so we do not generate an error for every char on this line,
                // just pretend it is a comment
                skip_comment(&mut iter);
            }
        }
    }
}
