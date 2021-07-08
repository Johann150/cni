use cni_format::CniExt;
use std::collections::HashMap;
use std::fs::File;
use std::io::Read;

pub enum Format {
    /// The section threshold, i.e. how many items before a section heading
    /// is used. Zero means no section headings should be used.
    Cni(usize),
    /// If the first is None, the key is not printed.
    /// If the second is None, the value is not printed.
    Custom(Option<String>, Option<String>, String),
}

fn print_cni(map: &HashMap<String, String>) {
    // don't use cni_format::to_str so there are no section headings
    for (key, value) in map {
        print!("{} = ", key);
        if value.is_empty() {
            println!("#empty");
        } else if value
            .contains(|c| c == '`' || c == '#' || c == ';' || crate::linter::is_vertical_ws(&c))
        {
            println!("`{}`", value.replace("`", "``"));
        } else {
            println!("{}", value);
        }
    }
}

pub fn format(files: clap::Values, format: Format, opts: cni_format::Opts) {
    let map = files
        .flat_map(|file| {
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

            let stream =
                utf::decode_utf8(stream.bytes().filter_map(Result::ok)).filter_map(Result::ok);
            cni_format::CniParser::new_opts(stream, opts)
        })
        .collect::<Result<HashMap<_, _>, _>>();

    if let Err(e) = map {
        eprintln!("{}", e);
        std::process::exit(1);
    }
    let mut map = map.unwrap();

    match format {
        Format::Cni(0) => print_cni(&map),
        Format::Cni(section_threshold) => {
            // print the leaves in the top level
            print_cni(&map.sub_leaves(""));
            map.retain(|key, _| key.contains('.'));

            let mut sections = map.section_tree("").iter().cloned().collect::<Vec<_>>();
            sections.sort_unstable_by(|a, b|
                // long before short, then alphabetically
                a.len().cmp(&b.len()).reverse().then_with(|| a.cmp(b)));
            for section in sections {
                if map.sub_tree(&section).len() >= section_threshold {
                    println!("[{}]", section);
                    print_cni(&map.sub_tree(&section));
                    map.retain(|key, _| !key.starts_with(&format!("{}.", section)));
                }
            }

            // print the remaining values
            print_cni(&map);
        }
        Format::Custom(pre, mid, post) => {
            for (key, value) in map {
                if let Some(ref pre) = pre {
                    print!("{}{}", key, pre);
                }
                if let Some(ref mid) = mid {
                    print!("{}{}", value, mid);
                }
                print!("{}", post);
            }
        }
    }
}
