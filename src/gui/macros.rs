/// Helper macro to append a text to a job with a specific color.
#[macro_export]
macro_rules! monospace_append {
    ($job:expr, $text:expr, $color:expr) => {
        $job.append(
            &$text.to_string(),
            0.0,
            TextFormat {
                font_id: FontId::monospace(12.0),
                color: $color,
                ..Default::default()
            },
        );
    };
}

/// Helper macro to create a `RichText` with a specific style
/// that is suitable for the top bar menu.
#[macro_export]
macro_rules! menu_text {
    ($text:expr) => {
        $crate::egui::RichText::new($text).size(14.0)
    };
}
