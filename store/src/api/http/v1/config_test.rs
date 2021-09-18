/*
 * Copyright 2021 Datafuse Labs
 *
 * Licensed under the Apache License, Version 2.0 (the "License");
 * you may not use this file except in compliance with the License.
 * You may obtain a copy of the License at
 *
 * http://www.apache.org/licenses/LICENSE-2.0
 *
 * Unless required by applicable law or agreed to in writing, software
 * distributed under the License is distributed on an "AS IS" BASIS,
 * WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 * See the License for the specific language governing permissions and
 * limitations under the License.
 *
 */
use common_runtime::tokio;

#[tokio::test]
async fn test_config() -> common_exception::Result<()> {
    use axum::body::Body;
    use axum::handler::get;
    use axum::http::Request;
    use axum::http::StatusCode;
    use axum::http::{self};
    use axum::AddExtensionLayer;
    use axum::Router;
    use pretty_assertions::assert_eq;
    use tower::ServiceExt;

    use crate::api::http::v1::config::config_handler;
    use crate::configs::Config; // for `app.oneshot()`

    let conf = Config::empty();
    let cluster_router = Router::new()
        .route("/v1/config", get(config_handler))
        .layer(AddExtensionLayer::new(conf.clone()));

    let response = cluster_router
        .clone()
        .oneshot(
            Request::builder()
                .uri("/v1/config")
                .method(http::Method::GET)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    Ok(())
}
