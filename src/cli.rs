pub use clap::Parser;

// std
use std::path::PathBuf;

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
	/// Root `Cargo.toml`'s path.
	///
	/// If `Cargo.toml` is not provided, it will be searched for under the specified path.
	#[arg(value_name = "PATH", default_value = "./Cargo.toml")]
	pub manifest_path: PathBuf,
	/// Features to process.
	#[arg(long, required = true, value_name = "[NAME]", value_delimiter = ',')]
	pub features: Vec<String>,
	/// Determines whether to process only workspace members.
	#[arg(long)]
	pub workspace_only: bool,
	/// Determines whether to check default features.
	///
	/// This option is useful when working in a no-std environment.
	#[arg(long)]
	pub default_std: bool,
	/// Depth of the dependency tree to process.
	///
	/// Use `-1` to process the entire tree.
	#[arg(long, value_name = "NUM", default_value_t = 0, allow_hyphen_values = true)]
	pub depth: i16,
	/// Number of threads to use.
	///
	/// The default value is based on the number of logical cores.
	#[arg(long, value_name = "NUM", default_value_t = num_cpus::get() as _, allow_hyphen_values = true)]
	pub thread: u16,
}
