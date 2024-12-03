use std::net::{Ipv4Addr, Ipv6Addr};

use axum::extract::Query;
use serde::Deserialize;

#[derive(Deserialize)]
pub(super) struct EgregiousEncryptionQueryParams {
    from: Ipv4Addr,
    key: Ipv4Addr,
}

pub(super) async fn egregious_encryption(query: Query<EgregiousEncryptionQueryParams>) -> String {
    // Get the raw bytes of the "from" address
    let from_bytes = query.from.octets();
    // Get the raw bytes of the "key" address
    let key_bytes = query.key.octets();
    // Make an array to store the resulting destination address bytes
    let mut dest_bytes: [u8; 4] = [0; 4];
    for (i, (from_byte, key_byte)) in from_bytes.iter().zip(key_bytes.iter()).enumerate() {
        // Use the wrapping add to calculate the destination byte
        dest_bytes[i] = from_byte.wrapping_add(*key_byte);
    }
    // Convert the destination bytes to an Ipv4Addr and then to a string
    Ipv4Addr::from(dest_bytes).to_string()
}

#[derive(Deserialize)]
pub(super) struct GoingTheOtherWayQueryParams {
    from: Ipv4Addr,
    to: Ipv4Addr,
}

pub(super) async fn going_the_other_way(query: Query<GoingTheOtherWayQueryParams>) -> String {
    // Get the raw bytes of the "from" address
    let from_bytes = query.from.octets();
    // Get the raw bytes of the "to" address
    let to_bytes = query.to.octets();
    // Make an array to store the resulting destination address bytes
    let mut key_bytes: [u8; 4] = [0; 4];
    for (i, (from_byte, to_byte)) in from_bytes.iter().zip(to_bytes.iter()).enumerate() {
        // Use the wrapping add to calculate the destination byte
        key_bytes[i] = to_byte.wrapping_sub(*from_byte);
    }
    // Convert the destination bytes to an Ipv4Addr and then to a string
    Ipv4Addr::from(key_bytes).to_string()
}

#[derive(Deserialize)]
pub(super) struct V6DestQueryParams {
    from: Ipv6Addr,
    key: Ipv6Addr,
}

pub(super) async fn v6_dest(query: Query<V6DestQueryParams>) -> String {
    // Get the raw bytes of the "from" address
    let from_bytes = query.from.octets();
    // Get the raw bytes of the "key" address
    let key_bytes = query.key.octets();
    // Make an array to store the resulting destination address bytes
    let mut dest_bytes: [u8; 16] = [0; 16];
    for (i, (from_byte, key_byte)) in from_bytes.iter().zip(key_bytes.iter()).enumerate() {
        // Use the wrapping add to calculate the destination byte
        dest_bytes[i] = from_byte ^ key_byte;
    }
    // Convert the destination bytes to an Ipv4Addr and then to a string
    Ipv6Addr::from(dest_bytes).to_string()
}

#[derive(Deserialize)]
pub(super) struct V6KeyQueryParams {
    from: Ipv6Addr,
    to: Ipv6Addr,
}
pub(super) async fn v6_key(query: Query<V6KeyQueryParams>) -> String {
    // Get the raw bytes of the "from" address
    let from_bytes = query.from.octets();
    // Get the raw bytes of the "to" address
    let to_bytes = query.to.octets();
    // Make an array to store the resulting destination address bytes
    let mut key_bytes: [u8; 16] = [0; 16];
    for (i, (from_byte, to_byte)) in from_bytes.iter().zip(to_bytes.iter()).enumerate() {
        // Use the wrapping add to calculate the destination byte
        key_bytes[i] = to_byte ^ from_byte;
    }
    // Convert the destination bytes to an Ipv6Addr and then to a string
    Ipv6Addr::from(key_bytes).to_string()
}
