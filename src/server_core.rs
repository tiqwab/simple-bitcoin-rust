use crate::blockchain::manager::BlockchainManager;
use crate::blockchain::transaction_pool::TransactionPool;
use crate::connection_manager_core::{ApplicationPayloadHandler, ConnectionManagerCore};
use crate::message::ApplicationPayload;
use anyhow::Context;
use log::{debug, info};
use std::net::SocketAddr;
use std::sync::{Arc, Mutex};

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

fn generate_application_payload_handler(
    transaction_pool: Arc<Mutex<TransactionPool>>,
    _blockchain_manager: Arc<Mutex<BlockchainManager>>,
) -> impl ApplicationPayloadHandler {
    // An implementation of ApplicationPayloadHandler
    move |payload: ApplicationPayload,
          _peer: Option<SocketAddr>,
          core_nodes: Vec<SocketAddr>,
          is_core: bool| {
        debug!("handle_application_payload: {:?}", payload);
        match payload {
            ApplicationPayload::NewTransaction { transaction } => {
                transaction_pool
                    .lock()
                    .unwrap()
                    .add_new_transaction(transaction.clone());
                if !is_core {
                    let new_payload = ApplicationPayload::NewTransaction { transaction };
                    Some((new_payload, core_nodes))
                } else {
                    None
                }
            }
            ApplicationPayload::NewBlock => {
                unimplemented!()
            }
            ApplicationPayload::RequestFullChain => {
                unimplemented!()
            }
            ApplicationPayload::FullChain => {
                unimplemented!()
            }
            ApplicationPayload::Enhanced { .. } => {
                unimplemented!()
            }
        }
    }
}

impl ServerCore {
    pub fn new(
        my_addr: SocketAddr,
        core_node_addr: Option<SocketAddr>,
        pool: Arc<Mutex<TransactionPool>>,
        manager: Arc<Mutex<BlockchainManager>>,
    ) -> ServerCore {
        info!("Initializing ServerCore...");
        ServerCore {
            state: ServerCoreState::Init,
            core_node_addr,
            cm: ConnectionManagerCore::new(
                my_addr,
                generate_application_payload_handler(pool, manager),
            ),
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
