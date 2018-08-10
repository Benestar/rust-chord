//! This module allows to obtain identifiers from different data structures.
//!
//! The [`Identifier`] struct represents a 256 bit identifier obtained using
//! the SHA256 hashing algorithm. The identifier is meant to be part of an
//! identifier circle consisting of all non-negative integers modulo 2^256.
//!
//! Using the [`Identify`] trait, different data structures like ip addresses
//! can be associated with an identifier and stored accordingly.
//!
//! The [`IdentifierValue`] struct stores the identifier along with the original
//! value to avoid having to recalculate the hash value multiple times.
//!
//! [`Identifier`]: struct.Identifier.html
//! [`Identify`]: trait.Identify.html
//! [`IdentifierValue`]: struct.IdentifierValue.html

use bigint::U256;
use ring::digest;
use std::fmt;
use std::net::{SocketAddr, SocketAddrV4, SocketAddrV6};
use std::ops::{Add, Sub};
use std::ops::Deref;
use storage::Key;

/// A 256 bit identifier on an identifier circle
#[derive(Copy, Clone, PartialEq)]
pub struct Identifier(U256);

impl Identifier {
    /// Creates a new identifier from a byte slice.
    ///
    /// This method does not perform any hashing but interprets the bytes as
    /// a raw identifier.
    ///
    /// # Panics
    ///
    /// Panics if the slice does not contain exactly 32 elements.
    pub fn new(identifier: &[u8]) -> Self {
        Identifier(U256::from_big_endian(identifier))
    }

    /// Create a new identifier with the given bit set.
    ///
    /// # Panics
    ///
    /// Panics if `index` exceeds the bit width of the number.
    pub fn with_bit(index: usize) -> Self {
        Identifier(U256::one() << index)
    }

    fn generate(bytes: &[u8]) -> Self {
        let dig = digest::digest(&digest::SHA256, bytes);
        Self::new(dig.as_ref())
    }

    /// Returns whether this identifier is between `first` and `second` on the
    /// identifier circle.
    ///
    /// The first identifier is excluded from the range while the second one is included.
    /// This method can be written as `self ∈ (first, second]` if `first ≤ second` and as
    /// `self ∈ [0, first) ∪ [second, 0]` if `first > second`.
    ///
    /// # Examples
    ///
    /// ```
    /// # use dht::routing::identifier::Identifier;
    /// #
    /// let id1 = Identifier::new(&[1; 32]);
    /// let id2 = Identifier::new(&[2; 32]);
    /// let id3 = Identifier::new(&[3; 32]);
    ///
    /// assert!(id2.is_between(&id1, &id3));
    /// assert!(id3.is_between(&id2, &id1));
    /// assert!(!id3.is_between(&id1, &id2));
    /// ```
    pub fn is_between(&self, first: &Identifier, second: &Identifier) -> bool {
        let (diff1, _) = second.0.overflowing_sub(self.0);
        let (diff2, _) = second.0.overflowing_sub(first.0);

        diff1 < diff2
    }

    /// Returns the binary logarithm of this identifier minus the given offset.
    ///
    /// The result is the floor of the actual result.
    ///
    /// # Examples
    ///
    /// ```
    /// # use dht::routing::identifier::Identifier;
    /// #
    /// let identifier = Identifier::new(&[
    ///     0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    ///     5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5,
    /// ]);
    ///
    /// assert_eq!(133, identifier.leading_zeros());
    /// ```
    pub fn leading_zeros(&self) -> u32 {
        self.0.leading_zeros()
    }

    /// Returns the raw bytes of this identifier.
    ///
    /// # Examples
    ///
    /// ```
    /// # use dht::routing::identifier::Identifier;
    /// #
    /// let id = Identifier::new(&[5; 32]);
    ///
    /// assert_eq!([5; 32], id.as_bytes());
    /// ```
    pub fn as_bytes(&self) -> [u8; 32] {
        let mut bytes = [0; 32];
        self.0.to_big_endian(&mut bytes);
        bytes
    }
}

/// Implement overflowing addition for identifiers
impl Add for Identifier {
    type Output = Self;

    fn add(self, other: Self) -> Self {
        let (sum, _) = self.0.overflowing_add(other.0);

        Identifier(sum)
    }
}

/// Implement overflowing subtraction for identifiers
impl Sub for Identifier {
    type Output = Self;

    fn sub(self, other: Self) -> Self {
        let (diff, _) = self.0.overflowing_sub(other.0);

        Identifier(diff)
    }
}

impl fmt::Display for Identifier {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "‹{}›", self.0)
    }
}

impl fmt::Debug for Identifier {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let bytes = self.as_bytes();
        let mut iter = bytes.iter();

        write!(f, "‹{:02x}", iter.next().unwrap())?;

        for byte in iter {
            write!(f, ":{:02x}", byte)?;
        }

        write!(f, "›")
    }
}

/// Trait to obtain an identifier from a data structure
pub trait Identify {
    /// Generates an identifier for this object.
    fn identifier(&self) -> Identifier;
}

