use crate::configuration::Settings;
use crate::email::aws_email_client::SesClientFactory;
use crate::email::email_client::{EmailClient, EmailClientProvider};
use std::sync::Arc;

pub struct Dependencies {
    pub email_client: Arc<dyn EmailClient>,
}

pub async fn build_dependencies(configuration: &Settings) -> Dependencies {
    let email_client = Arc::new(
        SesClientFactory::new(&configuration.aws)
            .email_client()
            .await,
    );
    Dependencies { email_client }
}
