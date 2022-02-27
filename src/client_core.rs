use crate::connection_manager_edge::ConnectionManagerEdge;
use crate::message::ApplicationPayload;
use log::info;
use std::net::SocketAddr;

pub enum ClientCoreState {
    Init,
    Active,
    ShuttingDown,
}

pub struct ClientCore {
    state: ClientCoreState,
    cm: ConnectionManagerEdge,
}

impl ClientCore {
    pub fn new(my_addr: SocketAddr, core_node_addr: SocketAddr) -> ClientCore {
        info!("Initializing ClientCore");
        ClientCore {
            state: ClientCoreState::Init,
            cm: ConnectionManagerEdge::new(my_addr, core_node_addr),
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

    pub async fn send_message_to_my_core_node(&self) {
        let data = "hello".as_bytes().to_owned();
        let payload = ApplicationPayload::Enhanced { data };
        self.cm.send_message_to_my_core_node(payload).await;
    }
}
