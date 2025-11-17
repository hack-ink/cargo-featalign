mod prelude {
	pub(crate) use anyhow::Result;

	pub(crate) use crate::{shared, util};
}

mod cli;
use cli::{Cli, Mode, Parser};

mod analyzer;
use analyzer::{Analyzer, PROBLEMS};

mod sorter;

mod resolver;
use resolver::Resolver;

mod shared;
use shared::{MODE, Shared};

mod util;

#[cfg(test)] mod test;

// std
use std::{env, process};

fn main() -> prelude::Result<()> {
	color_eyre::install().map_err(|e| anyhow::anyhow!(e))?;

	let mut args = env::args();

	if let Some("featalign") = env::args().nth(1).as_deref() {
		args.next();
	}

	let Cli { shared_initiator, analyzer_initiator, depth, resolver_initiator, verbose } =
		Cli::parse_from(args);
	let mut exit_code = 0;

	Shared::initialize(shared_initiator);
	Analyzer::initialize(analyzer_initiator).analyze(depth);

	let problems = PROBLEMS.lock().unwrap();
	let mode = MODE.get().unwrap();

	if verbose || matches!(mode, Mode::Check) {
		println!("{}", serde_json::to_string(&*problems).unwrap());
	}
	if !problems.is_empty() && matches!(mode, Mode::Check | Mode::DryRun | Mode::DryRun2) {
		exit_code = -1;
	}

	drop(problems);

	Resolver::initialize(resolver_initiator).resolve()?;

	process::exit(exit_code);
}
