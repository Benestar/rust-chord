//! This modules implements identifier-based routing using consistent hashing.
//!
//! An [`Identifier`] is a 256 bit number on an identifier circle containing
//! all non-negative integers module 2^256. Each peer can obtain its identifier
//! by hashing its own ip address. The peers are responsible for the range on
//! the identifier circle beginning with the identifier after their predecessor
//! up to and including their own identifier.
//!
//! The [`Routing`] struct supports all operations to obtain the closest known
//! peer on the identifier circle to some given identifier by using a so called
//! "finger table". This data structure contains pointers to the peers
//! responsible for every 2^i-th identifier after our own identifier. This
//! allows us to find the responsible peer for an arbitrary identifier in
//! O(log(N)) steps where N is the size of the whole network.
//!
//! [`Identifier`]: identifier/struct.Identifier.html
//! [`Routing`]: struct.Routing.html

use routing::identifier::*;

pub mod identifier;

/// This struct stores routing information about other peers.
pub struct Routing<T> {
    current: IdentifierValue<T>,
    // TODO should maybe be an Option
    predecessor: IdentifierValue<T>,
    // TODO use BinaryHeap for multiple successors
    successor: IdentifierValue<T>
}

impl<T: Identify> Routing<T> {
    /// Creates a new `Routing` instance for the given initial values.
    pub fn new(current: T, predecessor: T, successor: T) -> Self {
        Routing {
            current: IdentifierValue::new(current),
            predecessor: IdentifierValue::new(predecessor),
            successor: IdentifierValue::new(successor)
        }
    }

    /// Returns the current address.
    pub fn get_current(&self) -> &T {
        self.current.get_value()
    }

    /// Returns the predecessor's address.
    pub fn get_predecessor(&self) -> &T {
        self.predecessor.get_value()
    }

    /// Sets the predecessor's address.
    pub fn set_predecessor(&mut self, new_pred: T) {
        self.predecessor = IdentifierValue::new(new_pred);
    }

    /// Checks whether the given address is closer than the address of the
    /// current predecessor.
    pub fn is_closer_predecessor(&self, new_pred: &T) -> bool {
        new_pred.get_identifier().is_between(
            self.predecessor.get_identifier(),
            self.current.get_identifier()
        )
    }

    /// Returns the current successor.
    pub fn get_successor(&self) -> &T {
        self.successor.get_value()
    }

    /// Sets the current successor.
    pub fn set_successor(&mut self, new_succ: T) {
        self.successor = IdentifierValue::new(new_succ);
    }

    /// Checks whether this peer is responsible for the given identifier.
    pub fn responsible_for(&self, identifier: &Identifier) -> bool {
        identifier.is_between(
            self.predecessor.get_identifier(),
            self.current.get_identifier()
        )
    }
}
