// Copyright 2018-2020 Cargill Incorporated
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

use actix_web::{client::Client, http::StatusCode, web, Error, HttpResponse};
use percent_encoding::utf8_percent_encode;
use splinter::node_registry::Node;
use splinter::protocol;
use std::collections::HashMap;

use super::{ErrorResponse, SuccessResponse, DEFAULT_LIMIT, DEFAULT_OFFSET, QUERY_ENCODE_SET};

pub async fn fetch_node(
    identity: web::Path<String>,
    client: web::Data<Client>,
    splinterd_url: web::Data<String>,
) -> Result<HttpResponse, Error> {
    let mut response = client
        .get(&format!(
            "{}/admin/nodes/{}",
            splinterd_url.get_ref(),
            identity
        ))
        .header(
            "SplinterProtocolVersion",
            protocol::ADMIN_PROTOCOL_VERSION.to_string(),
        )
        .send()
        .await?;

    let body = response.body().await?;

    match response.status() {
        StatusCode::OK => {
            let node: Node = serde_json::from_slice(&body)?;
            Ok(HttpResponse::Ok().json(SuccessResponse::new(node)))
        }
        StatusCode::NOT_FOUND => {
            let message: String = serde_json::from_slice(&body)?;
            Ok(HttpResponse::NotFound().json(ErrorResponse::not_found(&message)))
        }
        _ => {
            let message: String = serde_json::from_slice(&body)?;
            debug!(
                "Internal Server Error. Splinterd responded with error {} message {}",
                response.status(),
                message
            );
            Ok(HttpResponse::InternalServerError().json(ErrorResponse::internal_error()))
        }
    }
}

pub async fn list_nodes(
    client: web::Data<Client>,
    splinterd_url: web::Data<String>,
    query: web::Query<HashMap<String, String>>,
) -> Result<HttpResponse, Error> {
    let mut request_url = format!("{}/admin/nodes", splinterd_url.get_ref());

    let offset = query
        .get("offset")
        .map(ToOwned::to_owned)
        .unwrap_or_else(|| DEFAULT_OFFSET.to_string());
    let limit = query
        .get("limit")
        .map(ToOwned::to_owned)
        .unwrap_or_else(|| DEFAULT_LIMIT.to_string());

    request_url = format!("{}?offset={}&limit={}", request_url, offset, limit);

    if let Some(filter) = query.get("filter") {
        request_url = format!(
            "{}&filter={}",
            request_url,
            utf8_percent_encode(filter, QUERY_ENCODE_SET).to_string()
        );
    }

    let mut response = client
        .get(&request_url)
        .header(
            "SplinterProtocolVersion",
            protocol::ADMIN_PROTOCOL_VERSION.to_string(),
        )
        .send()
        .await?;

    let body = response.body().await?;

    match response.status() {
        StatusCode::OK => {
            let list_reponse: SuccessResponse<Vec<Node>> = serde_json::from_slice(&body)?;
            Ok(HttpResponse::Ok().json(list_reponse))
        }
        StatusCode::BAD_REQUEST => {
            let message: String = serde_json::from_slice(&body)?;
            Ok(HttpResponse::BadRequest().json(ErrorResponse::bad_request(&message)))
        }
        _ => {
            let message: String = serde_json::from_slice(&body)?;
            debug!(
                "Internal Server Error. Splinterd responded with error {} message {}",
                response.status(),
                message
            );
            Ok(HttpResponse::InternalServerError().json(ErrorResponse::internal_error()))
        }
    }
}

#[cfg(all(feature = "test-node-endpoint", test))]
mod test {
    use super::*;
    use crate::rest_api::routes::Paging;
    use actix_web::{
        http::{header, StatusCode},
        test, web, App,
    };
    use splinter::node_registry::NodeBuilder;

    static SPLINTERD_URL: &str = "http://splinterd-node:8085";

    #[actix_rt::test]
    /// Tests a GET /admin/nodes/{identity} request returns the expected node.
    async fn test_fetch_node_ok() {
        let mut app = test::init_service(
            App::new()
                .data(Client::new())
                .data(SPLINTERD_URL.to_string())
                .service(web::resource("/admin/nodes/{identity}").route(web::get().to(fetch_node))),
        )
        .await;

        let req = test::TestRequest::get()
            .uri(&format!("/admin/nodes/{}", get_node_1().identity))
            .to_request();

        let resp = test::call_service(&mut app, req).await;

        assert_eq!(resp.status(), StatusCode::OK);
        let response: SuccessResponse<Node> =
            serde_json::from_slice(&test::read_body(resp).await).unwrap();
        assert_eq!(response.data, get_node_1())
    }

