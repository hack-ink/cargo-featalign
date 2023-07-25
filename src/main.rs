mod prelude {
	pub use anyhow::Result;
}

mod cli;
use cli::{Cli, Parser};

mod processor;
use processor::{Processor, PROBLEMS};

mod util;

fn main() -> prelude::Result<()> {
	color_eyre::install().map_err(|e| anyhow::anyhow!(e))?;

	let Cli { manifest_path, features, workspace_only, default_std, depth, thread } = Cli::parse();

	Processor::analyze(
		&util::manifest_path_of(&manifest_path),
		features,
		workspace_only,
		default_std,
		thread,
	)
	.process(depth);

	println!("{}", serde_json::to_string(&*PROBLEMS.lock().unwrap()).unwrap());

	Ok(())
}
