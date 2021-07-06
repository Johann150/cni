use std::collections::HashMap;
use std::io::Read;

/// Reads in all files given as arguments (or stdin if "-" is given).
/// At the end dumps the internal representation reached by gathering
/// all definitions.
fn main() {
    let mut args = std::env::args();
    // ignore executable path
    args.next();

    let mut map = HashMap::new();

    for file in args {
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

    println!("{:#?}", map);
}
