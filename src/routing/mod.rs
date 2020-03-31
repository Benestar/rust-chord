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
///
/// The type parameter `T` is used to describe the identifying property of a
/// peer, for example its socket address.
#[derive(Debug)]
pub struct Routing<T> {
    /// Address where this peer is listening for peer-to-peer messages
    pub current: IdentifierValue<T>,
    /// Closest predecessor of this pee
    pub predecessor: IdentifierValue<T>,
    /// Successor of this peer
    // TODO use BinaryHeap for multiple successors
    pub successor: IdentifierValue<T>,
    /// The finger table of this peer with pointers accross the network
    finger_table: Vec<IdentifierValue<T>>,
}

impl<T: Identify + Clone> Routing<T> {
    /// Creates a new `Routing` instance for the given initial values.
    pub fn new(current: T, predecessor: T, successor: T, finger_table: Vec<T>) -> Self {
        Self {
            current: IdentifierValue::new(current),
            predecessor: IdentifierValue::new(predecessor),
            successor: IdentifierValue::new(successor),
            finger_table: finger_table.into_iter().map(IdentifierValue::new).collect(),
        }
    }

    /// Sets the predecessor's address.
    pub fn set_predecessor(&mut self, new_pred: T) {
        self.predecessor = IdentifierValue::new(new_pred);
    }

    /// Sets the current successor.
    pub fn set_successor(&mut self, new_succ: T) {
        self.successor = IdentifierValue::new(new_succ);
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
        identifier.is_between(&self.predecessor.identifier(), &self.current.identifier())
    }

    /// Checks whether this peer's successor is responsible for the given identifier.
    pub fn successor_responsible_for(&self, identifier: Identifier) -> bool {
        identifier.is_between(&self.current.identifier(), &self.successor.identifier())
    }

    /// Returns the peer closest to the given identifier.
    pub fn closest_peer(&self, identifier: Identifier) -> &IdentifierValue<T> {
        if self.responsible_for(identifier) {
            return &self.current;
        }

        if self.successor_responsible_for(identifier) {
            return &self.successor;
        }

        let diff = identifier - self.current.identifier();
        let zeros = diff.leading_zeros() as usize;

        self.finger_table.get(zeros).unwrap_or(&self.successor)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use routing::identifier::Identifier;
    use std::net::SocketAddr;
    
    #[test]
    fn new_routing() {
        let current: SocketAddr = "192.168.0.1:123".parse().unwrap();
        let predecessor: SocketAddr = "192.168.0.3:456".parse().unwrap();
        let successor: SocketAddr = "192.168.0.2:789".parse().unwrap();

        let finger_table = vec![
            "192.168.0.4:8080".parse().unwrap(),
            "192.168.0.5:8080".parse().unwrap(),
            "192.168.0.6:8080".parse().unwrap(),
            "192.168.0.7:8080".parse().unwrap(),
        ];

        let routing = Routing::new(current, predecessor, successor, finger_table.clone());

        assert_eq!(current, *routing.current);
        assert_eq!(predecessor, *routing.predecessor);
        assert_eq!(successor, *routing.successor);

        for (expected, finger) in finger_table.iter().zip(routing.finger_table) {
            assert_eq!(*expected, *finger);
        }
    }

    #[test]
    fn set_predecessor() {
        let addr: SocketAddr = "127.0.0.1:8080".parse().unwrap();
        let mut routing = Routing::new(addr, addr, addr, Vec::new());

        let predecessor: SocketAddr = "192.168.0.1:1234".parse().unwrap();
        routing.set_predecessor(predecessor);

        assert_eq!(predecessor, *routing.predecessor);
    }

    #[test]
    fn set_successor() {
        let addr: SocketAddr = "127.0.0.1:8080".parse().unwrap();
        let mut routing = Routing::new(addr, addr, addr, Vec::new());

        let succecessor: SocketAddr = "192.168.0.1:1234".parse().unwrap();
        routing.set_succecessor(succecessor);

        assert_eq!(succecessor, *routing.succecessor);
    }
}
