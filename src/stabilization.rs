use routing::identifier::*;
use routing::Routing;
use std::net::SocketAddr;
use procedures::Procedures;
use std::sync::Mutex;
use std::sync::Arc;

pub struct Bootstrap {
    current_addr: SocketAddr,
    boot_addr: SocketAddr,
    fingers: usize
}

impl Bootstrap {
    /// Initializes the bootstrap algorithm by providing the peer's own address,
    /// the address of a bootstrapping peer and the number of fingers that
    /// should be stored.
    pub fn new(current_addr: SocketAddr, boot_addr: SocketAddr, fingers: usize) -> Self {
        Self { current_addr, boot_addr, fingers }
    }

    pub fn bootstrap(&self, timeout: u64) -> ::Result<Routing<SocketAddr>> {
        let procedures = Procedures::new(timeout);
        let current_id = self.current_addr.identifier();

        let successor = procedures.find_peer(current_id, self.boot_addr)?;
        let predecessor = procedures.get_predecessor(successor)?;
        let finger_table = vec![self.current_addr; self.fingers];

        Ok(Routing::new(self.current_addr, predecessor, successor, finger_table))
    }
}

/// Stabilize the [`Routing`] table in regular intervals
///
/// [`Routing`]: ../routing/struct.Routing.html
pub struct Stabilization {
    procedures: Procedures,
    routing: Arc<Mutex<Routing<SocketAddr>>>
}

impl Stabilization {
    pub fn new(routing: Arc<Mutex<Routing<SocketAddr>>>, timeout: u64) -> Self {
        let procedures = Procedures::new(timeout);

        Self { procedures, routing }
    }

    pub fn stabilize(&mut self) -> ::Result<()> {
        let update_successor = self.update_successor();
        let update_fingers = self.update_fingers();

        update_successor.and(update_fingers)
    }

    fn update_successor(&mut self) -> ::Result<()> {
        let mut routing = self.routing.lock().or(Err("Lock is poisoned"))?;

        if *routing.successor == *routing.current {
            // avoid deadlock when requesting own predecessor
            return Ok(())
        }

        let current_id = routing.current.identifier();
        let successor_id = routing.successor.identifier();
        let new_successor = self.procedures.get_predecessor(*routing.successor)?;

        if new_successor.identifier().is_between(&current_id, &successor_id) {
            self.procedures.set_predecessor(new_successor)?;
            routing.set_successor(new_successor);
        }

        Ok(())
    }

    fn update_fingers(&mut self) -> ::Result<()> {
        let mut routing = self.routing.lock().or(Err("Lock is poisoned"))?;

        if *routing.successor == *routing.current {
            // avoid deadlock when requesting own predecessor
            return Ok(())
        }

        let current_id = routing.current.identifier();
        let successor = *routing.successor;

        for (i, finger) in routing.finger_table.iter_mut().enumerate() {
            // TODO do not hardcode for 256 bits here
            let identifier = current_id + Identifier::with_bit(255 - i);
            let peer = self.procedures.find_peer(identifier, successor)?;

            *finger = IdentifierValue::new(peer);
        }

        Ok(())
    }
}
