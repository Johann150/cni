use std::collections::HashMap;
use std::io::Read;

/// Reads in all files given as arguments (or stdin if "-" is given).
/// At the end dumps the internal representation reached by gathering
/// all definitions.
pub fn dump<'a>(files: clap::Values<'a>, prefix: &str, infix: &str, postfix: &str) {
    let mut map = HashMap::new();

    for file in files {
        let stream: Box<dyn Read> = if file == "-" {
            Box::new(std::io::stdin())
        } else if let Ok(f) = std::fs::File::open(&file) {
            Box::new(f)
        } else {
            eprintln!("could not open file {:?}", file);
            continue;
        };
        let stream = utf::decode_utf8(stream.bytes().filter_map(Result::ok)).filter_map(Result::ok);
        let parser = cni_format::CniParser::new(stream);

        for res in parser {
            match res {
                Ok((key, value)) => {
                    map.insert(key, value);
                }
                Err(e) => {
                    eprintln!("{}", e);
                    std::process::exit(1);
                }
            }
        }
    }

    for (key, value) in map {
        print!("{}{}{}{}{}", prefix, key, infix, value, postfix);
    }
}
