use aws_sdk_sesv2::operation::send_email::{SendEmailInput, SendEmailOutput};
use aws_sdk_sesv2::types::Body;
use aws_smithy_mocks_experimental::{mock, Rule};
use std::cell::RefCell;
use std::fmt::Debug;
use std::sync::{Arc, Mutex};
use zero2prod::email::aws_email_client::SesClient;

pub struct AwsRuleWrapper<I: Send + Sync + Debug + 'static> {
    received_requests: Arc<Mutex<RefCell<Vec<I>>>>,
}

impl AwsRuleWrapper<SendEmailInput> {
    pub fn new_send_email_wrapper() -> Self {
        Self {
            received_requests: Arc::new(Mutex::new(RefCell::new(Vec::new()))),
        }
    }

    pub fn send_any_email_rule(&self) -> Rule {
        self.create_mock_rule(|_| true)
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
        let received_requests = self.received_requests.lock().unwrap().borrow().clone();
        assert_eq!(received_requests.len(), 1);
        received_requests.first().unwrap().clone()
    }

    fn create_mock_rule<F>(&self, matcher: F) -> Rule
    where
        F: Fn(&SendEmailInput) -> bool + Send + Sync + 'static,
    {
        let received_requests = Arc::clone(&self.received_requests);
        mock!(SesClient::send_email)
            .match_requests(move |req| {
                received_requests
                    .lock()
                    .unwrap()
                    .borrow_mut()
                    .push(req.clone());
                matcher(req)
            })
            .then_output(|| SendEmailOutput::builder().build())
    }

    fn request_body_text(req: &SendEmailInput) -> &str {
        Self::request_body(req).text().unwrap().data()
    }

    fn request_body(req: &SendEmailInput) -> &Body {
        req.content().unwrap().simple().unwrap().body().unwrap()
    }
}
