use crate::node::{Node, Tickable};
use crate::status::Status;

/// A node that repeats its child until the child fails.
///
/// This node will return that it is running until the child fails. It can
/// potentially have a finite reset limit. If the child ever returns that it
/// fails, this node returns that it *succeeds*. If the limit is reached before
/// the child fails, this node *fails*.
///
/// # State
///
/// **Initialized:** Before being ticked after either being created or reset.
///
/// **Running:** While the child node has yet to fail and it is below the reset
/// limit.
///
/// **Succeeded:** Once the child node fails.
///
/// **Failed:** If the reset limit was reached before the child failed.
///
/// # Children
///
/// One, which will be ticked or reset every time the `UntilFail` node is
/// ticked or reset. The child may also be reset multiple times before the parent
/// node is reset or completed.
///
/// # Examples
///
/// A child that will be repeated infinitely until it fails.
///
/// ```
/// # use aspen::std_nodes::*;
/// # use aspen::Status;
/// # use aspen::node::Tickable;
/// let child = Condition::new(|&d| d < 10 );
/// let mut node = UntilFail::new(child);
///
/// for mut x in 0..10 {
///     assert_eq!(node.tick(&mut x), Status::Running);
/// }
///
/// assert_eq!(node.tick(&mut 11), Status::Succeeded);
/// ```
///
/// An `UntilFail` node will fail if the child doesn't within the limit:
///
/// ```
/// # use aspen::std_nodes::*;
/// # use aspen::Status;
/// # use aspen::node::Tickable;
/// let tries = 10;
/// let child = AlwaysSucceed::new();
/// let mut node = UntilFail::with_limit(tries, child);
///
/// // Subtract one since our final assert counts as a try
/// for _ in 0..(tries - 1) {
///     assert_eq!(node.tick(&mut ()), Status::Running);
/// }
///
/// assert_eq!(node.tick(&mut ()), Status::Failed);
/// ```
pub struct UntilFail<'a, W>
{
	/// Child node.
	child: Node<'a, W>,

	/// Optional number of times to do the reset.
	attempt_limit: Option<u32>,

