// std
use std::{
	mem,
	sync::{Arc, Mutex},
};
// crates.io
use cargo_metadata::{
	CargoOpt, DependencyKind, Metadata, MetadataCommand, Node, NodeDep, Package, PackageId, Resolve,
};
use fxhash::FxHashMap;
use once_cell::sync::{Lazy, OnceCell};
use serde::Serialize;
// cargo-featalign
use crate::{cli::AnalyzerInitiator, prelude::*, shared::FEATURES, util::GetById};

#[allow(clippy::type_complexity)]
pub static PROBLEMS: Lazy<Arc<Mutex<FxHashMap<PackageId, Vec<ProblemCrate>>>>> =
	Lazy::new(|| Arc::new(Mutex::new(FxHashMap::default())));
pub fn append_problems(id: PackageId, problems: Vec<ProblemCrate>) {
	if !problems.is_empty() {
		let mut ps = PROBLEMS.lock().unwrap();

		if let Some(pcs) = ps.get_mut(&id) {
			problems.into_iter().for_each(|pc| {
				if !pcs.contains(&pc) {
					pcs.push(pc);
				}
			});
		} else {
			ps.insert(id, problems);
		}
	}
}

static WORKSPACE_ONLY: OnceCell<bool> = OnceCell::new();
static IGNORE: OnceCell<Vec<String>> = OnceCell::new();
static DEFAULT_STD: OnceCell<bool> = OnceCell::new();
static NON_DEFAULT_STD: OnceCell<Vec<String>> = OnceCell::new();

#[derive(Debug, Clone)]
pub struct Analyzer {
	// TODO?: replace with `FxHashMap` packages
	metadata: Arc<Metadata>,
	// Remove?
	resolve: Arc<Resolve>,
}
impl Analyzer {
	pub fn initialize(initiator: AnalyzerInitiator) -> Self {
		let manifest_path = util::manifest_path_of(&initiator.manifest_path);

		WORKSPACE_ONLY.set(initiator.workspace_only).unwrap();
		IGNORE.set(initiator.ignore).unwrap();
		DEFAULT_STD.set(initiator.default_std).unwrap();
		NON_DEFAULT_STD.set(initiator.non_default_std).unwrap();

		let mut metadata = MetadataCommand::new()
			.manifest_path(&*manifest_path)
			.features(CargoOpt::AllFeatures)
			.exec()
			.unwrap_or_else(|_| {
				panic!(
					"failed to execute the `cargo metadata` command for the directory `{}`.",
					manifest_path.display()
				)
			});
		let resolve = mem::take(&mut metadata.resolve).unwrap();

		Self { metadata: Arc::new(metadata), resolve: Arc::new(resolve) }
	}

	pub fn analyze(self, depth: i16) {
		let r = self.resolve.root.as_ref().expect(
			"the `[package]` specified in the `Cargo.toml` cannot be found\n\
			it appears to be a pure workspace which is not supported",
		);
		let n = self.resolve.nodes.get_by_id(r).unwrap().to_owned();
		let p = self.metadata.get_by_id(&n.id).unwrap().to_owned();

		self.analyze_crate(n, p, depth, String::new());
	}

	fn analyze_crate(self, node: Node, package: Package, depth: i16, mut dependency_path: String) {
		if *WORKSPACE_ONLY.get().unwrap() && !self.is_workspace_member(&package.id)
			|| IGNORE.get().unwrap().contains(&package.name)
		{
			return;
		}

		dependency_path.push_str(&format!("/{}", package.name.clone()));

		self.analyze_features(&node, &package, &dependency_path);

		if in_depth(depth) {
			let mut ts = Vec::new();

			for d in &node.deps {
				if is_dev(d) {
					continue;
				}

				let n = self.resolve.get_by_id(&d.pkg).unwrap().to_owned();
				let p = self.metadata.get_by_id(&d.pkg).unwrap().to_owned();
				let dependency_path = dependency_path.clone();
				let analyzer = self.clone();

				if depth > 8 {
					analyzer.analyze_crate(n, p, depth - 1, dependency_path);
				} else {
					shared::activate_thread(&mut ts, move || {
						analyzer.analyze_crate(n, p, depth - 1, dependency_path)
					});
				}
			}

			shared::deactivate_threads(ts);
		}
	}

