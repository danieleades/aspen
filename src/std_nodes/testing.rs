//! Standard nodes used for debugging purposes.
use crate::{
    node::{Node, Tickable},
    status::Status,
};
use std::ops::Drop;

/// Implements a node that will panic upon being ticked.
pub struct NoTick;
impl NoTick {
    /// Construct a new `NoTick` node.
    pub fn new<W>() -> Node<'static, W> {
        Node::new(NoTick {})
    }
}
impl<W> Tickable<W> for NoTick {
    fn tick(&mut self, _: &mut W) -> Status {
        panic!("This node should not have been ticked");
    }

    fn reset(&mut self) {
        // No-op
    }

    /// Returns the string "NoTick".
    fn type_name(&self) -> &'static str {
        "NoTick"
    }
}

/// Implements a node that will panic if it is dropped without being ticked.
pub struct YesTick {
    /// The status that this node should return.
    status: Status,

    /// Whether or not this node has been ticked.
    ticked: bool,
}
impl YesTick {
    /// Create a new `YesTick` that always has the given status
    pub fn new<W>(status: Status) -> Node<'static, W> {
        let internals = YesTick {
            status,
            ticked: false,
        };
        Node::new(internals)
    }
}
impl<W> Tickable<W> for YesTick {
    fn tick(&mut self, _: &mut W) -> Status {
        self.ticked = true;
        self.status
    }

    fn reset(&mut self) {
        self.ticked = false;
    }

    /// Returns the string "YesTick".
    fn type_name(&self) -> &'static str {
        "YesTick"
    }
}
impl Drop for YesTick {
    fn drop(&mut self) {
        if !self.ticked {
            panic!("This node should have been ticked");
        }
    }
}

/// Implements a node that must be ticked a specific number of times.
pub struct CountedTick {
    /// The status this node is to return.
    status: Status,

    /// The number of times this node has been ticked.
    count: u32,

    /// The number of times this node is allowed to be ticked.
    limit: u32,

    /// Whether or not the node can be ticked more than the given count.
    exact: bool,

    /// Whether or not the count resets on node reset
    resetable: bool,
}
impl CountedTick {
    /// Creates a new `CountedTick` that always has the given status.
    pub fn new<W>(status: Status, count: u32, exact: bool) -> Node<'static, W> {
        let internals = CountedTick {
            status,
            count: 0,
            limit: count,
            exact,
            resetable: false,
        };
        Node::new(internals)
    }

    /// Creates a new `CountedTick` that will reset the count upon node reset
    pub fn resetable<W>(status: Status, count: u32, exact: bool) -> Node<'static, W> {
        let internals = CountedTick {
            status,
            count: 0,
            limit: count,
            exact,
            resetable: true,
        };
        Node::new(internals)
    }
}
impl<W> Tickable<W> for CountedTick {
    fn tick(&mut self, _: &mut W) -> Status {
        if self.exact && self.count == self.limit {
            panic!(
                "Node was ticked too many times: {} actual, {} expected",
                self.count + 1,
                self.limit
            );
        }

        self.count = self.count.saturating_add(1);
        self.status
    }

    fn reset(&mut self) {
        if self.resetable {
            self.count = 0;
        }
    }

    /// Returns the string "CountedTick".
    fn type_name(&self) -> &'static str {
        "CountedTick"
    }
}
impl Drop for CountedTick {
    fn drop(&mut self) {
        if self.count < self.limit {
            panic!(
                "Node was not ticked enough times: {} actual, {} expected",
                self.count, self.limit
            );
        }
    }
}
