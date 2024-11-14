use std::collections::HashMap;
use std::sync::Arc;

use tokio::sync::RwLock;
use tracing::{debug, error, info};
use wasmcloud_provider_sdk::initialize_observability;
use wasmcloud_provider_sdk::{serve_provider_exports, Context};

use crate::config::ProviderConfig;

pub(crate) mod bindings {
    wit_bindgen_wrpc::generate!();
}

use bindings::exports::wasmcloud::example::provider_executor::Handler;
use bindings::exports::wasmcloud::example::system_info;

#[derive(Default, Clone)]
pub struct CustomTemplateProvider {
    config: Arc<RwLock<ProviderConfig>>,
    linked_from: Arc<RwLock<HashMap<String, HashMap<String, String>>>>,
    linked_to: Arc<RwLock<HashMap<String, HashMap<String, String>>>>,
}

use futures::future::{pending, Future};
use std::pin::Pin;

impl CustomTemplateProvider {
    fn name() -> &'static str {
        "custom-template-provider"
    }
    pub async fn run() -> anyhow::Result<()> {
        initialize_observability!(
            Self::name(),
            std::env::var_os("PROVIDER_CUSTOM_TEMPLATE_FLAMEGRAPH_PATH")
        );
        let provider = Self::default();
        let shutdown: Pin<Box<dyn Future<Output = ()>>> = Box::pin(pending());
        // The [`serve`] function will set up RPC topics for your provider's exports and await invocations.
        // This is a generated function based on the contents in your `wit/world.wit` file.
        let connection = wasmcloud_provider_sdk::get_connection();
        serve_provider_exports(
            &connection.get_wrpc_client(connection.provider_key()),
            provider,
            shutdown,
            bindings::serve,
        )
        .await

        // If your provider has no exports, simply await the shutdown to keep the provider running
        // shutdown.await;
        // Ok(())
    }
}

impl Handler<Option<Context>> for CustomTemplateProvider {
    async fn run_command(&self, ctx: Option<Context>, command: String) -> anyhow::Result<String> {
        info!("received call to send data to linked components");
        let mut last_response = None;
        for (component_id, config) in self.linked_to.read().await.iter() {
            debug!(component_id, ?config, "sending data to component");
            let command = "ls".to_string();
            let client = wasmcloud_provider_sdk::get_connection().get_wrpc_client(component_id);
            match run_command(command).await {
                Ok(response) => {
                    last_response = Some(response);
                    info!(
                        component_id,
                        ?config,
                        ?last_response,
                        "successfully sent data to component"
                    );
                }
                Err(e) => {
                    error!(
                        component_id,
                        ?config,
                        ?e,
                        "failed to send data to component"
                    );
                }
            }
        }

        Ok(last_response.unwrap_or_else(|| "No components responded to request".to_string()))
    }
}
use std::process::Stdio;
use tokio::process::Command;

async fn run_command(command: String) -> Result<String, anyhow::Error> {
    let output = Command::new("sh")
        .arg("-c")
        .arg(&command)
        .stdout(Stdio::piped())
        .output()
        .await
        .expect("failed exec");

    Ok(String::from_utf8_lossy(&output.stdout).to_string())
}