	fn analyze_features(&self, node: &Node, package: &Package, dependency_path: &str) {
		let rs = package
			.dependencies
			.iter()
			.filter_map(|d| d.rename.as_ref().map(|rn| (d.name.as_str(), rn.as_str())))
			.collect::<Vec<_>>();
		let has_std_feat = package.features.contains_key("std");
		let non_optional_deps = if *DEFAULT_STD.get().unwrap() && has_std_feat {
			package.dependencies.iter().filter(|d| !d.optional).collect::<Vec<_>>()
		} else {
			Vec::new()
		};
		let fs = FEATURES.get().unwrap();
		let fs = package.features.iter().filter(|(f, _)| fs.contains(f)).collect::<Vec<_>>();
		let mut problem_cs = Vec::new();

		for d in &node.deps {
			if is_dev(d) {
				continue;
			}

			let p_id = &d.pkg;
			let p = self.metadata.get_by_id(p_id).unwrap();
			let p_name = p.name.as_str();
			let p_alias = rs.get_by_id(p_name).unwrap_or(p_name);
			let n = self.resolve.get_by_id(p_id).unwrap();
			let mut missing_fs = Vec::new();

			if !non_optional_deps.is_empty()
				&& !is_non_default_std(&d.name)
				// Package's dependencies have `std` feature enabled.
				&& non_optional_deps.iter().any(|d| {
					d.name == p.name
						&& d.uses_default_features
						&& p.features
							.get("default")
							.map(|dfs| dfs.iter().any(|f| f == "std"))
							.unwrap_or_default()
				}) {
				problem_cs.push(ProblemCrate {
					id: p_id.to_owned(),
					alias: p_alias.to_owned(),
					dependency_path: dependency_path.to_owned(),
					problem: Problem::DefaultFeaturesEnabled,
				});
			}

			'out: for (f, required_fs) in &fs {
				// If the dependency has the feature specified by the user for analyzing.
				if n.features.contains(f) {
					if package.dependencies.iter().any(|d| {
						d.name == p_name
							&& d.uses_default_features
							&& p.features
								.get("default")
								.map(|dfs| dfs.iter().any(|f_| f_ == *f))
								.unwrap_or_default()
					}) {
						continue;
					}

					// `assert!("general-a/std".contains("general-a"));`
					for f in *required_fs {
						// TODO: handle the full name here
						// e.g. this could be `general-a/std` or `general-a?/std`
						if f.contains(p_alias) {
							continue 'out;
						}
					}

					missing_fs.push((*f).to_owned());
				}
			}

			if !missing_fs.is_empty() {
				problem_cs.push(ProblemCrate {
					id: p_id.to_owned(),
					alias: p_alias.to_owned(),
					dependency_path: dependency_path.to_owned(),
					problem: Problem::MissingFeatures(missing_fs),
				});
			}
		}

		append_problems(node.id.clone(), problem_cs);
	}

	fn is_workspace_member(&self, id: &PackageId) -> bool {
		self.metadata.workspace_members.contains(id)
	}
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "kebab-case")]
pub struct ProblemCrate {
	pub id: PackageId,
	pub alias: String,
	pub dependency_path: String,
	pub problem: Problem,
}
// TODO?: this would affect the dependency path
impl PartialEq for ProblemCrate {
	fn eq(&self, other: &Self) -> bool {
		self.id == other.id
	}
}
#[derive(Debug, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum Problem {
	DefaultFeaturesEnabled,
	MissingFeatures(Vec<String>),
}

// Check if the this package is under the `[dev-dependencies]`.
fn is_dev(node_dep: &NodeDep) -> bool {
	node_dep.dep_kinds.iter().any(|k| matches!(k.kind, DependencyKind::Development))
}

fn is_non_default_std(name: &str) -> bool {
	NON_DEFAULT_STD.get().unwrap().iter().any(|n| n == name)
}

fn in_depth(depth: i16) -> bool {
	depth != 0 || depth == -1
}
