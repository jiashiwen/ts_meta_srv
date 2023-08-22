pub mod pb {
    tonic::include_proto!("meta_service");
    pub const NODE_DESCRIPTOR_SET: &[u8] = tonic::include_file_descriptor_set!("node_descriptor");
}

use std::net::SocketAddr;

use anyhow::Result;

use futures::FutureExt;
use pb::{AllNodesRequest, AllNodesResponse};
use tokio::{
    net::TcpListener,
    sync::{
        oneshot::{self, Receiver, Sender},
        Mutex,
    },
    task::{self, JoinHandle},
};
use tokio_stream::wrappers::TcpListenerStream;

use self::pb::NodeInfo;

#[derive(Debug)]
pub struct GrpcNodeServer {
    pub shutdown_tx: Mutex<Option<Sender<()>>>,
    pub serve_state: Mutex<Option<Receiver<Result<()>>>>,
}

impl GrpcNodeServer {
    pub async fn shutdown(&self) -> Result<()> {
        Ok(())
    }

    pub async fn start(
        &self,
        addr: SocketAddr,
    ) -> Result<JoinHandle<Result<(), Result<(), anyhow::Error>>>> {
        let (tx, rx) = oneshot::channel();
        let (listener, addr) = {
            let mut shutdown_tx = self.shutdown_tx.lock().await;

            let listener = TcpListener::bind(addr).await.unwrap();
            let addr = listener.local_addr().unwrap();
            println!("gRPC server is bound to {}", addr);

            *shutdown_tx = Some(tx);

            (listener, addr)
        };

        let reflection_service = tonic_reflection::server::Builder::configure()
            .register_encoded_file_descriptor_set(pb::NODE_DESCRIPTOR_SET)
            .build()
            .unwrap();

        let mut builder = tonic::transport::Server::builder();

        let builder = builder
            .add_service(reflection_service)
            .add_service(pb::node_server::NodeServer::new(NodeInfoServer {}));

        let (serve_state_tx, serve_state_rx) = oneshot::channel();
        let mut serve_state = self.serve_state.lock().await;
        *serve_state = Some(serve_state_rx);

        let handle = task::spawn(async move {
            let result = builder
                .serve_with_incoming_shutdown(TcpListenerStream::new(listener), rx.map(drop))
                .await
                .map_err(|e| anyhow::Error::from(e));
            println!("{:?}", result);

            serve_state_tx.send(result)
        });

        Ok(handle)
    }
}

#[derive(Debug)]
pub struct NodeInfoServer {}

#[tonic::async_trait]
impl pb::node_server::Node for NodeInfoServer {
    async fn all_nodes_info(
        &self,
        request: tonic::Request<AllNodesRequest>,
    ) -> std::result::Result<tonic::Response<AllNodesResponse>, tonic::Status> {
        let resp = AllNodesResponse {
            all_nodes: vec![NodeInfo {
                id: 12,
                rpc_addr: "127.0.0.1:8082".to_string(),
                http_addr: "127.0.0.1:8080".to_string(),
                status: 1,
            }],
        };

        Ok(tonic::Response::new(resp))
    }
}
