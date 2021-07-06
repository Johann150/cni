use clap::{crate_authors, crate_description, crate_version, App, AppSettings, Arg, SubCommand};

mod dumper;
mod iter;
mod linter;

fn main() {
    let matches = App::new("cniutil")
		.about(crate_description!())
		.version(crate_version!())
		.setting(AppSettings::GlobalVersion)
		.author(crate_authors!(", "))
		.setting(AppSettings::SubcommandRequiredElseHelp)
		.subcommand(
			SubCommand::with_name("lint")
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
				.about("reads in CNI files and shows the combined result in the specified format")
				.arg(
					Arg::with_name("cni")
						.help("The output format should be CNI. (This is the default)")
						.overrides_with_all(&["csv", "prefix", "infix", "postfix"])
						.long("cni")
				)
				.arg(
					Arg::with_name("csv")
						.help("The output format should be comma separated values.")
						.overrides_with_all(&["cni", "prefix", "infix", "postfix"])
						.long("csv")
						.short("c")
				)
				.arg(
					Arg::with_name("prefix")
						.help("Specifies a custom line prefix. Can be used together with --infix and --postfix.")
						.overrides_with_all(&["cni", "csv"])
						.long("prefix")
						.value_name("prefix")
				)
				.arg(
					Arg::with_name("infix")
						.help("Specifies a custom line prefix. Can be used together with --prefix and --postfix.")
						.overrides_with_all(&["cni", "csv"])
						.long("infix")
						.value_name("infix")
				)
				.arg(
					Arg::with_name("postfix")
						.help("Specifies a custom line postfix, but a line feed will be added to the specified string. Can be used together with --prefix and --infix.")
						.overrides_with_all(&["cni", "csv", "postfix-nonl"])
						.long("postfix")
						.value_name("postfix")
				)
				.arg(
					Arg::with_name("postfix-nonl")
						.help("Specifies a custom line postfix, no lin feed will be added. Can be used together with --prefix and --infix.")
						.overrides_with_all(&["cni", "csv", "postfix"])
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
		.get_matches();

    match matches.subcommand() {
        ("lint", Some(matches)) => {
            let opts = linter::Opts::default();
            // TODO parse linter options

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
            let (prefix, infix, postfix) = if matches.is_present("csv") {
                ("", ",\"", "\"\n")
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

            dumper::dump(matches.values_of("FILES").unwrap(), prefix, infix, postfix);
        }
        _ => unreachable!("unknown subcommand"),
    }
}