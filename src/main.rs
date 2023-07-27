mod prelude {
	pub(crate) use anyhow::Result;

	pub(crate) use crate::shared;
}

mod cli;
use cli::{Cli, Parser};

mod analyzer;
use analyzer::{Analyzer, PROBLEMS};

mod resolver;
use resolver::Resolver;

mod shared;
use shared::Shared;

mod util;

fn main() -> prelude::Result<()> {
	color_eyre::install().map_err(|e| anyhow::anyhow!(e))?;

	let Cli { shared_initiator, analyzer_initiator, depth, resolver_initiator } = Cli::parse();

	Shared::initialize(shared_initiator);
	Analyzer::initialize(analyzer_initiator).analyze(depth);

	println!("{}", serde_json::to_string(&*PROBLEMS.lock().unwrap()).unwrap());

	Resolver::initialize(resolver_initiator).resolve()?;

	Ok(())
}
