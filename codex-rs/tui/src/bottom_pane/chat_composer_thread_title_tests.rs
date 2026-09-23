use super::super::tests::snapshot_composer_state_with_width;

#[test]
fn composer_thread_title_on_top_right_rule() {
    snapshot_composer_state_with_width(
        "composer_thread_title_on_top_right_rule",
        /*width*/ 80,
        /*enhanced_keys_supported*/ false,
        |composer| {
            composer.thread_name = Some("Current conversation".to_string());
        },
    );
}

#[test]
fn composer_thread_title_truncates_wide_characters() {
    snapshot_composer_state_with_width(
        "composer_thread_title_truncates_wide_characters",
        /*width*/ 40,
        /*enhanced_keys_supported*/ false,
        |composer| {
            composer.thread_name = Some("\u{5bf9}\u{8bdd}\u{540d}\u{79f0}".repeat(/*n*/ 10));
        },
    );
}

#[test]
fn composer_thread_title_tiny_width() {
    snapshot_composer_state_with_width(
        "composer_thread_title_tiny_width",
        /*width*/ 8,
        /*enhanced_keys_supported*/ false,
        |composer| {
            composer.thread_name = Some("Current conversation".to_string());
        },
    );
}
