//! Nodes whose status is determined by a function
use node::{Node, Internals};
use status::Status;

/// A node whose success depends on a function that can be run in a single tick
pub struct Condition
{
	/// Function that is performed to determine the node's status
	///
	/// A return value of `true` means success and a return value of `false` means failure
	func: Box<Fn() -> bool>,
}
impl Condition
{
	/// Constructs a new Condition node
	///
	/// If the functio returns `true`, then then node succeeds. Otherwise the node fails.
	pub fn new(func: Box<Fn() -> bool>) -> Node
	{
		let internals = Condition { func: func };
		Node::new(internals)
	}
}
impl Internals for Condition
{
	fn tick(&mut self) -> Status
	{
		// Otherwise, run the function
		if (*self.func)() {
			Status::Succeeded
		} else {
			Status::Failed
		}
	}

	fn reset(&mut self)
	{
		// No-op
	}

	fn type_name(&self) -> &'static str
	{
		"Condition"
	}
}

#[cfg(test)]
mod test
{
	use status::Status;
	use std_nodes::*;

	#[test]
	fn failure()
	{
		let mut cond = Condition::new(Box::new(|| false));
		assert_eq!(cond.tick(&world), Status::Failed);
	}

	#[test]
	fn success()
	{
		let mut cond = Condition::new(Box::new(|| true));
		assert_eq!(cond.tick(&world), Status::Succeeded);
	}
}
