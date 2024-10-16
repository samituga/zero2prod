use aws_sdk_sesv2::config::interceptors::{
    BeforeSerializationInterceptorContextMut, FinalizerInterceptorContextMut,
};
use aws_sdk_sesv2::config::{ConfigBag, Intercept, Region, RuntimeComponents};
use aws_sdk_sesv2::error::BoxError;
use aws_sdk_sesv2::operation::send_email::{SendEmailInput, SendEmailOutput};
use aws_sdk_sesv2::types::Body;
use aws_sdk_sesv2::{Client, Config};
use aws_smithy_runtime_api::client::interceptors::context::Output;
use std::fmt::Debug;
use std::sync::{Arc, Mutex};

pub struct AwsRequestsWrapper {
    requests: Arc<Mutex<Vec<SendEmailInput>>>,
}

impl AwsRequestsWrapper {
    pub fn new(requests: Arc<Mutex<Vec<SendEmailInput>>>) -> Self {
        Self { requests }
    }

    pub fn assert_correct_destination(req: &SendEmailInput, email: &str) {
        let is_correct_destination = req
            .destination()
            .unwrap()
            .to_addresses()
            .contains(&email.to_string());
        assert!(is_correct_destination);
    }

    pub fn assert_correct_subject(req: &SendEmailInput, expected_subject: &str) {
        let content = req.content().unwrap().simple().unwrap();
        let is_correct_subject = content.subject().unwrap().data().contains(expected_subject);
        assert!(is_correct_subject);
    }

    pub fn assert_correct_body_text(req: &SendEmailInput, expected_subject: &str) {
        let is_correct_body_text = Self::request_body_text(req).contains(expected_subject);
        assert!(is_correct_body_text);
    }

    pub fn expect_one_request(&self) -> SendEmailInput {
        let requests = self.requests.lock().unwrap().clone();
        assert_eq!(
            requests.len(),
            1,
            "Expected exactly one request, but got: {:?}",
            requests.len()
        );
        requests.first().unwrap().clone()
    }

    fn request_body_text(req: &SendEmailInput) -> &str {
        Self::request_body(req).text().unwrap().data()
    }

    fn request_body(req: &SendEmailInput) -> &Body {
        req.content().unwrap().simple().unwrap().body().unwrap()
    }
}

pub fn aws_client_interceptor() -> MockAwsClientInterceptor {
    MockAwsClientInterceptor::default()
}

pub fn aws_ses_client(interceptor: MockAwsClientInterceptor) -> Client {
    Client::from_conf(
        Config::builder()
            .with_test_defaults()
            .region(Region::from_static("us-east-1"))
            .interceptor(interceptor)
            .build(),
    )
}

// TODO mock responses
// TODO Capture multiple request types
#[derive(Debug)]
pub struct MockAwsClientInterceptor {
    captured_requests: Arc<Mutex<Vec<SendEmailInput>>>,
}

impl Default for MockAwsClientInterceptor {
    fn default() -> Self {
        Self {
            captured_requests: Arc::new(Mutex::new(Vec::new())),
        }
    }
}

impl MockAwsClientInterceptor {
    pub fn captured_requests(&self) -> Arc<Mutex<Vec<SendEmailInput>>> {
        self.captured_requests.clone()
    }
}

impl Intercept for MockAwsClientInterceptor {
    fn name(&self) -> &'static str {
        "MockInterceptor"
    }

    fn modify_before_serialization(
        &self,
        context: &mut BeforeSerializationInterceptorContextMut<'_>,
        _runtime_components: &RuntimeComponents,
        _cfg: &mut ConfigBag,
    ) -> Result<(), BoxError> {
        let input = context.input();
        if let Some(typed_input) = input.downcast_ref::<SendEmailInput>() {
            self.captured_requests
                .lock()
                .unwrap()
                .push(typed_input.clone());
        } else {
            panic!("Interceptor only handles SendEmailInput for now");
        }
        Ok(())
    }

    fn modify_before_attempt_completion(
        &self,
        context: &mut FinalizerInterceptorContextMut<'_>,
        _runtime_components: &RuntimeComponents,
        _cfg: &mut ConfigBag,
    ) -> Result<(), BoxError> {
        context
            .inner_mut()
            .set_output_or_error(Ok(Output::erase(SendEmailOutput::builder().build())));
        Ok(())
    }
}
