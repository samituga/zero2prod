use crate::configuration::Settings;
use crate::email_client::SesClientProvider;
use std::sync::Arc;

pub struct Dependencies {
    pub ses_client_provider: Arc<dyn SesClientProvider + Send + Sync>,
}

pub fn build_dependencies(configuration: &Settings) -> Dependencies {
    Dependencies {
        ses_client_provider: Arc::new(configuration.aws.clone()),
    }
}
