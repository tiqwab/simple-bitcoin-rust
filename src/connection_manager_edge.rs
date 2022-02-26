use crate::message::{Message, Payload};
use anyhow::Result;
use log::{debug, error, info, warn};
use std::collections::HashSet;
use std::net::SocketAddr;
use std::sync::{Arc, Mutex};
use std::time::Duration;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};
use tokio::task::JoinHandle;

pub struct ConnectionManagerInner {
    my_addr: SocketAddr,
    current_core_node: SocketAddr,
    core_node_set: HashSet<SocketAddr>,
}

impl ConnectionManagerInner {
    pub fn new(my_addr: SocketAddr, core_node_addr: SocketAddr) -> ConnectionManagerInner {
        let node_set = HashSet::<SocketAddr>::new();
        let mut manager = ConnectionManagerInner {
            my_addr,
            current_core_node: core_node_addr,
            core_node_set: node_set,
        };
        manager.add_peer(core_node_addr);
        manager
    }

    // 新たに接続された Core ノードをリストに追加する
    fn add_peer(&mut self, peer: SocketAddr) -> bool {
        debug!("Adding peer: {}", peer);
        self.core_node_set.insert(peer)
    }

    // 指定した Core ノードをリストから削除する
    // もしそれが current_core_node だった場合は別の core ノードに設定し直す
    fn remove_peer(&mut self, peer: &SocketAddr) -> bool {
        debug!("Removing peer: {}", peer);
        let res = self.core_node_set.remove(peer);
        debug!("Current Core list: {:?}", self.core_node_set);
        if &self.current_core_node == peer {
            // TODO: avoid using unwrap
            let new_core_node = *self.core_node_set.iter().nth(0).unwrap();
            debug!(
                "Replace current_core_node({}) with {}",
                self.current_core_node, new_core_node
            );
            self.current_core_node = new_core_node;
        }
        res
    }

    pub fn get_my_addr(&self) -> SocketAddr {
        self.my_addr
    }

    pub fn get_current_code_node(&self) -> SocketAddr {
        self.current_core_node
    }
}

pub struct ConnectionManagerEdge {
    my_addr: SocketAddr,
    inner: Arc<Mutex<ConnectionManagerInner>>,
    check_peers_interval: Duration,
    join_handle_for_listen: Option<JoinHandle<Result<()>>>,
    join_handle_for_check_peer: Option<JoinHandle<()>>,
}

impl ConnectionManagerEdge {
    pub fn new(my_addr: SocketAddr, core_node_addr: SocketAddr) -> ConnectionManagerEdge {
        info!("Initializing ConnectionManagerEdge...");
        ConnectionManagerEdge {
            my_addr,
            inner: Arc::new(Mutex::new(ConnectionManagerInner::new(
                my_addr,
                core_node_addr,
            ))),
            check_peers_interval: Duration::from_secs(30),
            join_handle_for_listen: None,
            join_handle_for_check_peer: None,
        }
    }

    // 待受を開始する前に呼び出される
    pub async fn start(&mut self) {
        // do check_peers_connection repeatedly
        {
            let manager = Arc::clone(&self.inner);
            let handle = tokio::spawn(Self::check_peer_connection(
                manager,
                self.check_peers_interval,
            ));
            self.join_handle_for_check_peer = Some(handle);
        }

        let handle = tokio::spawn(Self::wait_for_access(Arc::clone(&self.inner), self.my_addr));
        self.join_handle_for_listen = Some(handle);
    }

    // 終了前の処理としてソケットを閉じる
    pub async fn connection_close(&mut self) {
        if let Some(handle) = self.join_handle_for_check_peer.as_mut() {
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

        let core_node_addr = self.inner.lock().unwrap().current_core_node;
        let manager_port = self.my_addr.port();
        Self::send_msg(
            Arc::clone(&self.inner),
            &core_node_addr,
            Message::new(manager_port, Payload::RemoveEdge {}),
        )
        .await;
    }

    // ユーザが指定した既知の Core ノードへの接続
    pub async fn join_network(&self) {
        Self::connect_to_core(Arc::clone(&self.inner)).await;
    }

    async fn connect_to_core(manager: Arc<Mutex<ConnectionManagerInner>>) {
        let core_node_addr = manager.lock().unwrap().current_core_node;
        info!("Connecting to Core node: {}", core_node_addr);
        let my_addr = manager.lock().unwrap().my_addr;
        let payload = Payload::AddAsEdge {};
        let msg = Message::new(my_addr.port(), payload);
        Self::send_msg(manager, &core_node_addr, msg).await;
    }

    async fn wait_for_access(
        manager: Arc<Mutex<ConnectionManagerInner>>,
        manager_addr: SocketAddr,
    ) -> Result<()> {
        let sock = TcpListener::bind(manager_addr).await?;

        loop {
            let (stream, src_addr) = sock.accept().await?;
            let manager = Arc::clone(&manager);
            tokio::spawn(Self::handle_message(manager, stream, src_addr));
        }
    }

    // 受信したメッセージを確認して、内容に応じた処理を行う
    async fn handle_message(
        manager: Arc<Mutex<ConnectionManagerInner>>,
        mut stream: TcpStream,
        src_addr: SocketAddr, // address the packet comes from
    ) -> Result<()> {
        let mut buf = vec![];
        stream.read_to_end(&mut buf).await?;
        let message: Message = serde_json::from_slice(&buf[..])?;
        debug!("Received Message from {}: {:?}", src_addr, message);

        match message.payload {
            Payload::CoreList { nodes } => {
                let mut manager = manager.lock().unwrap();
                for node in nodes.into_iter() {
                    if !manager.core_node_set.contains(&node) {
                        manager.core_node_set.insert(node);
                    }
                }
            }
            _ => {
                warn!(
                    "Unexpected message from {}. Ignore it: {:?}",
                    src_addr, message
                );
            }
        }

        Ok(())
    }

    // addr に msg を送信する。
    // addr への送信でエラーが発生した場合、その addr を core_node_set から除去する。
    // 送信に成功した場合 true を返す。
    async fn send_msg(
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

    // 接続されている Core ノードすべての接続状況確認を行う
    // interval 毎に実行される
    #[async_recursion::async_recursion]
    async fn check_peer_connection(
        manager: Arc<Mutex<ConnectionManagerInner>>,
        interval: Duration,
    ) {
        debug!("check_peer_connection was called");
        let manager_addr = manager.lock().unwrap().get_my_addr();
        let core_node_addr = manager.lock().unwrap().get_current_code_node();
        let payload = Payload::Ping;
        let msg = Message::new(manager_addr.port(), payload);

        if !Self::send_msg(Arc::clone(&manager), &core_node_addr, msg).await {
            // remove disconnected
            info!(
                "Couldn't connect to the current code node: {}",
                core_node_addr
            );
            manager.lock().unwrap().remove_peer(&core_node_addr);
            Self::connect_to_core(Arc::clone(&manager)).await;
        }

        tokio::time::sleep(interval).await;
        Self::check_peer_connection(manager, interval).await;
    }
}
