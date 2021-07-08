use clap::{crate_authors, crate_description, crate_version, App, AppSettings, Arg, SubCommand};
use std::collections::HashMap;

mod formatter;
mod iter;
mod linter;

fn main() {
    let matches = App::new("cniutil")
        .about(crate_description!())
        .version(crate_version!())
        .setting(AppSettings::GlobalVersion)
        .author(crate_authors!(", "))
        .setting(AppSettings::SubcommandRequiredElseHelp)
        .setting(AppSettings::UnifiedHelpMessage)
        // these arguments are available for all subcommands
        .arg(
            Arg::with_name("extension")
                .help("Enable CNI extensions.")
                .long("with")
                .possible_values(&["ini", "more-keys"])
                .case_insensitive(true)
                .multiple(true)
                .require_delimiter(true)
                .require_equals(true)
                .global(true)
        )
        .arg(
            Arg::with_name("removed-extension")
                .help("Disable CNI extensions.")
                .long("without")
                .possible_values(&["ini", "more-keys"])
                .case_insensitive(true)
                .multiple(true)
                .require_delimiter(true)
                .require_equals(true)
                .global(true)
        )
        .subcommand(
            SubCommand::with_name("lint")
                .setting(AppSettings::UnifiedHelpMessage)
                .about("comments on validity and style of CNI files")
                .arg(
                    Arg::with_name("FILES")
                        .help("The input files to read. '-' will result in stdin being read.")
                        .multiple(true)
                        .default_value("-")
                )
        )
        .subcommand(
        SubCommand::with_name("format")
                .setting(AppSettings::UnifiedHelpMessage)
                .visible_alias("fmt")
                .about("Reads in CNI files and shows the combined result in the specified format.")
                .arg(
                    Arg::with_name("cni")
                        .help("The output format should be CNI. Equivalent to --format=\"KEY = `VALUE`\". [default]")
                        .overrides_with_all(&["csv", "null", "format"])
                        .long("cni")
                )
                .arg(
                    Arg::with_name("threshold")
                        .help("Can only be used with --cni. Specifies the threshold of how many entries have to be in a section to make use of a section header. 0 means no section headers will be used. [default: 10]")
                        .long("section-threshold")
                        .short("n")
                        .validator(|arg| arg.parse::<usize>().map(|_| ()).map_err(|e| e.to_string()))
                        .requires("cni")
                )
                .arg(
                    Arg::with_name("csv")
                        .help("The output format should be comma separated values. Equivalent to --format=\"KEY,VALUE\"")
                        .overrides_with_all(&["cni", "null", "format"])
                        .long("csv")
                        .short("c")
                )
                .arg(
                    Arg::with_name("null")
                        .help("Records are terminated by a null character instead of a line feed to better accomodate values containing line feeds.")
                        .overrides_with_all(&["cni", "csv", "format"])
                        .long("null")
                        .short("0")
                )
                .arg(
                    Arg::with_name("format")
                        .help("Sets a custom format. KEY and VALUE are placeholders and may not occur more than once.")
                        .overrides_with_all(&["cni", "csv", "null"])
                        .long("format")
                        .takes_value(true)
                )
                .arg(
                    Arg::with_name("FILES")
                        .help("The input files to read. '-' will result in stdin being read.")
                        .multiple(true)
                        .default_value("-")
                )
        )
        .get_matches();

    // get enabled CNI extensions
    let opts = {
        let mut extensions = if matches.is_present("extension") {
            matches
                .values_of("extension")
                .unwrap()
                .zip(matches.indices_of("extension").unwrap())
                // removes duplicates
                .collect::<HashMap<_, _>>()
        } else {
            HashMap::new()
        };
        let removed_extensions = if matches.is_present("removed-extension") {
            matches
                .values_of("removed-extension")
                .unwrap()
                .zip(matches.indices_of("removed-extension").unwrap())
                // removes duplicates
                .collect::<HashMap<_, _>>()
        } else {
            HashMap::new()
        };

        for (removed, i) in removed_extensions {
            if matches!(extensions.get(removed), Some(&j) if j<i) {
                extensions.remove(removed);
            }
        }

        cni_format::Opts {
            ini: extensions.contains_key("ini"),
            more_keys: extensions.contains_key("more-keys"),
        }
    };

    match matches.subcommand() {
        ("lint", Some(matches)) => {
            let files = matches.values_of("FILES").unwrap();

            if files.len() == 1 {
                // don't show the filename if there is only one file
                linter::lint(&opts, matches.value_of("FILES").unwrap());
            } else {
                for file in files {
                    println!("{}", file);
                    linter::lint(&opts, file);
                }
            }
        }
        ("format", Some(matches)) => {
            use formatter::Format;

            let format = if matches.is_present("csv") {
                Format::Custom(Some("".into()), Some(",\"".into()), "\"\n".into())
            } else if matches.is_present("null") {
                Format::Custom(Some("".into()), Some("=".into()), "\0".into())
            } else if matches.is_present("format") {
                let format = format!("{}\n", matches.value_of("format").unwrap());
                let key_pos = format.find("KEY");
                let val_pos = format.find("VALUE");

                Format::Custom(
                    key_pos.map(|i| format[..i].into()),
                    val_pos.map(|i| format[key_pos.unwrap_or(0)..i].into()),
                    format[val_pos.or(key_pos).unwrap_or(0)..].into(),
                )
            } else {
                // must be the default CNI formatting

                // the unwrap is okay because of the validator in clap
                let section_threshold = matches.value_of("threshold").unwrap_or("10").parse().unwrap();
                Format::Cni(section_threshold)
            };

            formatter::format(matches.values_of("FILES").unwrap(), format, opts);
        }
        _ => unreachable!("unknown subcommand"),
    }
}