    #[actix_rt::test]
    /// Tests a GET /admin/nodes/{identity} request returns NotFound when an invalid identity is passed
    async fn test_fetch_node_not_found() {
        let mut app = test::init_service(
            App::new()
                .data(Client::new())
                .data(SPLINTERD_URL.to_string())
                .service(web::resource("/admin/nodes/{identity}").route(web::get().to(fetch_node))),
        )
        .await;

        let req = test::TestRequest::get()
            .uri("/admin/nodes/Node-not-valid")
            .to_request();

        let resp = test::call_service(&mut app, req).await;

        assert_eq!(resp.status(), StatusCode::NOT_FOUND);
    }

    #[actix_rt::test]
    /// Tests a GET /admin/nodes request with no filters returns the expected nodes.
    async fn test_list_node_ok() {
        let mut app = test::init_service(
            App::new()
                .data(Client::new())
                .data(SPLINTERD_URL.to_string())
                .service(web::resource("/admin/nodes").route(web::get().to(list_nodes))),
        )
        .await;

        let req = test::TestRequest::get().uri("/admin/nodes").to_request();

        let resp = test::call_service(&mut app, req).await;

        assert_eq!(resp.status(), StatusCode::OK);
        let nodes: SuccessResponse<Vec<Node>> =
            serde_json::from_slice(&test::read_body(resp).await).unwrap();
        assert_eq!(nodes.data.len(), 2);
        assert!(nodes.data.contains(&get_node_1()));
        assert!(nodes.data.contains(&get_node_2()));
        assert_eq!(
            nodes.paging,
            Some(create_test_paging_response(
                0,
                100,
                0,
                0,
                0,
                2,
                "/admin/nodes?"
            ))
        )
    }

    #[actix_rt::test]
    /// Tests a GET /admin/nodes request with filters returns the expected node.
    async fn test_list_node_with_filters_ok() {
        let mut app = test::init_service(
            App::new()
                .data(Client::new())
                .data(SPLINTERD_URL.to_string())
                .service(web::resource("/admin/nodes").route(web::get().to(list_nodes))),
        )
        .await;

        let filter = utf8_percent_encode("{\"company\":[\"=\",\"Bitwise IO\"]}", QUERY_ENCODE_SET)
            .to_string();

        let req = test::TestRequest::get()
            .uri(&format!("/admin/nodes?filter={}", filter))
            .header(header::CONTENT_TYPE, "application/json")
            .to_request();

        let resp = test::call_service(&mut app, req).await;

        assert_eq!(resp.status(), StatusCode::OK);
        let nodes: SuccessResponse<Vec<Node>> =
            serde_json::from_slice(&test::read_body(resp).await).unwrap();
        assert_eq!(nodes.data, vec![get_node_1()]);
        let link = format!("/admin/nodes?filter={}&", filter);
        assert_eq!(
            nodes.paging,
            Some(create_test_paging_response(0, 100, 0, 0, 0, 1, &link))
        )
    }

    #[actix_rt::test]
    /// Tests a GET /admin/nodes request with invalid filter returns BadRequest response.
    async fn test_list_node_with_filters_bad_request() {
        let mut app = test::init_service(
            App::new()
                .data(Client::new())
                .data(SPLINTERD_URL.to_string())
                .service(web::resource("/admin/nodes").route(web::get().to(list_nodes))),
        )
        .await;

        let filter = utf8_percent_encode("{\"company\":[\"*\",\"Bitwise IO\"]}", QUERY_ENCODE_SET)
            .to_string();

        let req = test::TestRequest::get()
            .uri(&format!("/admin/nodes?filter={}", filter))
            .header(header::CONTENT_TYPE, "application/json")
            .to_request();

        let resp = test::call_service(&mut app, req).await;

        assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
    }

    fn get_node_1() -> Node {
        NodeBuilder::new("Node-123")
            .with_endpoint("tcps://127.0.0.1:8080")
            .with_display_name("Bitwise IO - Node 1")
            .with_metadata("company", "Bitwise IO")
            .build()
            .expect("Failed to build node1")
    }

    fn get_node_2() -> Node {
        NodeBuilder::new("Node-456")
            .with_endpoint("tcps://127.0.0.1:8082")
            .with_display_name("Cargill - Node 1")
            .with_metadata("company", "Cargill")
            .build()
            .expect("Failed to build node2")
    }

    fn create_test_paging_response(
        offset: usize,
        limit: usize,
        next_offset: usize,
        previous_offset: usize,
        last_offset: usize,
        total: usize,
        link: &str,
    ) -> Paging {
        let base_link = format!("{}limit={}&", link, limit);
        let current_link = format!("{}offset={}", base_link, offset);
        let first_link = format!("{}offset=0", base_link);
        let next_link = format!("{}offset={}", base_link, next_offset);
        let previous_link = format!("{}offset={}", base_link, previous_offset);
        let last_link = format!("{}offset={}", base_link, last_offset);

        Paging {
            current: current_link,
            offset,
            limit,
            total,
            first: first_link,
            prev: previous_link,
            next: next_link,
            last: last_link,
        }
    }
}
