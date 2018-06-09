use message::Message;
use std::error::Error;
use std::fmt;

#[derive(Debug)]
pub struct MessageError {
    msg: Message,
    error: Box<Error + Send + Sync>
}

impl MessageError {
    pub fn new<E>(msg: Message, error: E) -> Self
        where E: Into<Box<Error + Send + Sync>>
    {
        MessageError { msg, error: error.into() }
    }
}

impl fmt::Display for MessageError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        // TODO implement display on message type
        write!(f, "Message error for message TODO ({})", self.error)
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
