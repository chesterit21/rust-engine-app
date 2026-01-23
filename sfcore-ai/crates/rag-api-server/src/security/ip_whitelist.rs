use anyhow::Result;
use ipnetwork::IpNetwork;
use std::net::IpAddr;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, info, warn};

#[derive(Debug, Clone)]
pub struct IpWhitelist {
    allowed_networks: Arc<RwLock<Vec<IpNetwork>>>,
    config_path: PathBuf,
}

impl IpWhitelist {
    /// Create new IP whitelist dari config file
    pub fn new(config_path: PathBuf, allowed_ips: Vec<String>) -> Result<Self> {
        let networks = Self::parse_ip_list(&allowed_ips)?;
        
        let whitelist = Self {
            allowed_networks: Arc::new(RwLock::new(networks)),
            config_path,
        };
        
        Ok(whitelist)
    }
    
    /// Parse IP list (support single IP, range, CIDR)
    fn parse_ip_list(ips: &[String]) -> Result<Vec<IpNetwork>> {
        let mut networks = Vec::new();
        
        for ip_str in ips {
            let ip_str = ip_str.trim();
            
            // Try parse as CIDR first
            match ip_str.parse::<IpNetwork>() {
                Ok(network) => {
                    networks.push(network);
                    debug!("Added network: {}", network);
                }
                Err(_) => {
                    // Try parse as single IP
                    if let Ok(ip) = ip_str.parse::<IpAddr>() {
                        let network = match ip {
                            IpAddr::V4(ipv4) => IpNetwork::V4(
                                ipnetwork::Ipv4Network::new(ipv4, 32).unwrap()
                            ),
                            IpAddr::V6(ipv6) => IpNetwork::V6(
                                ipnetwork::Ipv6Network::new(ipv6, 128).unwrap()
                            ),
                        };
                        networks.push(network);
                        debug!("Added single IP: {}", ip);
                    } else {
                        warn!("Invalid IP/CIDR format: {}", ip_str);
                    }
                }
            }
        }
        
        Ok(networks)
    }
    
    /// Check if IP is allowed
    pub async fn is_allowed(&self, ip: IpAddr) -> bool {
        let networks = self.allowed_networks.read().await;
        
        for network in networks.iter() {
            if network.contains(ip) {
                debug!("IP {} matched network {}", ip, network);
                return true;
            }
        }
        
        warn!("IP {} not in whitelist", ip);
        false
    }
    
    /// Reload whitelist dari file (manual trigger)
    pub async fn reload(&self, new_ips: Vec<String>) -> Result<()> {
        let networks = Self::parse_ip_list(&new_ips)?;
        
        let mut allowed = self.allowed_networks.write().await;
        *allowed = networks;
        
        info!("IP whitelist reloaded: {} entries", allowed.len());
        Ok(())
    }
    
    /// Start file watcher untuk hot-reload (placeholder - implement later)
    pub fn start_watcher(self) -> Result<()> {
        // TODO: Implement file watcher dengan notify crate
        // Untuk sekarang skip dulu
        info!("File watcher disabled (not implemented yet)");
        Ok(())
    }
    
    /// Get current whitelist (untuk debugging)
    pub async fn get_whitelist(&self) -> Vec<String> {
        let networks = self.allowed_networks.read().await;
        networks.iter().map(|n| n.to_string()).collect()
    }
}
