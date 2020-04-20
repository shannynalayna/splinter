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

use crate::config::PartialConfigBuilder;
use crate::config::{ConfigError, ConfigSource, PartialConfig};

use serde_derive::Deserialize;

use toml;

/// Holds configuration values defined in a toml file. This struct must be
/// treated as part of the external API of splinter because changes here
/// will impact the valid format of the config file.
#[derive(Deserialize, Default, Debug)]
struct TomlConfig {
    storage: Option<String>,
    cert_dir: Option<String>,
    ca_certs: Option<String>,
    client_cert: Option<String>,
    client_key: Option<String>,
    server_cert: Option<String>,
    server_key: Option<String>,
    service_endpoint: Option<String>,
    network_endpoints: Option<Vec<String>>,
    advertised_endpoints: Option<Vec<String>>,
    peers: Option<Vec<String>>,
    node_id: Option<String>,
    display_name: Option<String>,
    bind: Option<String>,
    #[cfg(feature = "database")]
    database: Option<String>,
    registries: Option<Vec<String>>,
    heartbeat_interval: Option<u64>,
    admin_service_coordinator_timeout: Option<u64>,
}

pub struct TomlPartialConfigBuilder {
    source: Option<ConfigSource>,
    toml_config: TomlConfig,
}

impl TomlPartialConfigBuilder {
    pub fn new(toml: String, toml_path: String) -> Result<TomlPartialConfigBuilder, ConfigError> {
        Ok(TomlPartialConfigBuilder {
            source: Some(ConfigSource::Toml { file: toml_path }),
            toml_config: toml::from_str::<TomlConfig>(&toml).map_err(ConfigError::from)?,
        })
    }
}

