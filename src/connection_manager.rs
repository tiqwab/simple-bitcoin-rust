use crate::message::{Message, Payload};
use anyhow::Result;
use log::{debug, error, info};
use std::collections::HashSet;
use std::net::SocketAddr;
use std::sync::{Arc, Mutex};
use std::time::Duration;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};
use tokio::task::JoinHandle;

pub struct ConnectionManagerInner {
    addr: SocketAddr,
    core_node_set: HashSet<SocketAddr>,
}

impl ConnectionManagerInner {
    pub fn new(addr: SocketAddr) -> ConnectionManagerInner {
        let node_set = HashSet::<SocketAddr>::new();
        let mut manager = ConnectionManagerInner {
            addr: addr.clone(),
            core_node_set: node_set,
        };
        manager.add_peer(addr);
        manager
    }

    // 新たに接続された Core ノードをリストに追加する
    fn add_peer(&mut self, peer: SocketAddr) -> bool {
        debug!("Adding peer: {}", peer);
        self.core_node_set.insert(peer)
    }

    // 離脱した Core ノードをリストから削除する
    fn remove_peer(&mut self, peer: &SocketAddr) -> bool {
        debug!("Removing peer: {}", peer);
        let res = self.core_node_set.remove(peer);
        debug!("Current Core list: {:?}", self.core_node_set);
        res
    }

    pub fn get_core_nodes(&self) -> Vec<SocketAddr> {
        self.core_node_set.iter().cloned().collect()
    }
}

pub struct ConnectionManager {
    addr: SocketAddr,
    inner: Arc<Mutex<ConnectionManagerInner>>,
    check_peers_interval: Duration,
    join_handle_for_listen: Option<JoinHandle<Result<()>>>,
    join_handle_for_check_peers: Option<JoinHandle<()>>,
}

impl ConnectionManager {
    pub fn new(addr: SocketAddr) -> ConnectionManager {
        info!("Initializing ConnectionManager...");
        ConnectionManager {
            addr,
            inner: Arc::new(Mutex::new(ConnectionManagerInner::new(addr))),
            check_peers_interval: Duration::from_secs(30),
            join_handle_for_listen: None,
            join_handle_for_check_peers: None,
        }
    }

    // 待受を開始する前に呼び出される (Server Core 向け)
    pub async fn start(&mut self) {
        // do check_peers_connection repeatedly
        {
            let manager = Arc::clone(&self.inner);
            let handle = tokio::spawn(Self::check_peers_connection(
                manager,
                self.check_peers_interval,
            ));
            self.join_handle_for_check_peers = Some(handle);
        }

        let handle = tokio::spawn(Self::wait_for_access(Arc::clone(&self.inner), self.addr));
        self.join_handle_for_listen = Some(handle);
    }

    // 終了前の処理としてソケットを閉じる (ServerCore 向け)
    pub async fn connection_close(&mut self) {
        if let Some(handle) = self.join_handle_for_check_peers.as_mut() {
            handle.abort();
            handle.await.unwrap_or_else(|err| {
                if !err.is_cancelled() {
                    error!("Error occurred when closing connection: {:?}", err);
                }
            });
        }
        if let Some(handle) = self.join_handle_for_listen.as_mut() {
            handle.abort();
            handle
                .await
                .unwrap_or_else(|err| {
                    if !err.is_cancelled() {
                        error!("Error occurred when closing connection: {:?}", err);
                    }
                    Ok(())
                })
                .unwrap_or_else(|err| {
                    error!("Error occurred when waiting_for_access: {:?}", err);
                });
        }
    }

    // ユーザが指定した既知の Core ノードへの接続 (Server Core 向け)
    pub async fn join_network(&self, target_addr: SocketAddr) -> Result<()> {
        info!("Send request to join network to: {}", target_addr);
        let mut stream = TcpStream::connect(target_addr).await?;
        let payload = Payload::Add { field1: 0 };
        let msg = Message::new(self.addr.port(), payload);
        stream
            .write_all(serde_json::to_string(&msg)?.as_bytes())
            .await?;
        Ok(())
    }

    async fn wait_for_access(
        manager: Arc<Mutex<ConnectionManagerInner>>,
        manager_addr: SocketAddr,
    ) -> Result<()> {
        let sock = TcpListener::bind(manager_addr).await?;

        loop {
            let (stream, src_addr) = sock.accept().await?;
            let manager = Arc::clone(&manager);
            tokio::spawn(Self::handle_message(
                manager,
                manager_addr,
                stream,
                src_addr,
            ));
        }
    }

