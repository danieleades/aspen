//! Nodes that have a constant behavior.
use node::{Node, Internals};
use status::Status;

/// Implements a node that always returns that it has failed
#[derive(PartialEq, Eq, Debug, Copy, Clone, Default)]
pub struct AlwaysFail;
impl AlwaysFail
{
	/// Construct a new AlwaysFail node
	pub fn new() -> Node
	{
		Node::new(AlwaysFail { })
	}
}
impl Internals for AlwaysFail
{
	/// Returns `Status::Failed`
	fn tick(&mut self) -> Status
	{
		Status::Failed
	}

	fn reset(&mut self)
	{
		// No-op
	}

	/// Returns the string "AlwaysFail"
	fn type_name(&self) -> &'static str
	{
		"AlwaysFail"
	}
}

/// Implements a node that always returns that it has succeeded
#[derive(PartialEq, Eq, Debug, Copy, Clone, Default)]
pub struct AlwaysSucceed;
impl AlwaysSucceed
{
	/// Construct a new AlwaysSucceed node
	pub fn new() -> Node
	{
		Node::new(AlwaysSucceed { })
	}
}
impl Internals for AlwaysSucceed
{
	/// Returns `Status::Succeeded`
	fn tick(&mut self) -> Status
	{
		Status::Succeeded
	}

	fn reset(&mut self)
	{
		// No-op
	}

	/// Returns the string "AlwaysSucceed"
	fn type_name(&self) -> &'static str
	{
		"AlwaysSucceed"
	}
}

/// Implements a node that always returns that it is currently running
#[derive(PartialEq, Eq, Debug, Copy, Clone, Default)]
pub struct AlwaysRunning;
impl AlwaysRunning
{
	/// Construct a new AlwaysRunning node
	pub fn new() -> Node
	{
		Node::new(AlwaysRunning { })
	}
}
impl Internals for AlwaysRunning
{
	/// Returns `Status::Running`
	fn tick(&mut self) -> Status
	{
		Status::Running
	}

	fn reset(&mut self)
	{
		// No-op
	}

	/// Returns the string "AlwaysRunning"
	fn type_name(&self) -> &'static str
	{
		"AlwaysRunning"
	}
}
