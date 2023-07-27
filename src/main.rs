mod prelude {
	pub(crate) use anyhow::Result;

	pub(crate) use crate::{shared, util};
}

mod cli;
use cli::{Cli, Mode, Parser};

mod analyzer;
use analyzer::{Analyzer, PROBLEMS};

mod resolver;
use resolver::Resolver;

mod shared;
use shared::{Shared, MODE};

mod util;

#[cfg(test)] mod test;

// std
use std::env;

fn main() -> prelude::Result<()> {
	color_eyre::install().map_err(|e| anyhow::anyhow!(e))?;

	let mut args = env::args();

	if let Some("featalign") = env::args().nth(1).as_deref() {
		args.next();
	}

	let Cli { shared_initiator, analyzer_initiator, depth, resolver_initiator, verbose } =
		Cli::parse_from(args);

	Shared::initialize(shared_initiator);
	Analyzer::initialize(analyzer_initiator).analyze(depth);

	if verbose || *MODE.get().unwrap() == Mode::Check {
		println!("{}", serde_json::to_string(&*PROBLEMS.lock().unwrap()).unwrap());
	}

	Resolver::initialize(resolver_initiator).resolve()?;

	Ok(())
}
