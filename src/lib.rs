#![doc = include_str!("../README.md")]
#![cfg_attr(not(feature = "std"), no_std)]

mod builder;
mod captures;
mod key_part;
mod prefix_tree_map;

pub use self::{
    builder::PrefixTreeMapBuilder, captures::Captures, key_part::KeyPart,
    prefix_tree_map::PrefixTreeMap,
};

#[cfg(not(feature = "std"))]
mod std_lib {
    extern crate alloc;
    pub(crate) use alloc::{
        collections::{BTreeMap, BinaryHeap},
        vec::Vec,
    };
    pub(crate) use core::{
        cmp::Ordering,
        fmt::{Debug, Formatter, Result as FmtResult},
    };
}

#[cfg(feature = "std")]
mod std_lib {
    pub(crate) use std::{
        cmp::Ordering,
        collections::{BTreeMap, BinaryHeap},
        fmt::{Debug, Formatter, Result as FmtResult},
        vec::Vec,
    };
}
