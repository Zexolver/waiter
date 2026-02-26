use axum::{routing::get, Router};
use mdns_sd::{ServiceDaemon, ServiceInfo};
use std::collections::HashMap;

#[tokio::main]
async fn main() {
    // 1. Start mDNS Discovery
    let mdns = ServiceDaemon::new().expect("Failed to create mDNS daemon");
    let service_type = "_http._tcp.local.";
    let instance_name = "rust-cyber-lab"; // This becomes rust-cyber-lab.local
    let host_name = "rust-cyber-lab.local.";
    let port = 8080;
    
    // Fill in your PC's LAN IP or leave it to the lib to find
    let properties: HashMap<String, String> = HashMap::new();
    let my_service = ServiceInfo::new(
        service_type,
        instance_name,
        host_name,
        "192.168.1.50", // Change to your school PC's static LAN IP
        port,
        properties,
    ).expect("valid service info");

    mdns.register(my_service).expect("Failed to register mDNS service");

    // 2. Start the Web Server
    let app = Router::new().route("/", get(|| async { "Hello from the Rust Lab!" }));
    let listener = tokio::net::TcpListener::bind("0.0.0.0:8080").await.unwrap();
    
    println!("Server running on http://{}.local:{}", instance_name, port);
    axum::serve(listener, app).await.unwrap();
}
