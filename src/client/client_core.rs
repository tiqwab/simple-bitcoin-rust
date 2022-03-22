use log::{debug, info};
use simple_bitcoin::blockchain::transaction::Transaction;
use simple_bitcoin::blockchain::utxo::UTXOManager;
use simple_bitcoin::connection_manager_edge::{ApplicationPayloadHandler, ConnectionManagerEdge};
use simple_bitcoin::message::ApplicationPayload;
use std::net::SocketAddr;
use std::sync::{Arc, Mutex};

#[derive(Debug)]
pub enum ClientCoreState {
    Init,
    Active,
    ShuttingDown,
}

pub struct ClientCore {
    state: ClientCoreState,
    cm: ConnectionManagerEdge,
}

fn generate_application_payload_handler(
    utxo_manager: Arc<Mutex<UTXOManager>>,
) -> impl ApplicationPayloadHandler {
    move |payload: ApplicationPayload| {
        debug!("handle_application_payload: {:?}", payload);

        match payload {
            ApplicationPayload::FullChain { chain } => {
                let txs: Vec<Transaction> = chain
                    .into_iter()
                    .flat_map(|block| block.get_transactions())
                    .collect();
                utxo_manager.lock().unwrap().refresh_utxos(&txs);
            }
            _ => {}
        }
    }
}

impl ClientCore {
    pub fn new(
        my_addr: SocketAddr,
        core_node_addr: SocketAddr,
        utxo_manager: Arc<Mutex<UTXOManager>>,
    ) -> ClientCore {
        info!("Initializing ClientCore");
        ClientCore {
            state: ClientCoreState::Init,
            cm: ConnectionManagerEdge::new(
                my_addr,
                core_node_addr,
                generate_application_payload_handler(utxo_manager),
            ),
        }
    }

    pub async fn start(&mut self) {
        self.state = ClientCoreState::Active;
        self.cm.start().await;
        self.cm.join_network().await;
    }

    pub async fn shutdown(&mut self) {
        self.state = ClientCoreState::ShuttingDown;
        info!("Shutdown ClientCore ...");
        self.cm.connection_close().await;
    }

    pub fn get_my_current_state(&self) -> &ClientCoreState {
        &self.state
    }

    pub async fn send_msg_to_core(&self, payload: ApplicationPayload) {
        self.cm.send_message_to_my_core_node(payload).await;
    }
}
