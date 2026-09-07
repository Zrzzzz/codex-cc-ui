use super::*;
use crate::app_event::AppEvent;
use pretty_assertions::assert_eq;
use tokio::sync::mpsc::unbounded_channel;

#[test]
fn token_count_stays_next_to_working_and_fits_narrow_terminals() {
    let (tx, _rx) = unbounded_channel::<AppEvent>();
    let mut widget = StatusIndicatorWidget::new(
        AppEventSender::new(tx),
        FrameRequester::test_dummy(),
        /*animations_enabled*/ false,
    );
    widget.update_output_tokens(Some("↓ ~123 tokens".into()));
    let mut timer = StatusTimer::default();
    timer.pause_at(Instant::now());
    let mut snapshots = Vec::new();
    for header in [
        "Working",
        "Checking a very long status header with 文件 and more text",
    ] {
        widget.update_header(header.into());
        for width in [20, 40, 80] {
            let lines = StatusIndicator {
                row: &widget,
                timer: &timer,
            }
            .lines(width);
            assert_eq!(lines.len(), 1);
            assert!(lines[0].width() <= usize::from(width));
            if width >= 40 {
                assert!(lines[0].to_string().contains("↓ ~123 tokens"));
            }
            snapshots.push((header, width, lines[0].to_string()));
        }
    }
    insta::assert_debug_snapshot!(snapshots);
}