    // 受信したメッセージを確認して、内容に応じた処理を行う
    async fn handle_message(
        manager: Arc<Mutex<ConnectionManagerInner>>,
        manager_addr: SocketAddr,
        mut stream: TcpStream,
        src_addr: SocketAddr, // address the packet comes from
    ) -> Result<()> {
        let mut buf = vec![];
        stream.read_to_end(&mut buf).await?;
        let message: Message = serde_json::from_slice(&buf[..])?;
        debug!("Received Message from {}: {:?}", src_addr, message);

        // address the peer core node listens to.
        let peer_addr = SocketAddr::new(src_addr.ip(), message.port);
        let manager_port = manager_addr.port();

        match message.payload {
            Payload::Add { .. } => {
                let added = manager.lock().unwrap().add_peer(peer_addr);
                let nodes = manager.lock().unwrap().get_core_nodes();
                if added {
                    let payload = Payload::CoreList {
                        nodes: nodes.clone(),
                    };
                    Self::send_msg_to_all_peer(manager, nodes, Message::new(manager_port, payload))
                        .await;
                }
            }
            Payload::Remove => {
                let removed = manager.lock().unwrap().remove_peer(&peer_addr);
                let nodes = manager.lock().unwrap().get_core_nodes();
                if removed {
                    let payload = Payload::CoreList {
                        nodes: nodes.clone(),
                    };
                    Self::send_msg_to_all_peer(manager, nodes, Message::new(manager_port, payload))
                        .await;
                }
            }
            Payload::CoreList { nodes } => {
                let mut manager = manager.lock().unwrap();
                for node in nodes.into_iter() {
                    if !manager.core_node_set.contains(&node) {
                        manager.core_node_set.insert(node);
                    }
                }
            }
            Payload::RequestCoreList => {
                let nodes = manager.lock().unwrap().get_core_nodes();
                let payload = Payload::CoreList { nodes };
                Self::send_msg(manager, &peer_addr, Message::new(manager_port, payload)).await;
            }
            Payload::Ping => {}
            Payload::AddAsEdge => {}
            Payload::RemoveEdge => {}
        }

        Ok(())
    }

    // addr に msg を送信する。
    // addr への送信でエラーが発生した場合、その addr を core_node_set から除去する。
    // 送信に成功した場合 true を返す。
    pub async fn send_msg(
        manager: Arc<Mutex<ConnectionManagerInner>>,
        addr: &SocketAddr,
        msg: Message,
    ) -> bool {
        debug!("Send message to {}: {:?}", addr, msg);
        if let Err(e) = Self::do_send_msg(addr, msg).await {
            error!("Error occurred in send_msg: {:?}", e);
            let mut manager = manager.lock().unwrap();
            manager.remove_peer(addr);
            return false;
        }
        return true;
    }

    // 指定されたノードに対してメッセージを送信する
    async fn do_send_msg(addr: &SocketAddr, msg: Message) -> Result<()> {
        let mut sock = TcpStream::connect(addr).await?;
        let content = serde_json::to_string(&msg)?;
        sock.write_all(content.as_bytes()).await?;
        Ok(())
    }

    // Core ノードリストに登録されているすべてのノードに対して同じメッセージを broadcast する
    pub async fn send_msg_to_all_peer(
        manager: Arc<Mutex<ConnectionManagerInner>>,
        addrs: Vec<SocketAddr>,
        msg: Message,
    ) {
        for addr in addrs.iter() {
            Self::send_msg(Arc::clone(&manager), addr, msg.clone()).await;
        }
    }

    // 接続されている Core ノードすべての接続状況確認を行う
    // interval 毎に実行される
    #[async_recursion::async_recursion]
    async fn check_peers_connection(
        manager: Arc<Mutex<ConnectionManagerInner>>,
        interval: Duration,
    ) {
        debug!("check_peers_connection was called");

        // check peers
        let manager_addr = manager.lock().unwrap().addr.clone();
        let target_nodes = manager.lock().unwrap().get_core_nodes();
        let mut failed_nodes = vec![];
        for node in target_nodes.iter() {
            let payload = Payload::Ping;
            let msg = Message::new(manager_addr.port(), payload);
            if !Self::send_msg(Arc::clone(&manager), node, msg).await {
                failed_nodes.push(node.clone());
            }
        }

        // remove disconnected nodes
        {
            let mut manager = manager.lock().unwrap();
            for node in failed_nodes.iter() {
                manager.remove_peer(node);
            }
        }

        // send core node list if necessary
        if !failed_nodes.is_empty() {
            let nodes = manager.lock().unwrap().get_core_nodes();
            let payload = Payload::CoreList {
                nodes: nodes.clone(),
            };
            let msg = Message::new(manager_addr.port(), payload);
            Self::send_msg_to_all_peer(Arc::clone(&manager), nodes, msg).await;
        }

        tokio::time::sleep(interval).await;
        Self::check_peers_connection(manager, interval).await;
    }
}
