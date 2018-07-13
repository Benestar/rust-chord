//! Custom error types
//!
//! The [`MessageError`] is currently the only custom error type. It can be used
//! when an unexpected message has been received.
//!
//! [`MessageError`]: struct.MessageError.html

use message::Message;
use std::error::Error;
use std::fmt;

/// Error type to use when an unexpected message has been received
///
/// A specific [`Message`] object needs to be available for this error type.
/// If no valid message has been received yet, one should use a different
/// error type like [`io::Error`].
///
/// [`Message`]: message/enum.Message.html
/// [`io::Error`]: ../std/io/struct.Error.html
#[derive(Debug)]
pub struct MessageError {
    msg: Message,
    error: Box<Error + Send + Sync>
}

impl MessageError {
    /// Creates a new message error from an existing message as well as an
    /// arbitrary error payload.
    ///
    /// # Examples
    ///
    /// ```
    /// # use dht::error::MessageError;
    /// # use dht::message::{Message, p2p};
    /// # let msg = Message::PredecessorGet(p2p::PredecessorGet);
    /// #
    /// let result = if let Message::DhtSuccess(_) = msg {
    ///     Ok("yay")
    /// } else {
    ///     Err(MessageError::new(msg, "unexpected message type"))
    /// };
    /// ```
    pub fn new<E>(msg: Message, error: E) -> Self
        where E: Into<Box<Error + Send + Sync>>
    {
        MessageError { msg, error: error.into() }
    }
}

impl fmt::Display for MessageError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Message error ({})\n\n{:?}", self.error, self.msg)
    }
}

impl Error for MessageError {
    fn description(&self) -> &str {
        "Message error"
    }

    fn cause(&self) -> Option<&Error> {
        Some(self.error.as_ref())
    }
}