	/// Number of times the child has been reset.
	attempts: u32,
}
impl<'a, W> UntilFail<'a, W>
	where W: 'a
{
	/// Creates a new `UntilFail` node that will keep trying indefinitely.
	pub fn new(child: Node<'a, W>) -> Node<'a, W>
	{
		let internals = UntilFail {
			child: child,
			attempt_limit: None,
			attempts: 0,
		};
		Node::new(internals)
	}

	/// Creates a new `UntilFail` node that will only retry a specific number of times.
	///
	/// The limit is the number of times the node will run, not the number of
	/// times it will be reset. A limit of zero means instant failure.
	pub fn with_limit(limit: u32, child: Node<'a, W>) -> Node<'a, W>
	{
		let internals = UntilFail {
			child: child,
			attempt_limit: Some(limit),
			attempts: 0,
		};
		Node::new(internals)
	}
}
impl<'a, W> Tickable<W> for UntilFail<'a, W>
{
	fn tick(&mut self, world: &mut W) -> Status
	{
		// Take care of the infinite version so we don't have to worry
		if self.attempt_limit.is_none() {
			return if self.child.tick(world) == Status::Failed {
				Status::Succeeded
			} else { Status::Running };
		}

		// We're using the finite version
		let limit = self.attempt_limit.unwrap();
		let child_status = self.child.tick(world);

		// It's either check this here or do it at both of the following
		// returns. I'll take here.
		if child_status == Status::Failed {
			return Status::Succeeded;
		}

		if child_status.is_done() {
			self.attempts += 1;
			if self.attempts < limit {
				return Status::Running;
			}
			else {
				return Status::Failed;
			}
		}

		// We're still running
		Status::Running
	}

	fn reset(&mut self)
	{
		// Reset our own status
		self.attempts = 0;

		// Reset the child
		self.child.reset();
	}

	fn children(&self) -> Vec<&Node<W>>
	{
		vec![&self.child]
	}

	/// Returns the string "UntilFail".
	fn type_name(&self) -> &'static str
	{
		"UntilFail"
	}
}

/// Convenience macro for creating UntilFail nodes.
///
/// # Examples
///
/// ```
/// # #[macro_use] extern crate aspen;
/// # fn main() {
/// let until_fail = UntilFail!{
///     Condition!{ |&(a, b): &(u32, u32)| a < b }
/// };
/// let limited_until_fail = UntilFail!{ 12,
///     Condition!{ |&(a, b): &(u32, u32)| a < b }
/// };
/// # }
/// ```
#[macro_export]
macro_rules! UntilFail
{
	( $e:expr ) => {
		$crate::std_nodes::UntilFail::new($e)
	};
	( $c:expr, $e:expr ) => {
		$crate::std_nodes::UntilFail::with_limit($c, $e)
	}
}

/// A node that repeats its child until the child succeeds.
///
/// This node will return that it is running until the child succeeds. It can
/// potentially have a finite reset limit. If the child ever returns that it
/// succeeds, this node returns that it *succeeds*. If the limit is reached before
/// the child succeeds, this node *fails*.
///
/// # State
///
/// **Initialized:** Before being ticked after either being created or reset.
///
/// **Running:** While the child node has yet to succeed and it is below the reset
/// limit.
///
/// **Succeeded:** Once the child node succeeds.
///
/// **Failed:** If the reset limit was reached before the child succeeded.
///
/// # Children
///
/// One, which will be ticked or reset every time the `UntilSuccess` node is
/// ticked or reset. The child may also be reset multiple times before the parent
/// node is reset or completed.
///
/// # Examples
///
/// A child that will be repeated infinitely until it succeeds.
///
/// ```
/// # use aspen::std_nodes::*;
/// # use aspen::Status;
/// # use aspen::node::Tickable;
/// let child = Condition::new(|&d| d == 10 );
/// let mut node = UntilSuccess::new(child);
///
/// for mut x in 0..10 {
///     assert_eq!(node.tick(&mut x), Status::Running);
/// }
///
/// assert_eq!(node.tick(&mut 10), Status::Succeeded);
/// ```
///
/// An `UntilSuccess` node will fail if the child doesn't succeed within the limit:
///
/// ```
/// # use aspen::std_nodes::*;
/// # use aspen::Status;
/// # use aspen::node::Tickable;
/// let runs = 10;
/// let child = AlwaysFail::new();
/// let mut node = UntilSuccess::with_limit(runs, child);
///
/// // Minus one since our final assert is a run
/// for _ in 0..(runs - 1) {
///     assert_eq!(node.tick(&mut ()), Status::Running);
/// }
///
/// assert_eq!(node.tick(&mut ()), Status::Failed);
/// ```
pub struct UntilSuccess<'a, W>
{
	/// Child node.
	child: Node<'a, W>,

	/// Optional number of times to do the reset.
	attempt_limit: Option<u32>,

	/// Number of times the child has been reset.
	attempts: u32,
}
impl<'a, W> UntilSuccess<'a, W>
	where W: 'a
{
	/// Creates a new `UntilSuccess` node that will keep trying indefinitely.
	pub fn new(child: Node<'a, W>) -> Node<'a, W>
	{
		let internals = UntilSuccess {
			child: child,
			attempt_limit: None,
			attempts: 0,
		};
		Node::new(internals)
	}

	/// Creates a new `UntilSuccess` node that will only retry a specific number of times.
	///
	/// `limit` is the number of times the node can be *reset*, not the number
	/// of times it can be run. A limit of one means the node can be run twice.
	pub fn with_limit(limit: u32, child: Node<'a, W>) -> Node<'a, W>
	{
		let internals = UntilSuccess {
			child: child,
			attempt_limit: Some(limit),
			attempts: 0,
		};
		Node::new(internals)
	}
}
impl<'a, W> Tickable<W> for UntilSuccess<'a, W>
{
	fn tick(&mut self, world: &mut W) -> Status
	{
		// Take care of the infinite version so we don't have to worry
		if self.attempt_limit.is_none() {
			return if self.child.tick(world) == Status::Succeeded {
				Status::Succeeded
			} else { Status::Running };
		}

		// We're using the finite version
		let limit = self.attempt_limit.unwrap();
		let child_status = self.child.tick(world);

		// It's either check this here or do it at both of the following
		// returns. I'll take here.
		if child_status == Status::Succeeded {
			return Status::Succeeded;
		}

		if child_status.is_done() {
			self.attempts += 1;
			if self.attempts < limit {
				return Status::Running;
			}
			else {
				return Status::Failed;
			}
		}

		// We're still running
		Status::Running
	}

	fn reset(&mut self)
	{
		// Reset our own status
		self.attempts = 0;

		// Reset the child
		self.child.reset();
	}

	fn children(&self) -> Vec<&Node<W>>
	{
		vec![&self.child]
	}

	/// Returns the string "UntilSuccess".
	fn type_name(&self) -> &'static str
	{
		"UntilSuccess"
	}
}

/// Convenience macro for creating UntilSuccess nodes.
///
/// # Examples
///
/// ```
/// # #[macro_use] extern crate aspen;
/// # fn main() {
/// let until_success = UntilSuccess!{
///     Condition!{ |&(a, b): &(u32, u32)| a < b }
/// };
/// let limited_until_success = UntilSuccess!{ 12,
///     Condition!{ |&(a, b): &(u32, u32)| a < b }
/// };
/// # }
/// ```
#[macro_export]
macro_rules! UntilSuccess
{
	( $e:expr ) => {
		$crate::std_nodes::UntilSuccess::new($e)
	};
	( $c:expr, $e:expr ) => {
		$crate::std_nodes::UntilSuccess::with_limit($c, $e)
	}
}

#[cfg(test)]
mod tests
{
	use crate::status::Status;
	use crate::std_nodes::*;
	use crate::node::Tickable;

	#[test]
	fn until_fail_infinite()
	{
		let child = CountedTick::new(Status::Failed, 1, true);
		let mut node = UntilFail::new(child);
		let status = node.tick(&mut ());
		drop(node);
		assert_eq!(status, Status::Succeeded);
	}

	#[test]
	fn until_fail_finite()
	{
		let limit = 5;
		let child = CountedTick::new(Status::Succeeded, limit, true);
		let mut node = UntilFail::with_limit(limit, child);
		for _ in 0..(limit - 1) {
			assert_eq!(node.tick(&mut ()), Status::Running);
		}
		let status = node.tick(&mut ());
		drop(node);
		assert_eq!(status, Status::Failed);
	}

	#[test]
	fn until_success_infinite()
	{
		let child = CountedTick::new(Status::Succeeded, 1, true);
		let mut node = UntilSuccess::new(child);
		let status = node.tick(&mut ());
		drop(node);
		assert_eq!(status, Status::Succeeded);
	}

	#[test]
	fn until_success_finite()
	{
		let limit = 5;
		let child = CountedTick::new(Status::Failed, limit, true);
		let mut node = UntilSuccess::with_limit(limit, child);
		for _ in 0..(limit - 1) {
			assert_eq!(node.tick(&mut ()), Status::Running);
		}
		let status = node.tick(&mut ());
		drop(node);
		assert_eq!(status, Status::Failed);
	}
}
