use std::io::Read;

/// Reads in a file given as argument (or stdin if "-" or none is given).
/// Produces an equivalent CNI representation of what was just read,
/// but formatted much more strictly. Most comments are removed.
#[cfg(feature = "serializer")]
fn main() {
    let mut args = std::env::args();
    // ignore executable path
    args.next();

    let file = args.next().unwrap_or("-".to_string());

    let stream: Box<dyn Read> = if file == "-" {
        Box::new(std::io::stdin())
    } else if let Ok(f) = std::fs::File::open(&file) {
        Box::new(f)
    } else {
        eprintln!("could not open file {:?}", file);
        std::process::exit(1);
    };

    let stream = utf::decode_utf8(stream.bytes().filter_map(Result::ok)).filter_map(Result::ok);
    let map: std::collections::HashMap<String, String> =
        match cni_format::CniParser::new(stream).collect() {
            Ok(map) => map,
            Err(e) => {
                eprintln!("{}", e);
                std::process::exit(1);
            }
        };

    println!("{}", cni_format::to_str(map));
}

#[cfg(not(feature = "serializer"))]
fn main() {
    eprintln!("This example has to be compiled with `--features serializer`.");
    std::process::exit(1);
}
