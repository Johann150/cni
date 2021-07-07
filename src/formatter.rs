use std::collections::HashMap;
use std::io::Read;
use std::fs::File;
use cni_format::CniExt;

pub struct Opts{
    pub section_threshold: usize,
}

pub fn format(file: &str, opts: &Opts) {
    let stream: Box<dyn Read> = if file == "-" {
        Box::new(std::io::stdin())
    } else {
        match File::open(&file) {
            Ok(f) => Box::new(f),
            Err(e) => {
                eprintln!("{:?}: {}", file, e);
                std::process::exit(1);
            }
        }
    };

    let stream = utf::decode_utf8(stream.bytes().filter_map(Result::ok)).filter_map(Result::ok);
    let mut map: HashMap<String, String> =
        match cni_format::CniParser::new(stream).collect() {
            Ok(map) => map,
            Err(e) => {
                eprintln!("{}", e);
                std::process::exit(1);
            }
        };

    // print the leaves in the top level
    println!("{}", cni_format::to_str(map.sub_leaves("")));

    let mut sections = map.sub_tree("").keys().cloned().collect::<Vec<_>>();
    sections.sort_unstable_by(|a, b|
        // long before short, then alphabetically
        a.len().cmp(&b.len()).reverse().then_with(|| a.cmp(b))
    );
    for section in sections {
        if map.sub_tree(&section).len() >= opts.section_threshold {
            println!("[{}]\n{}\n", section, cni_format::to_str(map.sub_tree(&section)));
            map.retain(|key, _| !key.starts_with(&format!("{}.", section)));
        }
    }

    println!("{}", cni_format::to_str(map));
}
