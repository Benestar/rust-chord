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
        let finger_table = vec![self.current_addr; self.fingers];

        Ok(Routing::new(self.current_addr, self.current_addr, successor, finger_table))
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
        info!("Stabilizing routing information");

        let update_successor = self.update_successor();
        let update_fingers = self.update_fingers();

        let routing = self.routing.lock().unwrap();

        debug!("Current routing information:\n\n{:#?}", *routing);

        update_successor.and(update_fingers)
    }

    fn update_successor(&self) -> ::Result<()> {
        let (current, successor) = {
            let routing = self.routing.lock().unwrap();

            (routing.current, routing.successor)
        };

        info!("Obtaining new successor from current successor with address {}", *successor);

        let new_successor = self.procedures.notify_predecessor(*current, *successor)?;

        let current_id = current.identifier();
        let successor_id = successor.identifier();

        if new_successor.identifier().is_between(&current_id, &successor_id) {
            info!("Updating successor to address {}", new_successor);

            let mut routing = self.routing.lock().unwrap();
            routing.set_successor(new_successor);
        }

        Ok(())
    }

    fn update_fingers(&self) -> ::Result<()> {
        let (current, successor, fingers) = {
            let routing = self.routing.lock().unwrap();

            (routing.current, routing.successor, routing.fingers())
        };

        info!("Update fingers using successor with address {}", *successor);

        for i in 0..fingers {
            // TODO do not hardcode for 256 bits here
            let identifier = current.identifier() + Identifier::with_bit(255 - i);
            let peer = self.procedures.find_peer(identifier, *successor)?;

            let mut routing = self.routing.lock().unwrap();
            routing.set_finger(i, peer);
        }

        Ok(())
    }
}
