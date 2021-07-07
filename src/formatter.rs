use cni_format::CniExt;
use std::collections::HashMap;
use std::fs::File;
use std::io::Read;

pub fn format(file: &str, section_threshold: usize, opts: cni_format::Opts) {
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
        match cni_format::CniParser::new_opts(stream, opts).collect() {
            Ok(map) => map,
            Err(e) => {
                eprintln!("{}", e);
                std::process::exit(1);
            }
        };

    // print the leaves in the top level
    print!("{}", cni_format::to_str(map.sub_leaves("")));
    map.retain(|key, _| key.contains('.'));

    let mut sections = map.sub_tree("").keys().cloned().collect::<Vec<_>>();
    sections.sort_unstable_by(|a, b|
        // long before short, then alphabetically
        a.len().cmp(&b.len()).reverse().then_with(|| a.cmp(b)));
    for section in sections {
        if map.sub_tree(&section).len() >= section_threshold {
            print!(
                "[{}]\n{}",
                section,
                cni_format::to_str(map.sub_tree(&section))
            );
            map.retain(|key, _| !key.starts_with(&format!("{}.", section)));
        }
    }

    // don't use cni_format::to_str here so there are no section headings
    for (key, value) in map {
        print!("{} = ", key);
        if value.is_empty() {
            println!("#empty");
        } else if value.contains(|c| c=='`'||c=='#'||c==';'||crate::linter::is_vertical_ws(&c)) {
            println!("`{}`", value.replace("`", "``"));
        } else {
            println!("{}", value);
        }
    }
}
