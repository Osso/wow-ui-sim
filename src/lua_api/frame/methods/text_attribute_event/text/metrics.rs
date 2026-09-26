pub(super) fn approximate_text_width(text: &str, font_size: f32) -> f32 {
    let Some(avg_char_width) = approximate_average_char_width(font_size) else {
        return 0.0;
    };

    text.lines()
        .map(|line| line.chars().count() as f32 * avg_char_width)
        .fold(0.0, f32::max)
}

pub(super) fn approximate_text_height(
    text: &str,
    font_size: f32,
    wrap_width: Option<f32>,
    spacing: f32,
) -> f32 {
    approximate_text_layout(text, font_size, wrap_width, spacing).0
}

pub(super) fn approximate_text_layout(
    text: &str,
    font_size: f32,
    wrap_width: Option<f32>,
    spacing: f32,
) -> (f32, usize) {
    let Some(line_height) = approximate_line_height(font_size) else {
        return (0.0, 0);
    };
    let Some(avg_char_width) = approximate_average_char_width(font_size) else {
        return (0.0, 0);
    };

    let positive_wrap_width = wrap_width.filter(|width| *width > 0.0);
    let line_count = text
        .lines()
        .map(|line| approximate_wrapped_line_count(line, avg_char_width, positive_wrap_width))
        .sum::<usize>()
        .max(1);

    (
        crate::render::font::spaced_text_height(
            line_height * line_count as f32,
            line_count,
            spacing,
        ),
        line_count,
    )
}

fn approximate_wrapped_line_count(
    line: &str,
    avg_char_width: f32,
    wrap_width: Option<f32>,
) -> usize {
    let Some(wrap_width) = wrap_width else {
        return 1;
    };

    let line_width = line.chars().count() as f32 * avg_char_width;
    (line_width / wrap_width).ceil().max(1.0) as usize
}

fn approximate_average_char_width(font_size: f32) -> Option<f32> {
    approximate_line_height(font_size).map(|line_height| (line_height * 0.5).max(1.0))
}

fn approximate_line_height(font_size: f32) -> Option<f32> {
    (font_size > 0.0).then(|| (font_size * 1.2).ceil())
}
