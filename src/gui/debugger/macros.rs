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