impl PartialConfigBuilder for TomlPartialConfigBuilder {
    fn build(self) -> Result<PartialConfig, ConfigError> {
        let source = match self.source {
            Some(s) => s,
            None => ConfigSource::Toml {
                file: String::from(""),
            },
        };

        let mut partial_config = PartialConfig::new(source);

        partial_config = partial_config
            .with_storage(self.toml_config.storage)
            .with_cert_dir(self.toml_config.cert_dir)
            .with_ca_certs(self.toml_config.ca_certs)
            .with_client_cert(self.toml_config.client_cert)
            .with_client_key(self.toml_config.client_key)
            .with_server_cert(self.toml_config.server_cert)
            .with_server_key(self.toml_config.server_key)
            .with_service_endpoint(self.toml_config.service_endpoint)
            .with_network_endpoints(self.toml_config.network_endpoints)
            .with_advertised_endpoints(self.toml_config.advertised_endpoints)
            .with_peers(self.toml_config.peers)
            .with_node_id(self.toml_config.node_id)
            .with_display_name(self.toml_config.display_name)
            .with_bind(self.toml_config.bind)
            .with_registries(self.toml_config.registries)
            .with_heartbeat_interval(self.toml_config.heartbeat_interval)
            .with_admin_service_coordinator_timeout(
                self.toml_config.admin_service_coordinator_timeout,
            );

        #[cfg(feature = "database")]
        {
            partial_config = partial_config.with_database(self.toml_config.database);
        }

        Ok(partial_config)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use toml::{map::Map, Value};

    /// Path to an example config toml file.
    static TEST_TOML: &str = "config_test.toml";

    /// Example configuration values.
    static EXAMPLE_STORAGE: &str = "yaml";
    static EXAMPLE_CA_CERTS: &str = "certs/ca.pem";
    static EXAMPLE_CLIENT_CERT: &str = "certs/client.crt";
    static EXAMPLE_CLIENT_KEY: &str = "certs/client.key";
    static EXAMPLE_SERVER_CERT: &str = "certs/server.crt";
    static EXAMPLE_SERVER_KEY: &str = "certs/server.key";
    static EXAMPLE_SERVICE_ENDPOINT: &str = "127.0.0.1:8043";
    static EXAMPLE_NODE_ID: &str = "012";
    static EXAMPLE_DISPLAY_NAME: &str = "Node 1";

    /// Converts a list of tuples to a toml Table Value used to write a toml file.
    fn get_toml_value() -> Value {
        let values = vec![
            ("storage".to_string(), EXAMPLE_STORAGE.to_string()),
            ("ca_certs".to_string(), EXAMPLE_CA_CERTS.to_string()),
            ("client_cert".to_string(), EXAMPLE_CLIENT_CERT.to_string()),
            ("client_key".to_string(), EXAMPLE_CLIENT_KEY.to_string()),
            ("server_cert".to_string(), EXAMPLE_SERVER_CERT.to_string()),
            ("server_key".to_string(), EXAMPLE_SERVER_KEY.to_string()),
            (
                "service_endpoint".to_string(),
                EXAMPLE_SERVICE_ENDPOINT.to_string(),
            ),
            ("node_id".to_string(), EXAMPLE_NODE_ID.to_string()),
            ("display_name".to_string(), EXAMPLE_DISPLAY_NAME.to_string()),
        ];

        let mut config_values = Map::new();
        values.iter().for_each(|v| {
            config_values.insert(v.0.clone(), Value::String(v.1.clone()));
        });
        Value::Table(config_values)
    }

    /// Asserts config values based on the example configuration values.
    fn assert_config_values(config: PartialConfig) {
        assert_eq!(config.storage(), Some(EXAMPLE_STORAGE.to_string()));
        assert_eq!(config.cert_dir(), None);
        assert_eq!(config.ca_certs(), Some(EXAMPLE_CA_CERTS.to_string()));
        assert_eq!(config.client_cert(), Some(EXAMPLE_CLIENT_CERT.to_string()));
        assert_eq!(config.client_key(), Some(EXAMPLE_CLIENT_KEY.to_string()));
        assert_eq!(config.server_cert(), Some(EXAMPLE_SERVER_CERT.to_string()));
        assert_eq!(config.server_key(), Some(EXAMPLE_SERVER_KEY.to_string()));
        assert_eq!(
            config.service_endpoint(),
            Some(EXAMPLE_SERVICE_ENDPOINT.to_string())
        );
        assert_eq!(config.network_endpoints(), None);
        assert_eq!(config.advertised_endpoints(), None);
        assert_eq!(config.peers(), None);
        assert_eq!(config.node_id(), Some(EXAMPLE_NODE_ID.to_string()));
        assert_eq!(
            config.display_name(),
            Some(EXAMPLE_DISPLAY_NAME.to_string())
        );
        assert_eq!(config.bind(), None);
        #[cfg(feature = "database")]
        assert_eq!(config.database(), None);
        assert_eq!(config.registries(), None);
        assert_eq!(config.heartbeat_interval(), None);
        assert_eq!(config.admin_service_coordinator_timeout(), None);
    }

    #[test]
    /// This test verifies that a PartialConfig object, constructed from the
    /// TomlPartialConfigBuilder module, contains the correct values using the following steps:
    ///
    /// 1. An example config toml is string is created.
    /// 2. A TomlPartialConfigBuilder object is constructed by passing in the toml string created
    ///    in the previous step.
    /// 3. The TomlPartialConfigBuilder object is transformed to a PartialConfig object using the
    ///    `build` method.
    ///
    /// This test then verifies the PartialConfig object built from the TomlPartialConfigBuilder
    /// object by asserting each expected value.
    fn test_toml_build() {
        // Create an example toml string.
        let toml_string = toml::to_string(&get_toml_value()).expect("Could not encode TOML value");
        // Create a TomlPartialConfigBuilder object from the toml string.
        let toml_builder = TomlPartialConfigBuilder::new(toml_string, TEST_TOML.to_string())
            .expect(&format!(
                "Unable to create TomlPartialConfigBuilder from: {}",
                TEST_TOML
            ));
        // Build a PartialConfig from the TomlPartialConfigBuilder object created.
        let built_config = toml_builder
            .build()
            .expect("Unable to build TomlPartialConfigBuilder");
        // Compare the generated PartialConfig object against the expected values.
        assert_config_values(built_config);
    }
}