/// Obtains an identifier by hashing the four octets of the ip address.
impl Identify for SocketAddrV4 {
    fn identifier(&self) -> Identifier {
        Identifier::generate(self.ip().octets().as_ref())
    }
}

/// Obtains an identifier by hashing the first eight octets of the ip address.
impl Identify for SocketAddrV6 {
    fn identifier(&self) -> Identifier {
        Identifier::generate(self.ip().octets()[..8].as_ref())
    }
}

/// Get the identifier for a V4 or V6 socket address.
impl Identify for SocketAddr {
    fn identifier(&self) -> Identifier {
        match self {
            SocketAddr::V4(v4) => v4.identifier(),
            SocketAddr::V6(v6) => v6.identifier()
        }
    }
}

/// Hashes the raw key and its replication index.
impl Identify for Key {
    fn identifier(&self) -> Identifier {
        let mut bytes = [0; 33];
        bytes[..32].copy_from_slice(&self.raw_key);
        bytes[32] = self.replication_index;
        Identifier::generate(&bytes)
    }
}

/// Container for a value and its identifier
#[derive(Clone, Copy, Debug)]
pub struct IdentifierValue<T> {
    value: T,
    identifier: Identifier
}

impl<T: Identify> IdentifierValue<T> {
    /// Obtains the identifier for the given `value` and stores it along with
    /// the value in an `IdentifierValue` object.
    pub fn new(value: T) -> Self {
        let identifier = value.identifier();

        Self { value, identifier }
    }

    /// Returns the identifier obtained during the creation of this object.
    ///
    /// # Examples
    ///
    /// ```
    /// # use dht::routing::identifier::{IdentifierValue, Identify};
    /// # use std::net::SocketAddr;
    /// #
    /// let value: SocketAddr = "127.0.0.1:8080".parse().unwrap();
    ///
    /// let idv = IdentifierValue::new(value);
    ///
    /// assert_eq!(value.identifier(), idv.identifier());
    /// ```
    pub fn identifier(&self) -> Identifier {
        self.identifier
    }
}

impl<T> Deref for IdentifierValue<T> {
    type Target = T;

    fn deref(&self) -> &<Self as Deref>::Target {
        &self.value
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn identifier_with_bit() {
        let bytes = [
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1,
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        ];

        let expected = Identifier::new(&bytes);

        assert_eq!(expected, Identifier::with_bit(128));
    }

    #[test]
    fn is_between() {
        let id1 = Identifier::new(&[1; 32]);
        let id2 = Identifier::new(&[2; 32]);
        let id3 = Identifier::new(&[3; 32]);

        assert!(id1.is_between(&id3, &id2));
        assert!(!id1.is_between(&id2, &id3));

        assert!(id2.is_between(&id1, &id3));
        assert!(!id2.is_between(&id3, &id1));

        assert!(id3.is_between(&id2, &id1));
        assert!(!id3.is_between(&id1, &id2));
    }

    #[test]
    fn is_between_edges() {
        let id1 = Identifier::new(&[1; 32]);
        let id2 = Identifier::new(&[2; 32]);

        // the right id is included while the left one is excluded
        assert!(id1.is_between(&id2, &id1));
        assert!(!id1.is_between(&id1, &id2));

        // there is nothing between an id and itself
        assert!(!id2.is_between(&id1, &id1));
        assert!(!id1.is_between(&id1, &id1));
    }

    #[test]
    fn leading_zeros() {
        let identifier = Identifier::new(&[
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5,
        ]);

        assert_eq!(133, identifier.leading_zeros());
    }

    #[test]
    fn leading_zeros_min() {
        let identifier = Identifier::new(&[0; 32]);

        assert_eq!(256, identifier.leading_zeros());
    }

    #[test]
    fn leading_zeros_max() {
        let identifier = Identifier::new(&[0xff; 32]);

        assert_eq!(0, identifier.leading_zeros());
    }

    #[test]
    fn identifier_add() {
        let id1 = Identifier::new(&[1; 32]);
        let id2 = Identifier::new(&[2; 32]);
        let id3 = Identifier::new(&[3; 32]);

        assert_eq!(id3, id1 + id2);
    }

    #[test]
    fn identifier_add_overflow() {
        let id1 = Identifier::new(&[0xff; 32]);
        let id2 = Identifier::with_bit(0);
        let id3 = Identifier::new(&[0; 32]);

        assert_eq!(id3, id1 + id2);
    }

    #[test]
    fn identifier_sub() {
        let id1 = Identifier::new(&[1; 32]);
        let id2 = Identifier::new(&[2; 32]);
        let id3 = Identifier::new(&[3; 32]);

        assert_eq!(id1, id3 - id2);
    }

    #[test]
    fn identifier_sub_overflow() {
        let id1 = Identifier::new(&[0xff; 32]);
        let id2 = Identifier::with_bit(0);
        let id3 = Identifier::new(&[0; 32]);

        assert_eq!(id1, id3 - id2);
    }
}
