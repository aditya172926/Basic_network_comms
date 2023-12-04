use std::net::UdpSocket;

use mac_address::MacAddressError;

mod structs;

fn main() {
    println!("Hello, world!");
    let socket = UdpSocket::bind(&local_addr).await?;
    socket.set_broadcast(true)?;
}

fn get_mac_address() -> Result<String, MacAddressError> {
    let mac = mac_address::get_mac_address()?;
    match mac {
        Some(address) => Ok(address.to_string()),
        None => Err(MacAddressError::InternalError)
    }
}
