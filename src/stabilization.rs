use routing::identifier::*;
use routing::Routing;
use std::net::SocketAddr;
use routines::Routines;

/// Stabilize the [`Routing`] table in regular intervals
///
/// [`Routing`]: ../routing/struct.Routing.html
pub struct Stabilization {
    routines: Routines,
    routing: Routing<SocketAddr>
}

impl Stabilization {
    /// Initializes the stabilization by providing the peer's own address,
    /// the address of a bootstrapping peer and the number of fingers that
    /// should be stored.
    pub fn new(current_addr: SocketAddr, boot_addr: SocketAddr, fingers: usize, timeout: u64)
       -> ::Result<Self>
    {
        let routines = Routines::new(timeout);

        let successor = routines.find_peer(current_addr.identifier(), boot_addr)?;
        let predecessor = routines.get_predecessor(successor)?;
        let finger_table = vec![current_addr; fingers];

        let routing = Routing::new(current_addr, predecessor, successor, finger_table);

        Ok(Self { routines, routing })
    }

    pub fn stabilize(&mut self) -> ::Result<()> {
        let current_id = self.routing.current.identifier();
        let successor_id = self.routing.successor.identifier();
        let new_successor = self.routines.get_predecessor(*self.routing.successor)?;

        if new_successor.identifier().is_between(&current_id, &successor_id) {
            self.routines.set_predecessor(new_successor)?;
            self.routing.set_successor(new_successor);
        }

        Ok(())
    }

    pub fn update_fingers(&mut self) -> ::Result<()> {
        let current_id = self.routing.current.identifier();

        for (i, finger) in self.routing.finger_table.iter_mut().enumerate() {
            // TODO do not hardcode for 256 bits here
            let identifier = current_id + Identifier::with_bit(255 - i);
            let peer = self.routines.find_peer(identifier, *self.routing.successor)?;

            *finger = IdentifierValue::new(peer);
        }

        Ok(())
    }
}
