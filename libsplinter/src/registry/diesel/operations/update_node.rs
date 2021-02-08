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

//! Provides the "update node" operation for the `DieselRegistry`.

use diesel::{
    dsl::{delete, insert_into, update},
    prelude::*,
};

use crate::registry::{
    check_node_required_fields_are_not_empty,
    diesel::{
        models::{NodeEndpointsModel, NodeKeysModel, NodeMetadataModel, NodesModel},
        schema::{
            splinter_nodes, splinter_nodes_endpoints, splinter_nodes_keys, splinter_nodes_metadata,
        },
    },
    InvalidNodeError, Node, RegistryError,
};

use super::RegistryOperations;

pub(in crate::registry::diesel) trait RegistryUpdateNodeOperation {
    fn update_node(&self, node: Node) -> Result<(), RegistryError>;
}

#[cfg(feature = "postgres")]
impl<'a> RegistryUpdateNodeOperation for RegistryOperations<'a, diesel::pg::PgConnection> {
    fn update_node(&self, node: Node) -> Result<(), RegistryError> {
        // Verify that the node's required fields are non-empty
        check_node_required_fields_are_not_empty(&node)?;

        self.conn.transaction::<(), _, _>(|| {
            // Verify that the node's endpoints are unique.
            let filters = node
                .endpoints
                .iter()
                .map(|endpoint| endpoint.to_string())
                .collect::<Vec<_>>();

            let duplicate_endpoint = splinter_nodes_endpoints::table
                .filter(splinter_nodes_endpoints::endpoint.eq_any(filters))
                .first::<NodeEndpointsModel>(self.conn)
                .optional()
                .map_err(|err| {
                    RegistryError::general_error_with_source(
                        "Failed to check for duplicate endpoints",
                        Box::new(err),
                    )
                })?;

            if let Some(endpoint) = duplicate_endpoint {
                return Err(RegistryError::from(InvalidNodeError::DuplicateEndpoint(
                    endpoint.endpoint,
                )));
            }

            // Check if the node exists
            let existing_node = splinter_nodes::table
                .find(&node.identity)
                .first::<NodesModel>(self.conn)
                .optional()
                .map_err(|err| {
                    RegistryError::general_error_with_source(
                        "Failed to check if node already exists",
                        Box::new(err),
                    )
                })?;

            if existing_node.is_some() {
                // Update existing node
                update(splinter_nodes::table.find(&node.identity))
                    .set(splinter_nodes::display_name.eq(&node.display_name))
                    .execute(self.conn)
                    .map_err(|err| {
                        RegistryError::general_error_with_source(
                            "Failed to update node",
                            Box::new(err),
                        )
                    })?;
                // Remove old endpoints, keys, and metadata for the node
                delete(
                    splinter_nodes_endpoints::table
                        .filter(splinter_nodes_endpoints::identity.eq(&node.identity)),
                )
                .execute(self.conn)
                .map_err(|err| {
                    RegistryError::general_error_with_source(
                        "Failed to remove old endpoints",
                        Box::new(err),
                    )
                })?;
                delete(
                    splinter_nodes_keys::table
                        .filter(splinter_nodes_keys::identity.eq(&node.identity)),
                )
                .execute(self.conn)
                .map_err(|err| {
                    RegistryError::general_error_with_source(
                        "Failed to remove old keys",
                        Box::new(err),
                    )
                })?;
                delete(
                    splinter_nodes_metadata::table
                        .filter(splinter_nodes_metadata::identity.eq(&node.identity)),
                )
                .execute(self.conn)
                .map_err(|err| {
                    RegistryError::general_error_with_source(
                        "Failed to remove old metadata",
                        Box::new(err),
                    )
                })?;

                // Add endpoints, keys, and metadata for the node
                let endpoints: Vec<NodeEndpointsModel> = Vec::from(&node);
                insert_into(splinter_nodes_endpoints::table)
                    .values(&endpoints)
                    .execute(self.conn)
                    .map_err(|err| {
                        RegistryError::general_error_with_source(
                            "Failed to update node endpoints",
                            Box::new(err),
                        )
                    })?;
                let keys: Vec<NodeKeysModel> = Vec::from(&node);
                insert_into(splinter_nodes_keys::table)
                    .values(&keys)
                    .execute(self.conn)
                    .map_err(|err| {
                        RegistryError::general_error_with_source(
                            "Failed to update node keys",
                            Box::new(err),
                        )
                    })?;
                let metadata: Vec<NodeMetadataModel> = Vec::from(&node);
                insert_into(splinter_nodes_metadata::table)
                    .values(&metadata)
                    .execute(self.conn)
                    .map_err(|err| {
                        RegistryError::general_error_with_source(
                            "Failed to update node metadata",
                            Box::new(err),
                        )
                    })?;

                Ok(())
            } else {
                Err(RegistryError::general_error("Node does not exist"))
            }
        })
    }
}

