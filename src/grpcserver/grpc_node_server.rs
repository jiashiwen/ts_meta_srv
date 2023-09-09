pub mod ts_meta {
    tonic::include_proto!("meta");
    pub const SERVICE_DESCRIPTOR_SET: &[u8] =
        tonic::include_file_descriptor_set!("service_descriptor");
}

use anyhow::anyhow;
use anyhow::Result;
use etcd_client::GetOptions;
use futures::FutureExt;
use std::net::SocketAddr;
use tokio::{
    net::TcpListener,
    sync::{
        oneshot::{self, Receiver, Sender},
        Mutex,
    },
    task::{self, JoinHandle},
};
use tokio_stream::wrappers::TcpListenerStream;
use ts_meta::{TestRequest, TestResponse};

use crate::{
    commons::{json_to_struct, struct_to_json_string},
    resources::GLOBAL_ETCD,
};

use self::ts_meta::{
    AllBucketsResponse, AllNodesResponse, BucketInfo, CreateBucketResponse, EmptyRequest,
};

pub const NODE_ID_PREFIX: &'static str = "nodeid_";
pub const BUCKET_ID_PREFIX: &'static str = "bucketid_";

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
            .register_encoded_file_descriptor_set(ts_meta::SERVICE_DESCRIPTOR_SET)
            .build()
            .unwrap();

        let mut builder = tonic::transport::Server::builder();

        let builder = builder
            .add_service(reflection_service)
            .add_service(ts_meta::node_server::NodeServer::new(NodeInfoServer {}))
            .add_service(ts_meta::meta_service_server::MetaServiceServer::new(
                MetaServciceServer {},
            ));

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
impl ts_meta::node_server::Node for NodeInfoServer {
    // async fn all_nodes_info(
    //     &self,
    //     _request: tonic::Request<AllNodesRequest>,
    // ) -> std::result::Result<tonic::Response<AllNodesInfoResponse>, tonic::Status> {
    //     let resp = AllNodesInfoResponse {
    //         all_nodes: vec![NodeInfomation {
    //             id: 12,
    //             rpc_addr: "127.0.0.1:8082".to_string(),
    //             http_addr: "127.0.0.1:8080".to_string(),
    //             status: 1,
    //         }],
    //     };
    //     Ok(tonic::Response::new(resp))
    // }

    async fn test(
        &self,
        request: tonic::Request<TestRequest>,
    ) -> std::result::Result<tonic::Response<TestResponse>, tonic::Status> {
        let req = request.into_inner();

        let resp = TestResponse {
            resp: req.echo + "_resp",
        };
        Ok(tonic::Response::new(resp))
    }
}

#[derive(Debug)]
pub struct MetaServciceServer {}

#[tonic::async_trait]
impl ts_meta::meta_service_server::MetaService for MetaServciceServer {
    async fn create_bucket_if_not_exists(
        &self,
        request: tonic::Request<BucketInfo>,
    ) -> std::result::Result<tonic::Response<CreateBucketResponse>, tonic::Status> {
        let bucketinfo: BucketInfo = request.into_inner();
        let mut key = BUCKET_ID_PREFIX.to_string();
        if bucketinfo.id.is_empty() {
            return Err(tonic::Status::data_loss("id must be set"));
        }
        key.push_str(bucketinfo.id.as_str());
        unsafe {
            match GLOBAL_ETCD.get_mut() {
                Some(etcd_client) => {
                    let value = struct_to_json_string(&bucketinfo).unwrap();
                    etcd_client.put(key, value, None).await;
                }
                None => {}
            }
        }

        let resp = CreateBucketResponse {
            created: true,
            bucket_info: Some(bucketinfo),
        };
        Ok(tonic::Response::new(resp))
    }
    async fn get_all_buckets(
        &self,
        _request: tonic::Request<EmptyRequest>,
    ) -> std::result::Result<tonic::Response<AllBucketsResponse>, tonic::Status> {
        let mut bucketsinfo = Vec::new();
        unsafe {
            match GLOBAL_ETCD.get_mut() {
                Some(etcd_client) => {
                    match etcd_client
                        .get(BUCKET_ID_PREFIX, Some(GetOptions::new().with_prefix()))
                        .await
                    {
                        Ok(resp) => {
                            for kv in resp.kvs() {
                                let vec_to_string = String::from_utf8(kv.value().to_vec()).unwrap();
                                let info =
                                    json_to_struct::<ts_meta::BucketInfo>(&vec_to_string).unwrap();
                                bucketsinfo.push(info);
                            }
                        }
                        Err(_) => {}
                    };
                }
                None => {}
            }
        }
        let resp = AllBucketsResponse { bucketsinfo };
        Ok(tonic::Response::new(resp))
    }
    async fn get_all_nodes(
        &self,
        _request: tonic::Request<EmptyRequest>,
    ) -> std::result::Result<tonic::Response<AllNodesResponse>, tonic::Status> {
        let mut nodesinfo: Vec<ts_meta::NodeInfo> = Vec::new();
        unsafe {
            match GLOBAL_ETCD.get_mut() {
                Some(etcd_client) => {
                    match etcd_client
                        .get(NODE_ID_PREFIX, Some(GetOptions::new().with_prefix()))
                        .await
                    {
                        Ok(resp) => {
                            for kv in resp.kvs() {
                                let vec_to_string = String::from_utf8(kv.value().to_vec()).unwrap();
                                let nodeinfo =
                                    json_to_struct::<ts_meta::NodeInfo>(&vec_to_string).unwrap();
                                nodesinfo.push(nodeinfo);
                            }
                        }
                        Err(_) => {}
                    };
                }
                None => {}
            }
        }
        let resp = AllNodesResponse { nodesinfo };
        Ok(tonic::Response::new(resp))
    }
}

async fn all_nodes_info() -> Result<Vec<ts_meta::NodeInfo>> {
    let mut nodesinfo: Vec<ts_meta::NodeInfo> = Vec::new();
    unsafe {
        match GLOBAL_ETCD.get_mut() {
            Some(etcd_client) => {
                match etcd_client
                    .get(NODE_ID_PREFIX, Some(GetOptions::new().with_prefix()))
                    .await
                {
                    Ok(resp) => {
                        for kv in resp.kvs() {
                            match String::from_utf8(kv.value().to_vec()) {
                                Ok(vec_to_string) => {
                                    if let Ok(nodeinfo) =
                                        json_to_struct::<ts_meta::NodeInfo>(&vec_to_string)
                                    {
                                        nodesinfo.push(nodeinfo);
                                    };
                                }
                                Err(_) => continue,
                            };
                        }
                    }
                    Err(e) => {
                        return Err(anyhow!(e.to_string()));
                    }
                };
            }
            None => {
                return Err(anyhow!("can not get etcd client"));
            }
        }
    }
    return Ok(nodesinfo);
}
