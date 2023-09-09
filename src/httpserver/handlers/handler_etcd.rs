use crate::{
    grpcserver::ts_meta::NodeInfo,
    httpserver::{
        exception::{AppError, AppErrorType},
        module::Response,
    },
    service::nodes_info,
};
use axum::Json;

use super::HandlerResult;

pub async fn get_nodes_info() -> HandlerResult<Vec<NodeInfo>> {
    let vec_nodes_info = nodes_info().await.map_err(|e| {
        let err = AppError {
            message: Some(e.to_string()),
            cause: None,
            error_type: AppErrorType::UnknowErr,
        };
        return err;
    })?;

    Ok(Json(Response::ok(vec_nodes_info)))
}
