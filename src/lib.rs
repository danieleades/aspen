#![deny(missing_docs)]

//! This crate is an implementation of behavior trees in Rust. It is largely
//! based on behavior trees as described by Marzinotto et al. ^1 and was designed
//! to be used on an actual robot using LCM for communication.
//!
//! A nice overview of behavior trees can be found on
//! [Craft AI's website](http://www.craft.ai/blog/bt-101-behavior-trees-grammar-basics/).
//!
//! 1: Marzinotto, Alejandro, et al. "Towards a unified behavior trees
//! framework for robot control." Robotics and Automation (ICRA), 2014 IEEE
//! International Conference on. IEEE, 2014.

#[macro_use]
extern crate log;

mod bt;
pub use crate::bt::BehaviorTree;

pub mod node;

mod status;
pub use crate::status::Status;

pub mod std_nodes;
