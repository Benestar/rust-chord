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
use std::fmt::{self, Debug};
use std::net::{SocketAddr, SocketAddrV4, SocketAddrV6};
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

    fn generate(bytes: &[u8]) -> Self {
        let dig = digest::digest(&digest::SHA256, bytes);
        Self::new(dig.as_ref())
    }

    /// Returns whether this identifier is between `first` and `second` on the
    /// identifier circle.
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

    /// Calculate the distance to the given offset in positive direction.
    ///
    /// # Examples
    ///
    /// ```
    /// # use dht::routing::identifier::Identifier;
    /// #
    /// let id1 = Identifier::new(&[5; 32]);
    /// let id2 = Identifier::new(&[1; 32]);
    ///
    /// let offset = Identifier::new(&[4; 32]);
    ///
    /// assert_eq!(offset, id1.offset(&id2));
    /// ```
    pub fn offset(&self, base: &Identifier) -> Identifier {
        let (diff, _) = self.0.overflowing_sub(base.0);

        Identifier(diff)
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

impl Debug for Identifier {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        (self.0).0.fmt(f)
    }
}

/// Trait to obtain an identifier from a data structure
pub trait Identify {
    /// Generates an identifier for this object.
    fn get_identifier(&self) -> Identifier;
}

/// Obtains an identifier by hashing the four octets of the ip address.
impl Identify for SocketAddrV4 {
    fn get_identifier(&self) -> Identifier {
        Identifier::generate(self.ip().octets().as_ref())
    }
}

/// Obtains an identifier by hashing the first 16 octets of the ip address.
impl Identify for SocketAddrV6 {
    fn get_identifier(&self) -> Identifier {
        Identifier::generate(self.ip().octets().as_ref())
    }
}

/// Get the identifier for a V4 or V6 socket address.
impl Identify for SocketAddr {
    fn get_identifier(&self) -> Identifier {
        match self {
            SocketAddr::V4(v4) => v4.get_identifier(),
            SocketAddr::V6(v6) => v6.get_identifier()
        }
    }
}

/// Hashes the raw key and its replication index.
impl Identify for Key {
    fn get_identifier(&self) -> Identifier {
        let mut bytes = [0; 33];
        bytes[..32].copy_from_slice(&self.raw_key);
        bytes[32] = self.replication_index;
        Identifier::generate(&bytes)
    }
}

/// Container for a value and its identifier
pub struct IdentifierValue<T> {
    value: T,
    identifier: Identifier
}

impl<T: Identify> IdentifierValue<T> {
    /// Obtains the identifier for the given `value` and stores it along with
    /// the value in an `IdentifierValue` object.
    pub fn new(value: T) -> Self {
        let identifier = value.get_identifier();

        Self { value, identifier }
    }

    /// Returns the value in this struct.
    ///
    /// # Examples
    ///
    /// ```
    /// # use dht::routing::identifier::IdentifierValue;
    /// #
    /// let idv = IdentifierValue::new([4; 32]);
    ///
    /// assert_eq!([4; 32], *idv.get_value());
    /// ```
    pub fn get_value(&self) -> &T {
        &self.value
    }

    /// Returns the identifier obtained during the creation of this object.
    ///
    /// # Examples
    ///
    /// ```
    /// # use dht::routing::identifier::{IdentifierValue, Identify};
    /// #
    /// let idv = IdentifierValue::new([4; 32]);
    ///
    /// assert_eq!([4; 32].get_identifier(), *idv.get_identifier());
    /// ```
    pub fn get_identifier(&self) -> &Identifier {
        &self.identifier
    }
}
