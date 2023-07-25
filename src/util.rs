// std
use std::{borrow::Cow, path::PathBuf};
// crates.io
use cargo_metadata::{Metadata, Node, Package, PackageId, Resolve};

pub trait GetById<'a> {
	type Id;
	type Item;

	fn get_by_id<'b>(self, id: &'b Self::Id) -> Option<&'a Self::Item>
	where
		'a: 'b;
}
impl<'a> GetById<'a> for &'a Metadata {
	type Id = PackageId;
	type Item = Package;

	fn get_by_id<'b>(self, id: &'b Self::Id) -> Option<&'a Self::Item>
	where
		'a: 'b,
	{
		self.packages.get_by_id(id)
	}
}
impl<'a> GetById<'a> for &'a [Package] {
	type Id = PackageId;
	type Item = Package;

	fn get_by_id<'b>(self, id: &'b Self::Id) -> Option<&'a Self::Item>
	where
		'a: 'b,
	{
		self.iter().find(|p| &p.id == id)
	}
}
impl<'a> GetById<'a> for &'a Resolve {
	type Id = PackageId;
	type Item = Node;

	fn get_by_id<'b>(self, id: &'b Self::Id) -> Option<&'a Self::Item>
	where
		'a: 'b,
	{
		self.nodes.get_by_id(id)
	}
}
impl<'a> GetById<'a> for &'a [Node] {
	type Id = PackageId;
	type Item = Node;

	fn get_by_id<'b>(self, id: &'b Self::Id) -> Option<&'a Self::Item>
	where
		'a: 'b,
	{
		self.iter().find(|n| &n.id == id)
	}
}

pub fn manifest_path_of(path: &PathBuf) -> Cow<PathBuf> {
	if path.is_file() {
		Cow::Borrowed(path)
	} else {
		let mut p = path.to_owned();

		p.push("Cargo.toml");

		Cow::Owned(p)
	}
}
