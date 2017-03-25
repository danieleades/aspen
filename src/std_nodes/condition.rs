//! Nodes whose status is determined by a function that runs within the tick.
use node::{Node, Internals};
use status::Status;

/// A node whose status is determined by a function.
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
	/// If the function returns `true` then then node succeeds, otherwise the
	/// node fails. The function will run within the same thread that calls
	/// `tick`, so its execution should be completed within a single tick's
	/// amount of time.
	pub fn new<F: Fn() -> bool + 'static>(func: F) -> Node
	{
		let internals = Condition { func: Box::new(func) };
		Node::new(internals)
	}
}
impl Internals for Condition
{
	/// Calls the supplied status function, returning `Status::Succeeded` if it
	/// returned `true`, with `Status::Failed` otherwise. This will never return
	/// `Status::Running`.
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

	/// Returns the string "Condition"
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
		let mut cond = Condition::new(|| false);
		assert_eq!(cond.tick(), Status::Failed);
	}

	#[test]
	fn success()
	{
		let mut cond = Condition::new(|| true);
		assert_eq!(cond.tick(), Status::Succeeded);
	}
}
