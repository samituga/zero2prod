use aws_sdk_sesv2::operation::send_email::SendEmailOutput;
use aws_smithy_mocks_experimental::{mock, Rule};
use zero2prod::email::aws_email_client::SesClient;

pub fn send_email_rule(email: String) -> Rule {
    mock!(SesClient::send_email)
        .match_requests(move |req| {
            let is_correct_destination = req
                .destination()
                .unwrap()
                .to_addresses()
                .contains(&email.to_string());
            let content = req.content().unwrap().simple().unwrap();
            let is_correct_subject = content.subject().unwrap().data().contains("Welcome");
            let is_correct_body = content
                .body()
                .unwrap()
                .text()
                .unwrap()
                .data()
                .contains("Welcome to our newsletter!");
            is_correct_destination && is_correct_subject && is_correct_body
        })
        .then_output(|| {
            SendEmailOutput::builder()
                .message_id("newsletter-email")
                .build()
        })
}
