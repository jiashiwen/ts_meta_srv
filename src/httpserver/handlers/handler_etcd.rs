use std::fs;

use anyhow::Context;
use axum::Json;
use etcd_client::GetOptions;

use crate::{
    commons::json_to_struct,
    httpserver::{
        exception::{AppError, AppErrorType},
        module::{NodeInfo, Response, NODE_ID_PREFIX},
    },
    resources::GLOBAL_ETCD,
};

use super::HandlerResult;

pub async fn get_nodes_info() -> HandlerResult<Vec<NodeInfo>> {
    let mut vec_nodes: Vec<NodeInfo> = vec![];
    unsafe {
        // let etcd_clinet = GLOBAL_ETCD.get_mut().unwrap();

        match GLOBAL_ETCD.get_mut() {
            Some(etcd_clinet) => {
                let resp = etcd_clinet
                    .get(NODE_ID_PREFIX, Some(GetOptions::new().with_prefix()))
                    .await
                    .map_err(|e| {
                        let err = AppError {
                            message: Some(e.to_string()),
                            cause: None,
                            error_type: AppErrorType::UnknowErr,
                        };
                        return err;
                    })?;

                for kv in resp.kvs() {
                    let vec_to_string = String::from_utf8(kv.value().to_vec()).unwrap();
                    let nodeinfo = json_to_struct::<NodeInfo>(&vec_to_string).unwrap();
                    vec_nodes.push(nodeinfo);
                }
            }
            None => {
                let err = AppError {
                    message: Some("globale etcd client error".to_string()),
                    cause: None,
                    error_type: AppErrorType::UnknowErr,
                };
                return Err(err);
            }
        }
    }

    Ok(Json(Response::ok(vec_nodes)))
}
