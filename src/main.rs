use clap::{crate_authors, crate_description, crate_version, App, AppSettings, Arg, SubCommand};

mod dumper;
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
        // these arguments are available for all subcommands
        .arg(
            Arg::with_name("more-keys")
                .help("Enables the more-keys extension.")
                .overrides_with("no-more-keys")
                .long("more-keys")
                .global(true)
        )
        .arg(
            Arg::with_name("no-more-keys")
                .help("Disables the more-keys extension. [default]")
                .overrides_with("more-keys")
                .long("no-more-keys")
                .global(true)
        )
        .arg(
            Arg::with_name("ini")
                .help("Enables the ini compatibility extension.")
                .overrides_with("no-ini")
                .long("ini")
                .global(true)
        )
        .arg(
            Arg::with_name("no-ini")
                .help("Disables the ini compatibility extension. [default]")
                .overrides_with("ini")
                .long("no-ini")
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
            SubCommand::with_name("dump")
                .setting(AppSettings::UnifiedHelpMessage)
                .about("reads in CNI files and shows the combined result in the specified format")
                .arg(
                    Arg::with_name("cni")
                        .help("The output format should be CNI. [default]")
                        .overrides_with_all(&["csv", "null", "prefix", "infix", "postfix", "postfix-nonl"])
                        .long("cni")
                )
                .arg(
                    Arg::with_name("csv")
                        .help("The output format should be comma separated values.")
                        .overrides_with_all(&["cni", "null", "prefix", "infix", "postfix", "postfix-nonl"])
                        .long("csv")
                        .short("c")
                )
                .arg(
                    Arg::with_name("null")
                        .help("Records are terminated by a null character instead of a line feed to better accomodate values containing line feeds.")
                        .overrides_with_all(&["cni", "csv", "prefix", "infix", "postfix", "postfix-nonl"])
                        .long("null")
                        .short("0")
                )
                .arg(
                    Arg::with_name("prefix")
                        .help("Specifies a custom line prefix. Can be used together with --infix and --postfix.")
                        .overrides_with_all(&["cni", "csv", "null"])
                        .long("prefix")
                        .value_name("prefix")
                )
                .arg(
                    Arg::with_name("infix")
                        .help("Specifies a custom line prefix. Can be used together with --prefix and --postfix.")
                        .overrides_with_all(&["cni", "csv", "null"])
                        .long("infix")
                        .value_name("infix")
                )
                .arg(
                    Arg::with_name("postfix")
                        .help("Specifies a custom line postfix, but a line feed will be added to the specified string. Can be used together with --prefix and --infix.")
                        .overrides_with_all(&["cni", "csv", "null", "postfix-nonl"])
                        .long("postfix")
                        .value_name("postfix")
                )
                .arg(
                    Arg::with_name("postfix-nonl")
                        .help("Specifies a custom line postfix, no lin feed will be added. Can be used together with --prefix and --infix.")
                        .overrides_with_all(&["cni", "csv", "null", "postfix"])
                        .long("postfixx")
                        .value_name("postfix")
                )
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
                .alias("fmt")
                .about("Reads in a CNI file and outputs a strictly formatted representation of the input")
                .arg(
                    Arg::with_name("threshold")
                        .help("Specifies the threshold of how many entries have to be in a section to make use of a section header.")
                        .long("section-threshold")
                        .short("n")
                        .default_value("10")
                        .validator(|arg| arg.parse::<usize>().map(|_| ()).map_err(|e| e.to_string()))
                )
                .arg(
                    Arg::with_name("FILE")
                        .help("The input file to read. '-' will result in stdin being read.")
                        .default_value("-")
                )
        )
        .get_matches();

    match matches.subcommand() {
        ("lint", Some(matches)) => {
            let opts = cni_format::Opts {
                ini: matches.is_present("ini"),
                more_keys: matches.is_present("more-keys"),
            };

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
        ("dump", Some(matches)) => {
            let opts = cni_format::Opts {
                ini: matches.is_present("ini"),
                more_keys: matches.is_present("more-keys"),
            };

            let format = if matches.is_present("csv") {
                ("", ",\"", "\"\n")
            } else if matches.is_present("null") {
                ("", "=", "\0")
            } else if matches.is_present("prefix")
                || matches.is_present("infix")
                || matches.is_present("postfix")
                || matches.is_present("postfix-nonl")
            {
                (
                    matches.value_of("prefix").unwrap_or(""),
                    matches.value_of("infix").unwrap_or(" "),
                    matches
                        .value_of("postfix")
                        .or_else(|| matches.value_of("postfix-nonl"))
                        .unwrap_or("\n"),
                )
            } else {
                // must be the default CNI formatting
                ("", " = `", "`")
            };

            dumper::dump(matches.values_of("FILES").unwrap(), format, opts);
        }
        ("format", Some(matches)) => {
            let opts = cni_format::Opts {
                ini: matches.is_present("ini"),
                more_keys: matches.is_present("more-keys"),
            };

            // the first unwrap is okay because there is a default value in clap
            // the second unwrap is okay because of the validator in clap
            let section_threshold = matches.value_of("threshold").unwrap().parse().unwrap();

            formatter::format(matches.value_of("FILE").unwrap(), section_threshold, opts);
        }
        _ => unreachable!("unknown subcommand"),
    }
}
