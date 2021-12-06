#![doc = include_str!("../README.md")]

mod builder;
mod captures;
mod key_part;
mod prefix_tree_map;

pub use self::{
    builder::PrefixTreeMapBuilder, captures::Captures, key_part::KeyPart,
    prefix_tree_map::PrefixTreeMap,
};
