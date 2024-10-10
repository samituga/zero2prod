use aws_sdk_sesv2::operation::send_email::SendEmailOutput;
use aws_smithy_mocks_experimental::{mock, Rule};
use zero2prod::email::aws_email_client::SesClient;

pub fn send_any_email_rule() -> Rule {
    mock!(SesClient::send_email)
        .match_requests(|_| true)
        .then_output(|| SendEmailOutput::builder().build())
}

pub fn send_confirmation_email_with_a_link_rule(email: String) -> Rule {
    let get_link = |s: &str| {
        let links: Vec<_> = linkify::LinkFinder::new()
            .links(s)
            .filter(|l| *l.kind() == linkify::LinkKind::Url)
            .collect();
        assert_eq!(links.len(), 1);
        links[0].as_str().to_owned()
    };

    mock!(SesClient::send_email)
        .match_requests(move |req| {
            let is_correct_destination = req
                .destination()
                .unwrap()
                .to_addresses()
                .contains(&email.to_string());
            let content = req.content().unwrap().simple().unwrap();
            let is_correct_subject = content.subject().unwrap().data().contains("Welcome");

            let body = content.body().unwrap();

            let body_html = body.html().unwrap().data();
            let body_text = body.text().unwrap().data();

            let is_correct_body = body_text.contains("Welcome to our newsletter!");

            let body_html_link = get_link(body_html);
            let body_text_link = get_link(body_text);

            let is_identical_links = body_html_link == body_text_link;

            is_correct_destination && is_correct_subject && is_correct_body && is_identical_links
        })
        .then_output(|| {
            SendEmailOutput::builder()
                .message_id("newsletter-email")
                .build()
        })
}
