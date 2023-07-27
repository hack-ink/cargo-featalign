// std
use std::fs;
// cargo-featalign
use crate::{
	analyzer::Analyzer,
	cli::{AnalyzerInitiator, IndentSymbol, Mode, ResolverInitiator, SharedInitiator},
	resolver::Resolver,
	shared::Shared,
};

#[test]
fn cargo_featalign_should_work() {
	Shared::initialize(SharedInitiator { thread: 32, mode: Mode::DryRun2 });
	Analyzer::initialize(AnalyzerInitiator {
		manifest_path: "mock".into(),
		features: ["std", "runtime-benchmarks", "try-runtime", "empty"]
			.iter()
			.copied()
			.map(Into::into)
			.collect(),
		workspace_only: true,
		default_std: true,
	})
	.analyze(-1);
	Resolver::initialize(ResolverInitiator { indent_symbol: IndentSymbol::Tab, indent_size: 4 })
		.resolve()
		.unwrap();

	["mock", "mock/nested/a", "mock/nested/b"].iter().for_each(|p| {
		let expect_p = format!("{}/Cargo.toml.expect", p);
		let dry_run_p = format!("{}/Cargo.toml.cargo-featalign.swap", p);

		let expect_s = fs::read_to_string(expect_p).unwrap();
		let dry_run_s = fs::read_to_string(&dry_run_p).unwrap();

		assert_eq!(expect_s, dry_run_s);

		fs::remove_file(dry_run_p).unwrap();
	});
}
