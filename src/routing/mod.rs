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
use std::collections::BinaryHeap;

pub mod identifier;

/// This struct stores routing information about other peers.
#[derive(Debug)]
pub struct Routing<T> {
    /// Address where the peer is listening for peer-to-peer messages
    pub current: IdentifierValue<T>,
    /// The closest predecessor of this peer
    pub predecessor: IdentifierValue<T>,
    /// A list of successors of this peer
    pub successors: BinaryHeap<IdentifierValue<T>>,
    /// The finger table of
    finger_table: Vec<IdentifierValue<T>>
}

impl<T: Identify + Clone> Routing<T> {
    /// Creates a new `Routing` instance for the given initial values.
    pub fn new(current: T, predecessor: T, successors: Vec<T>, finger_table: Vec<T>)
        -> Self
    {
        Self {
            current: IdentifierValue::new(current),
            predecessor: IdentifierValue::new(predecessor),
            successors: successors.into_iter().map(IdentifierValue::new).collect(),
            finger_table: finger_table.into_iter().map(IdentifierValue::new).collect()
        }
    }

    /// Sets the predecessor's address.
    pub fn set_predecessor(&mut self, new_pred: T) {
        self.predecessor = IdentifierValue::new(new_pred);
    }

    /// Sets the current successor.
    pub fn set_successor(&mut self, new_succ: T) {
        self.successor = IdentifierValue::new(new_succ);

        // update finger table so that all fingers closer than successor point to successor
        let diff = self.successor.identifier() - self.current.identifier();

        for i in diff.leading_zeros() as usize..self.finger_table.len() {
            self.finger_table[i] = self.successor.clone();
        }
    }

    /// Sets the finger for the given index.
    pub fn set_finger(&mut self, index: usize, finger: T) {
        self.finger_table[index] = IdentifierValue::new(finger);
    }

    /// Returns the number of fingers.
    pub fn fingers(&self) -> usize {
        self.finger_table.len()
    }

    /// Checks whether this peer is responsible for the given identifier.
    pub fn responsible_for(&self, identifier: Identifier) -> bool {
        identifier.is_between(
            &self.predecessor.identifier(),
            &self.current.identifier()
        )
    }

    /// Returns the peer closest to the given identifier.
    pub fn closest_peer(&self, identifier: Identifier) -> &IdentifierValue<T> {
        if self.responsible_for(identifier) {
            return &self.current;
        }

        let diff = identifier - self.current.identifier();
        let zeros = diff.leading_zeros() as usize;

        self.finger_table.get(zeros).unwrap_or(&self.successor)
    }
}
