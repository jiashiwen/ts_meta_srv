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
        let etcd_clinet = GLOBAL_ETCD.get_mut().unwrap();

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
            println!("{:?}", nodeinfo);
            vec_nodes.push(nodeinfo);
        }
    }

    // let result = put(payload);
    // match result {
    //     Ok(str) => Ok(Json(Response::ok(str))),
    //     Err(e) => {
    //         let err = AppError {
    //             message: Some(e.to_string()),
    //             cause: None,
    //             error_type: AppErrorType::UnknowErr,
    //         };
    //         return Err(err);
    //     }
    // }
    Ok(Json(Response::ok(vec_nodes)))
}
