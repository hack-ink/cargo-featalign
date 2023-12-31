pub use clap::Parser;

// std
use std::path::PathBuf;
// crates.io
use clap::ValueEnum;

#[derive(Debug, Parser)]
#[command(
	version = concat!(
		env!("CARGO_PKG_VERSION"),
		"-",
		env!("VERGEN_GIT_SHA"),
		"-",
		env!("VERGEN_CARGO_TARGET_TRIPLE"),
	),
	about,
	rename_all = "kebab",
)]
pub struct Cli {
	#[command(flatten)]
	pub shared_initiator: SharedInitiator,

	#[command(flatten)]
	pub analyzer_initiator: AnalyzerInitiator,
	/// Depth of the dependency tree to process.
	///
	/// Use `-1` to process the entire tree.
	///
	/// !! Running with this flag under a large project, even with 128 threads configured, is
	/// incredibly challenging.
	#[arg(long, value_name = "NUM", default_value_t = 0, allow_hyphen_values = true)]
	pub depth: i16,

	#[command(flatten)]
	pub resolver_initiator: ResolverInitiator,

	/// Verbose output.
	#[arg(long)]
	pub verbose: bool,
}

#[derive(Debug, Parser)]
pub struct SharedInitiator {
	/// Features to process.
	#[arg(long, required = true, value_name = "[NAME]", value_delimiter = ',')]
	pub features: Vec<String>,
	/// Number of threads to use.
	///
	/// The default value is based on the number of logical cores.
	#[arg(long, value_name = "NUM", default_value_t = num_cpus::get() as _, allow_hyphen_values = true)]
	pub thread: u16,
	/// Running mode.
	///
	/// Check: Prints the analysis result.
	/// DryRun: Prints the resolved result without modifying the `Cargo.toml` file.
	/// DryRun2: creates a `*.cargo-featalign.swap` file.
	/// Overwrite: Overwrites the original `Cargo.toml` file.
	#[arg(long, value_enum, verbatim_doc_comment, default_value_t = Mode::Overwrite)]
	pub mode: Mode,
	/// Use the given symbol for indentation.
	#[arg(long, value_enum, default_value_t = IndentSymbol::Tab)]
	pub indent_symbol: IndentSymbol,
	/// The number of spaces used for indentation.
	#[arg(long, value_name = "SIZE", default_value_t = 4)]
	pub indent_size: usize,
}

#[derive(Debug, Parser)]
pub struct AnalyzerInitiator {
	/// Root `Cargo.toml`'s path.
	///
	/// If `Cargo.toml` is not provided, it will be searched for under the specified path.
	#[arg(value_name = "PATH", default_value = "./Cargo.toml")]
	pub manifest_path: PathBuf,
	/// Determines whether to process only workspace members.
	#[arg(long)]
	pub workspace_only: bool,
	/// Disregard the analysis of the specified crates.
	#[arg(long, value_delimiter = ',')]
	pub ignore: Vec<String>,
	/// Determines whether to check default features.
	///
	/// This option is useful when working in a no-std environment.
	/// This feature checks if you have set `default-features = false` while also having a `std =
	/// ["x/std"]` part to control it separately.
	#[arg(long)]
	pub default_std: bool,
	// Specify the crates here that do not use the default to control their `std` feature. This
	// will prevent the check from being applied to them, which is only useful when enabling the
	// `default-std`.
	#[arg(long, value_delimiter = ',')]
	pub non_default_std: Vec<String>,
}

#[derive(Debug, Parser)]
pub struct ResolverInitiator {
	/// Wether to sort the required features while aligning.
	#[arg(long)]
	pub sort: bool,
}
#[derive(Clone, Debug, ValueEnum)]
pub enum IndentSymbol {
	Tab,
	Whitespace,
}
#[derive(Clone, Debug, PartialEq, ValueEnum)]
pub enum Mode {
	Check,
	DryRun,
	DryRun2,
	Overwrite,
}
