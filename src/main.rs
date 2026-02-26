use axum::{routing::get, Router};
use mdns_sd::{ServiceDaemon, ServiceInfo};
use rustls_pki_types::{CertificateDer, PrivateKeyDer};
use std::{net::SocketAddr, sync::Arc, collections::HashMap};
use tokio::net::TcpListener;
use tokio_rustls::TlsAcceptor;
use tower::Service; // Import the trait directly

#[tokio::main]
async fn main() {
    // 1. Generate Certs
    let cert = rcgen::generate_simple_self_signed(vec!["rust-cyber-lab.local".to_string()]).unwrap();
    let cert_der = CertificateDer::from(cert.serialize_der().unwrap());
    let key_der = PrivateKeyDer::Pkcs8(cert.serialize_private_key_der().into());

    let server_config = rustls::ServerConfig::builder()
        .with_no_client_auth()
        .with_single_cert(vec![cert_der], key_der)
        .expect("Failed to build Rustls config");
    let acceptor = TlsAcceptor::from(Arc::new(server_config));

    // 2. mDNS Setup
    let mdns = ServiceDaemon::new().expect("Failed to create mDNS daemon");
    let my_service = ServiceInfo::new(
        "_http._tcp.local.",
        "rust-cyber-lab",
        "rust-cyber-lab.local.",
        "0.0.0.0", 
        8443,
        HashMap::new(),
    ).expect("valid service info");
    mdns.register(my_service).unwrap();

    // 3. Router
    let app = Router::new().route("/", get(|| async { "Bypass Active: Encrypted Lab Online." }));
    let addr = SocketAddr::from(([0, 0, 0, 0], 8443));
    let listener = TcpListener::bind(addr).await.unwrap();

    println!("Ghost Server running on https://rust-cyber-lab.local:8443");

    loop {
        let (stream, _) = listener.accept().await.unwrap();
        let acceptor = acceptor.clone();
        let app = app.clone();

        tokio::spawn(async move {
            if let Ok(tls_stream) = acceptor.accept(stream).await {
                // We use hyper_util to bridge Axum to the TLS stream
                let _ = hyper_util::server::conn::auto::Builder::new(hyper_util::rt::TokioExecutor::new())
                    .serve_connection(
                        hyper_util::rt::TokioIo::new(tls_stream),
                        hyper_util::service::TowerToHyperService::new(app),
                    )
                    .await;
            }
        });
    }
}
