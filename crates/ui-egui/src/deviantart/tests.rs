use super::*;

#[test]
fn a_rejected_refresh_token_is_recognized() {
    assert!(super::errors::login_was_rejected("The refresh token is invalid"));
    assert!(super::errors::login_was_rejected("invalid_request: The refresh token is invalid."));
    assert!(super::errors::login_was_rejected("invalid_grant"));
    assert!(!super::errors::login_was_rejected("HTTP 500"));
}

#[test]
fn settings_drop_the_refresh_token() {
    let text = settings_json(&Settings { client_id: "99".into(), refresh_token: "secret-token".into(), title: "Hill".into(), ..Settings::default() });
    let (disk, token) = without_token(&text).unwrap();
    assert_eq!(token, "secret-token");
    assert!(!disk.contains("secret-token"));
    assert!(disk.contains("99"));
    let back = settings_from_json(&with_token(&disk, &token));
    assert_eq!(back.refresh_token, "secret-token");
    assert_eq!(back.title, "Hill");
}

#[test]
fn tags_keep_letters_numbers_and_underscores() {
    assert_eq!(clean_tags("night sky, night, café, a-b"), vec!["night".to_string(), "sky".to_string(), "caf".to_string(), "ab".to_string()]);
}

#[test]
fn submit_body_carries_the_file_and_not_a_token() {
    let (_, body) = multipart("Hill", "notes", &["night".into()], "hill.png", b"PNG");
    let text = String::from_utf8_lossy(&body);
    assert!(text.contains("name=\"title\""));
    assert!(text.contains("Hill"));
    assert!(text.contains("name=\"tags[0]\""));
    assert!(text.contains("filename=\"hill.png\""));
    assert!(text.contains("PNG"));
    assert!(!text.contains("refresh"));
    assert_eq!(item_id(r#"{"status":"success","itemid":42}"#), Some(42));
    assert_eq!(stash_url(2730074306229899).as_deref(), Some("https://www.deviantart.com/stash/0qvqatlhzez"));
    assert_eq!(stash_url(0), None);
    let (_, quoted) = multipart("Hill", "notes", &[], "my \"best\" sketch.png", b"x");
    let quoted = String::from_utf8_lossy(&quoted);
    assert!(quoted.contains("filename=\"my _best_ sketch.png\""));
    assert!(!quoted.contains("filename=\"my \"best\""));
}

#[test]
fn artist_comments_omit_the_local_path() {
    let text = comments("hill.png", 1200);
    assert!(text.contains("hill.png"));
    assert!(text.contains("PNG"));
    assert!(!text.contains('/'));
    assert!(!text.contains("home"));
}
