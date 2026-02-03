use anyhow::{Result, Context};
use mc_connect_core::models::packet::{AllowedPort, Protocol};

pub fn parse_allowed_ports(input: &str) -> Result<Vec<AllowedPort>> {
    let mut ports = Vec::new();
    for part in input.split(',') {
        let part = part.trim();
        if part.is_empty() { continue; }
        
        let subparts: Vec<&str> = part.split(':').collect();
        if subparts.len() != 2 {
            return Err(anyhow::anyhow!("Invalid format: {}. Expected 'port:protocol'", part));
        }
        
        let port: u16 = subparts[0].parse().with_context(|| format!("Invalid port: {}", subparts[0]))?;
        let protocol = match subparts[1].to_lowercase().as_str() {
            "tcp" => Protocol::TCP,
            "udp" => Protocol::UDP,
            _ => return Err(anyhow::anyhow!("Unsupported protocol: {}", subparts[1])),
        };
        
        ports.push(AllowedPort { port, protocol });
    }
    ports.sort_by_key(|p| p.port);
    Ok(ports)
}