#[cfg(feature = "sqlite")]
impl<'a> RegistryUpdateNodeOperation for RegistryOperations<'a, diesel::sqlite::SqliteConnection> {
    fn update_node(&self, node: Node) -> Result<(), RegistryError> {
        // Verify that the node's required fields are non-empty
        check_node_required_fields_are_not_empty(&node)?;

        self.conn.transaction::<(), _, _>(|| {
            // Verify that the node's endpoints are unique.
            let filters = node
                .endpoints
                .iter()
                .map(|endpoint| endpoint.to_string())
                .collect::<Vec<_>>();

            let duplicate_endpoint = splinter_nodes_endpoints::table
                .filter(splinter_nodes_endpoints::endpoint.eq_any(filters))
                .first::<NodeEndpointsModel>(self.conn)
                .optional()
                .map_err(|err| {
                    RegistryError::general_error_with_source(
                        "Failed to check for duplicate endpoints",
                        Box::new(err),
                    )
                })?;

            if let Some(endpoint) = duplicate_endpoint {
                return Err(RegistryError::from(InvalidNodeError::DuplicateEndpoint(
                    endpoint.endpoint,
                )));
            }

            // Check if the node exists
            let existing_node = splinter_nodes::table
                .find(&node.identity)
                .first::<NodesModel>(self.conn)
                .optional()
                .map_err(|err| {
                    RegistryError::general_error_with_source(
                        "Failed to check if node already exists",
                        Box::new(err),
                    )
                })?;

            if existing_node.is_some() {
                // Update existing node
                update(splinter_nodes::table.find(&node.identity))
                    .set(splinter_nodes::display_name.eq(&node.display_name))
                    .execute(self.conn)
                    .map_err(|err| {
                        RegistryError::general_error_with_source(
                            "Failed to update node",
                            Box::new(err),
                        )
                    })?;
                // Remove old endpoints, keys, and metadata for the node
                delete(
                    splinter_nodes_endpoints::table
                        .filter(splinter_nodes_endpoints::identity.eq(&node.identity)),
                )
                .execute(self.conn)
                .map_err(|err| {
                    RegistryError::general_error_with_source(
                        "Failed to remove old endpoints",
                        Box::new(err),
                    )
                })?;
                delete(
                    splinter_nodes_keys::table
                        .filter(splinter_nodes_keys::identity.eq(&node.identity)),
                )
                .execute(self.conn)
                .map_err(|err| {
                    RegistryError::general_error_with_source(
                        "Failed to remove old keys",
                        Box::new(err),
                    )
                })?;
                delete(
                    splinter_nodes_metadata::table
                        .filter(splinter_nodes_metadata::identity.eq(&node.identity)),
                )
                .execute(self.conn)
                .map_err(|err| {
                    RegistryError::general_error_with_source(
                        "Failed to remove old metadata",
                        Box::new(err),
                    )
                })?;

                // Add endpoints, keys, and metadata for the node
                let endpoints: Vec<NodeEndpointsModel> = Vec::from(&node);
                insert_into(splinter_nodes_endpoints::table)
                    .values(&endpoints)
                    .execute(self.conn)
                    .map_err(|err| {
                        RegistryError::general_error_with_source(
                            "Failed to update node endpoints",
                            Box::new(err),
                        )
                    })?;
                let keys: Vec<NodeKeysModel> = Vec::from(&node);
                insert_into(splinter_nodes_keys::table)
                    .values(&keys)
                    .execute(self.conn)
                    .map_err(|err| {
                        RegistryError::general_error_with_source(
                            "Failed to update node keys",
                            Box::new(err),
                        )
                    })?;
                let metadata: Vec<NodeMetadataModel> = Vec::from(&node);
                insert_into(splinter_nodes_metadata::table)
                    .values(&metadata)
                    .execute(self.conn)
                    .map_err(|err| {
                        RegistryError::general_error_with_source(
                            "Failed to update node metadata",
                            Box::new(err),
                        )
                    })?;

                Ok(())
            } else {
                Err(RegistryError::general_error("Node does not exist"))
            }
        })
    }
}