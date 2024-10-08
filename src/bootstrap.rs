use crate::configuration::Settings;
use crate::email::aws_email_client::{SesClient, SesClientFactory};
use crate::email::email_client::{EmailClient, EmailClientProvider};
use std::sync::Arc;

pub struct Dependencies<T: EmailClient> {
    pub email_client_provider: Arc<dyn EmailClientProvider<T> + Send + Sync>,
}

pub fn build_dependencies(configuration: &Settings) -> Dependencies<SesClient> {
    Dependencies {
        email_client_provider: Arc::new(SesClientFactory::new(&configuration.aws)),
    }
}
