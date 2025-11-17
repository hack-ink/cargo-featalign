// std
use std::fs;
// crates.io
use toml_edit::{DocumentMut, visit_mut::VisitMut};
// cargo-featalign
use crate::{
	analyzer::Analyzer,
	cli::{AnalyzerInitiator, IndentSymbol, Mode, ResolverInitiator, SharedInitiator},
	resolver::Resolver,
	shared::Shared,
	sorter::SortVisitor,
};

#[test]
fn cargo_featalign_should_work() {
	Shared::initialize(SharedInitiator {
		features: ["std", "runtime-benchmarks", "try-runtime", "empty"]
			.iter()
			.copied()
			.map(Into::into)
			.collect(),
		thread: 32,
		mode: Mode::DryRun2,
		indent_symbol: IndentSymbol::Tab,
		indent_size: 4,
	});
	Analyzer::initialize(AnalyzerInitiator {
		manifest_path: "mock".into(),
		workspace_only: true,
		default_std: true,
		ignore: Vec::new(),
		non_default_std: Vec::new(),
	})
	.analyze(-1);
	Resolver::initialize(ResolverInitiator { sort: true }).resolve().unwrap();

	["mock", "mock/nested/a", "mock/nested/b"].iter().for_each(|p| {
		let expect_p = format!("{}/Cargo.toml.expect", p);
		let dry_run_p = format!("{}/Cargo.toml.cargo-featalign.swap", p);

		let expect_s = fs::read_to_string(expect_p).unwrap();
		let dry_run_s = fs::read_to_string(&dry_run_p).unwrap();

		assert_eq!(expect_s, dry_run_s);

		fs::remove_file(dry_run_p).unwrap();
	});
}

#[test]
fn sort_visitor_should_work() {
	Shared::initialize(SharedInitiator {
		features: ["f", "g", "empty"].iter().copied().map(Into::into).collect(),
		thread: 32,
		mode: Mode::DryRun2,
		indent_symbol: IndentSymbol::Tab,
		indent_size: 4,
	});

	let s = r#"
[features]
f = [
	# x
	"x-d/f",
	"x-c/f",
	"b/f",


	"x-a/f",
	# y
	"y-c/f",
	"y-a/f",
	"y-b/f",
	# "y-d/f",
	# z
	"z-a/f",
	"z-d/f",

	"z-b/f",
	"z-c/f",
]

g = [
	# x
	"x-d/g",
	# "x-c/g",
	"b/g",


	"x-a/g",
	# y
	# "y-a/g",
	"y-c/g",

	"y-b/g",
	"y-d/g",
	# z
	"z-d/g",
	"z-a/g",
	# "z-b/g",
	"z-c/g",
]

empty = []
"#;

	let mut d = s.parse::<DocumentMut>().unwrap();
	let mut s = SortVisitor(["f", "g", "empty"].iter().map(|s| (*s).into()).collect());
	s.visit_document_mut(&mut d);

	assert_eq!(
		d.to_string(),
		r#"
[features]
f = [
	# x
	"b/f",
	"x-a/f",
	"x-c/f",
	"x-d/f",
	# y
	"y-a/f",
	"y-b/f",
	"y-c/f",
	# "y-d/f",
	# z
	"z-a/f",
	"z-b/f",
	"z-c/f",
	"z-d/f",
]

g = [
	# x
	"x-d/g",
	# "x-c/g",
	"b/g",
	"x-a/g",
	# y
	# "y-a/g",
	"y-b/g",
	"y-c/g",
	"y-d/g",
	# z
	"z-a/g",
	"z-d/g",
	# "z-b/g",
	"z-c/g",
]

empty = []
"#
	);
}
