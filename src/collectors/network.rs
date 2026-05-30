//! Network data collector
//! Handles collection of network interface statistics

use crate::types::*;
use sysinfo::Networks;

/// Collect network interface information
pub fn collect_network_info(networks: &Networks) -> Vec<NetworkInterface> {
    networks
        .iter()
        .map(|(name, data)| NetworkInterface {
            name: name.to_string(),
            bytes_received: data.total_received(),
            bytes_transmitted: data.total_transmitted(),
            packets_received: data.packets_received(),
            packets_transmitted: data.packets_transmitted(),
        })
        .collect()
}
