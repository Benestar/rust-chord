use routing::identifier::*;

pub mod identifier;

pub struct Routing<T> {
    current_ip: IdentifierValue<T>,
    // TODO should maybe be an Option
    predecessor: IdentifierValue<T>,
    // TODO use BinaryHeap for multiple successors
    successor: IdentifierValue<T>
}

impl<T: Identify> Routing<T> {
    pub fn new(current_ip: T, predecessor: T, successor: T) -> Self {
        Routing {
            current_ip: IdentifierValue::new(current_ip),
            predecessor: IdentifierValue::new(predecessor),
            successor: IdentifierValue::new(successor)
        }
    }

    pub fn get_current_ip(&self) -> &T {
        self.current_ip.get_value()
    }

    pub fn get_predecessor(&self) -> &T {
        self.predecessor.get_value()
    }

    pub fn set_predecessor(&mut self, new_pred: T) {
        self.predecessor = IdentifierValue::new(new_pred);
    }

    pub fn is_closer_predecessor(&self, new_pred: &T) -> bool {
        new_pred.get_identifier().is_between(
            self.predecessor.get_identifier(),
            self.current_ip.get_identifier()
        )
    }

    pub fn get_successor(&self) -> &T {
        self.successor.get_value()
    }

    pub fn set_successor(&mut self, new_succ: T) {
        self.successor = IdentifierValue::new(new_succ);
    }

    pub fn responsible_for(&self, identifier: &Identifier) -> bool {
        identifier.is_between(
            self.predecessor.get_identifier(),
            self.current_ip.get_identifier()
        )
    }
}