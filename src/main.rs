mod prelude {
	pub use anyhow::Result;
}

mod cli;
use cli::{Cli, Parser};

mod analyzer;
use analyzer::{Analyzer, PROBLEMS};

mod resolver;

mod util;

fn main() -> prelude::Result<()> {
	color_eyre::install().map_err(|e| anyhow::anyhow!(e))?;

	let cli = Cli::parse();
	let depth = cli.depth;

	Analyzer::from_cli(cli).analyze(depth);

	println!("{}", serde_json::to_string(&*PROBLEMS.lock().unwrap()).unwrap());

	resolver::resolve()?;

	Ok(())
}
