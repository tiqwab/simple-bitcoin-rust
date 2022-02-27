use crate::connection_manager_core::ConnectionManagerCore;
use crate::message::ApplicationPayload;
use anyhow::Context;
use log::{debug, info};
use std::net::SocketAddr;

pub enum ServerCoreState {
    Init,
    Standby,
    ConnectedToNetwork,
    ShuttingDown,
}

pub struct ServerCore {
    state: ServerCoreState,
    core_node_addr: Option<SocketAddr>,
    cm: ConnectionManagerCore,
}

// An implementation of ApplicationPayloadHandler
fn handle_application_payload(payload: ApplicationPayload, _peer: Option<SocketAddr>) {
    debug!("handle_application_payload: {:?}", payload);
    match payload {
        ApplicationPayload::NewTransaction => {}
        ApplicationPayload::NewBlock => {}
        ApplicationPayload::RequestFullChain => {}
        ApplicationPayload::FullChain => {}
        ApplicationPayload::Enhanced { .. } => {}
    }
}

impl ServerCore {
    pub fn new(my_addr: SocketAddr, core_node_addr: Option<SocketAddr>) -> ServerCore {
        info!("Initializing ServerCore...");
        ServerCore {
            state: ServerCoreState::Init,
            core_node_addr,
            cm: ConnectionManagerCore::new(my_addr, handle_application_payload),
        }
    }

    pub async fn start(&mut self) {
        self.state = ServerCoreState::Standby;
        self.cm.start().await;
    }

    pub async fn join_network(&mut self) {
        if let Some(core_node_addr) = self.core_node_addr {
            self.state = ServerCoreState::ConnectedToNetwork;
            self.cm
                .join_network(core_node_addr)
                .await
                .with_context(|| "Failed to join network")
                .unwrap();
        } else {
            info!("This server is running as Genesis Core Node...");
        }
    }

    pub async fn shutdown(&mut self) {
        self.state = ServerCoreState::ShuttingDown;
        info!("Shutdown ServerCore ...");
        self.cm.connection_close(self.core_node_addr.as_ref()).await;
    }

    pub fn get_my_current_state(&self) -> &ServerCoreState {
        &self.state
    }
}
