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

use self::identifier::*;

pub mod identifier;

/// This struct stores routing information about other peers.
pub struct Routing<T> {
    current: IdentifierValue<T>,
    // TODO should maybe be an Option
    predecessor: IdentifierValue<T>,
    // TODO use BinaryHeap for multiple successors
    successor: IdentifierValue<T>,
    // TODO
    finger_table: Vec<IdentifierValue<T>>
}

impl<T: Identify> Routing<T> {
    /// Creates a new `Routing` instance for the given initial values.
    pub fn new(current: T, predecessor: T, successor: T, finger_table: Vec<T>)
        -> Self
    {
        Self {
            current: IdentifierValue::new(current),
            predecessor: IdentifierValue::new(predecessor),
            successor: IdentifierValue::new(successor),
            finger_table: finger_table.into_iter().map(IdentifierValue::new).collect()
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

    /// Points the finger to the given peer.
    pub fn update_finger(&mut self, peer: T, index: usize) {
        self.finger_table[index] = IdentifierValue::new(peer);
    }

    /// Returns the peer closest to the given identifier.
    pub fn closest_peer(&self, identifier: &Identifier) -> &T {
        let diff = *identifier - *self.current.get_identifier();
        let zeros = diff.leading_zeros() as usize;

        if zeros >= self.finger_table.len() {
            self.successor.get_value()
        } else {
            self.finger_table[zeros].get_value()
        }
    }
}
