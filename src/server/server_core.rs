use anyhow::Context;
use log::{debug, info, warn};
use simple_bitcoin::blockchain::manager::BlockchainManager;
use simple_bitcoin::blockchain::transaction_pool::TransactionPool;
use simple_bitcoin::connection_manager_core::{ApplicationPayloadHandler, ConnectionManagerCore};
use simple_bitcoin::message::ApplicationPayload;
use std::net::SocketAddr;
use std::sync::{Arc, Mutex};
use std::time::Duration;

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
    bm: Arc<Mutex<BlockchainManager>>,
    tp: Arc<Mutex<TransactionPool>>,
}

fn generate_application_payload_handler(
    transaction_pool: Arc<Mutex<TransactionPool>>,
    blockchain_manager: Arc<Mutex<BlockchainManager>>,
) -> impl ApplicationPayloadHandler {
    // An implementation of ApplicationPayloadHandler
    move |payload: ApplicationPayload,
          peer: SocketAddr,
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
            ApplicationPayload::NewBlock { block } => {
                let mut blockchain_manager = blockchain_manager.lock().unwrap();
                let mut transaction_pool = transaction_pool.lock().unwrap();

                if let Err(err) = blockchain_manager.is_valid_block(&block) {
                    warn!("Invalid block: {}", err);

                    let payload = ApplicationPayload::RequestFullChain;
                    return Some((payload, vec![peer]));
                }

                blockchain_manager.add_new_block(block);
                debug!(
                    "Current blockchain is: {:?}",
                    blockchain_manager.get_chain()
                );

                blockchain_manager.remove_useless_transactions(&mut transaction_pool);

                None
            }
            ApplicationPayload::RequestFullChain => {
                debug!("Send our latest blockchain for reply to {}", peer);
                let chain = blockchain_manager.lock().unwrap().get_chain();
                let payload = ApplicationPayload::FullChain { chain };
                Some((payload, vec![peer]))
            }
            ApplicationPayload::FullChain { chain } => {
                if !is_core {
                    warn!("Blockchain received from unknown");
                    return None;
                }

                let mut blockchain_manager = blockchain_manager.lock().unwrap();
                let mut transaction_pool = transaction_pool.lock().unwrap();

                let orphan_transactions = blockchain_manager.resolve_conflicts(chain);
                for transaction in orphan_transactions {
                    transaction_pool.add_new_transaction(transaction);
                }

                None
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
                generate_application_payload_handler(Arc::clone(&pool), Arc::clone(&manager)),
            ),
            bm: manager,
            tp: pool,
        }
    }

    pub async fn start(&mut self) {
        self.state = ServerCoreState::Standby;
        self.cm.start().await;

        tokio::spawn(TransactionPool::generate_block_periodically(
            Arc::clone(&self.tp),
            Arc::clone(&self.bm),
            Arc::clone(&self.cm.inner),
            Duration::from_secs(10),
        ));
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
