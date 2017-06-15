//! Nodes that have a constant behavior.
use node::{Node, Internals};
use status::Status;

/// Implements a node that always returns that it has failed.
///
/// This node potentially takes a child node. If it does, then it will tick that
/// node until it is completed, disregard the child's status, and return that it
/// failed. If it does not have a child node, it will simply fail on every tick.
///
/// # State
///
/// **Initialized:** Before being ticked after either being created or reset.
///
/// **Running:** While child is running. If no child, then never.
///
/// **Succeeded:** Never.
///
/// **Failed:** After child finishes. If no child, always.
///
/// # Children
///
/// One optional child. The child will be reset every time this node is reset.
///
/// # Examples
///
/// An `AlwaysFail` node always fails when it has no child:
///
/// ```
/// # use aspen::std_nodes::*;
/// # use aspen::Status;
/// let mut node = AlwaysFail::new();
/// assert_eq!(node.tick(), Status::Failed);
/// ```
///
/// If the child is considered running, so is this node:
///
/// ```
/// # use aspen::std_nodes::*;
/// # use aspen::Status;
/// let mut node = AlwaysFail::with_child(AlwaysRunning::new());
/// assert_eq!(node.tick(), Status::Running);
/// ```
///
/// If the child is done running, its status is disregarded:
///
/// ```
/// # use aspen::std_nodes::*;
/// # use aspen::Status;
/// let mut node = AlwaysFail::with_child(AlwaysSucceed::new());
/// assert_eq!(node.tick(), Status::Failed);
/// ```
pub struct AlwaysFail<'a, S>
{
	/// Optional child node.
	child: Option<Node<'a, S>>,
}
impl<'a, S> AlwaysFail<'a, S>
	where S: 'a
{
	/// Construct a new AlwaysFail node.
	pub fn new() -> Node<'a, S>
	{
		Node::new(AlwaysFail { child: None })
	}

	/// Construct a new AlwaysFail node that has a child.
	pub fn with_child(child: Node<'a, S>) -> Node<'a, S>
	{
		Node::new(AlwaysFail { child: Some(child) })
	}
}
impl<'a, S> Internals<S> for AlwaysFail<'a, S>
{
	fn tick(&mut self, world: &mut S) -> Status
	{
		if let Some(ref mut child) = self.child {
			if !child.tick(world).is_done() {
				return Status::Running;
			}
		}

		Status::Failed
	}

	fn reset(&mut self)
	{
		if let Some(ref mut child) = self.child {
			child.reset();
		}
	}

	fn children(&self) -> Vec<&Node<S>>
	{
		if let Some(ref child) = self.child {
			vec![child]
		} else {
			Vec::new()
		}
	}

	/// Returns the string "AlwaysFail".
	fn type_name(&self) -> &'static str
	{
		"AlwaysFail"
	}
}

/// Convenience macro for creating AlwaysFail nodes.
///
/// # Examples
///
/// ```
/// # #[macro_use] extern crate aspen;
/// # fn main() {
/// # let (a, b) = (12, 13);
/// let fail = AlwaysFail!{};
/// let fail_child = AlwaysFail!{
///     Condition!{ || a < b }
/// };
/// # }
/// ```
#[macro_export]
macro_rules! AlwaysFail
{
	( $e:expr ) => {
		$crate::std_nodes::AlwaysFail::with_child($e)
	};
	( ) => {
		$crate::std_nodes::AlwaysFail::new()
	}
}

/// Implements a node that always returns that it has succeeded.
///
/// This node potentially takes a child node. If it does, then it will tick that
/// node until it is completed, disregard the child's status, and return that it
/// succeeded. If it does not have a child node, it will simply succeed on
/// every tick.
///
/// # State
///
/// **Initialized:** Before being ticked after either being created or reset.
///
/// **Running:** While child is running. If no child, then never.
///
/// **Succeeded:** After child finished. If no child, always.
///
/// **Failed:** Never.
///
/// # Children
///
/// One optional child. The child will be reset every time this node is reset.
///
/// # Examples
///
/// An `AlwaysSucceed` node always succeeds when it has no child:
///
/// ```
/// # use aspen::std_nodes::*;
/// # use aspen::Status;
/// let mut node = AlwaysSucceed::new();
/// assert_eq!(node.tick(), Status::Succeeded);
/// ```
///
/// If the child is considered running, so is this node:
///
/// ```
/// # use aspen::std_nodes::*;
/// # use aspen::Status;
/// let mut node = AlwaysSucceed::with_child(AlwaysRunning::new());
/// assert_eq!(node.tick(), Status::Running);
/// ```
///
/// If the child is done running, its status is disregarded:
///
/// ```
/// # use aspen::std_nodes::*;
/// # use aspen::Status;
/// let mut node = AlwaysSucceed::with_child(AlwaysFail::new());
/// assert_eq!(node.tick(), Status::Succeeded);
/// ```
pub struct AlwaysSucceed<'a, S>
{
	/// Optional child node.
	child: Option<Node<'a, S>>,
}
impl<'a, S> AlwaysSucceed<'a, S>
	where S: 'a
{
	/// Construct a new AlwaysSucceed node.
	pub fn new() -> Node<'a, S>
	{
		Node::new(AlwaysSucceed { child: None })
	}

	/// Construct a new AlwaysSucceed node with a child.
	pub fn with_child(child: Node<'a, S>) -> Node<'a, S>
	{
		Node::new(AlwaysSucceed { child: Some(child) })
	}
}
impl<'a, S> Internals<S> for AlwaysSucceed<'a, S>
{
	fn tick(&mut self, world: &mut S) -> Status
	{
		if let Some(ref mut child) = self.child {
			if !child.tick(world).is_done() {
				return Status::Running;
			}
		}

		Status::Succeeded
	}

	fn children(&self) -> Vec<&Node<S>>
	{
		if let Some(ref child) = self.child {
			vec![child]
		} else {
			Vec::new()
		}
	}

	fn reset(&mut self)
	{
		if let Some(ref mut child) = self.child {
			child.reset();
		}
	}

	/// Returns the string "AlwaysSucceed".
	fn type_name(&self) -> &'static str
	{
		"AlwaysSucceed"
	}
}

/// Convenience macro for creating AlwaysSucceed nodes.
///
/// # Examples
///
/// ```
/// # #[macro_use] extern crate aspen;
/// # fn main() {
/// # let (a, b) = (12, 13);
/// let succeed = AlwaysSucceed!{};
/// let succeed_child = AlwaysSucceed!{
///     Condition!{ || a < b }
/// };
/// # }
/// ```
#[macro_export]
macro_rules! AlwaysSucceed
{
	( $e:expr ) => {
		$crate::std_nodes::AlwaysSucceed::with_child($e)
	};
	( ) => {
		$crate::std_nodes::AlwaysSucceed::new()
	}
}

/// Implements a node that always returns that it is currently running.
///
/// # State
///
/// **Initialized:** Before being ticked after either being created or reset.
///
/// **Running:** Always.
///
/// **Succeeded:** Never.
///
/// **Failed:** Never.
///
/// # Children
///
/// None.
///
/// # Examples
///
/// An `AlwaysRunning` node is always running:
///
/// ```
/// # use aspen::std_nodes::*;
/// # use aspen::Status;
/// let mut node = AlwaysRunning::new();
/// assert_eq!(node.tick(), Status::Running);
/// ```
pub struct AlwaysRunning;
impl AlwaysRunning
{
	/// Construct a new AlwaysRunning node.
	pub fn new<S>() -> Node<'static, S>
	{
		Node::new(AlwaysRunning { })
	}
}
impl<S> Internals<S> for AlwaysRunning
{
	fn tick(&mut self, _: &mut S) -> Status
	{
		Status::Running
	}

	fn reset(&mut self)
	{
		// No-op
	}

	/// Returns the string "AlwaysRunning".
	fn type_name(&self) -> &'static str
	{
		"AlwaysRunning"
	}
}

/// Convenience macro for creating AlwaysRunning nodes.
///
/// # Examples
///
/// ```
/// # #[macro_use] extern crate aspen;
/// # fn main() {
/// let running = AlwaysRunning!{};
/// # }
/// ```
#[macro_export]
macro_rules! AlwaysRunning
{
	( ) => {
		$crate::std_nodes::AlwaysRunning::new()
	}
}

#[cfg(test)]
mod test
{
	use status::Status;
	use std_nodes::*;

	#[test]
	fn always_fail()
	{
		assert_eq!(AlwaysFail::new().tick(), Status::Failed);
	}

	#[test]
	fn always_fail_child()
	{
		let mut succeed = AlwaysFail::with_child(YesTick::new(Status::Succeeded));
		let succeed_res = succeed.tick();
		drop(succeed);
		assert_eq!(succeed_res, Status::Failed);

		let mut run = AlwaysFail::with_child(YesTick::new(Status::Running));
		let run_res = run.tick();
		drop(run);
		assert_eq!(run_res, Status::Running);

		let mut fail = AlwaysFail::with_child(YesTick::new(Status::Failed));
		let fail_res = fail.tick();
		drop(fail);
		assert_eq!(fail_res, Status::Failed);
	}

	#[test]
	fn always_succeed()
	{
		assert_eq!(AlwaysSucceed::new().tick(), Status::Succeeded);
	}

	#[test]
	fn always_succeed_child()
	{
		let mut succeed = AlwaysSucceed::with_child(YesTick::new(Status::Succeeded));
		let succeed_res = succeed.tick();
		drop(succeed);
		assert_eq!(succeed_res, Status::Succeeded);

		let mut run = AlwaysSucceed::with_child(YesTick::new(Status::Running));
		let run_res = run.tick();
		drop(run);
		assert_eq!(run_res, Status::Running);

		let mut fail = AlwaysSucceed::with_child(YesTick::new(Status::Failed));
		let fail_res = fail.tick();
		drop(fail);
		assert_eq!(fail_res, Status::Succeeded);
	}

	#[test]
	fn always_running()
	{
		assert_eq!(AlwaysRunning::new().tick(), Status::Running);
	}
}
