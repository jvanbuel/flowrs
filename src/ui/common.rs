use ratatui::{
    style::Style,
    text::{Line, Span},
};

use super::constants::AirflowStateColor;
use super::theme::theme;

pub fn create_headers<'a>(
    headers: impl IntoIterator<Item = &'a str>,
) -> impl Iterator<Item = Line<'a>> {
    let default_style = theme().default_style;
    headers
        .into_iter()
        .map(move |h| Line::from(h).style(default_style).centered())
}

pub fn state_to_colored_square<'a>(color: AirflowStateColor) -> Span<'a> {
    Span::styled("■", Style::default().fg(color.into()))
}

const GANTT_BAR: &str = "▃";

/// A `▃ label` swatch in the given Gantt phase/state color, with `label` left-
/// padded to `pad` columns so swatches line up in a multi-column legend.
fn gantt_swatch(state: AirflowStateColor, label: &str, pad: usize) -> [Span<'static>; 2] {
    [
        Span::styled(GANTT_BAR, Style::default().fg(state.into())),
        Span::raw(format!(" {label:<pad$}")),
    ]
}

/// Complete Gantt color key — the two lead-in phases plus every task state the
/// run segment can take — laid out two per line for the `?` help popup.
pub fn gantt_legend_full() -> Vec<Line<'static>> {
    // (state, label) in the order Airflow phases progress, then terminal states.
    let items = [
        (AirflowStateColor::Scheduled, "scheduled"),
        (AirflowStateColor::Queued, "queued"),
        (AirflowStateColor::Running, "running"),
        (AirflowStateColor::Success, "success"),
        (AirflowStateColor::Failed, "failed"),
        (AirflowStateColor::UpForRetry, "up_for_retry"),
        (AirflowStateColor::UpForReschedule, "up_for_reschedule"),
        (AirflowStateColor::Skipped, "skipped"),
        (AirflowStateColor::UpstreamFailed, "upstream_failed"),
    ];
    let mut lines = Vec::new();
    let mut iter = items.into_iter();
    while let Some((color, label)) = iter.next() {
        let mut spans: Vec<Span<'static>> = gantt_swatch(color, label, 18).to_vec();
        if let Some((color, label)) = iter.next() {
            spans.extend(gantt_swatch(color, label, 0));
        }
        lines.push(Line::from(spans));
    }
    lines
}

#[cfg(test)]
mod tests {
    use super::*;
    use ratatui::style::Color;

    #[test]
    fn test_full_gantt_legend_covers_all_states() {
        let lines = gantt_legend_full();
        let text: String = lines
            .iter()
            .flat_map(|l| l.spans.iter())
            .map(|s| s.content.as_ref())
            .collect();
        for label in [
            "scheduled",
            "queued",
            "running",
            "success",
            "failed",
            "up_for_retry",
            "up_for_reschedule",
            "skipped",
            "upstream_failed",
        ] {
            assert!(text.contains(label), "legend missing `{label}`");
        }

        let colors: Vec<Option<Color>> = lines
            .iter()
            .flat_map(|l| l.spans.iter())
            .map(|s| s.style.fg)
            .collect();
        for state in [
            AirflowStateColor::Scheduled,
            AirflowStateColor::Queued,
            AirflowStateColor::Failed,
            AirflowStateColor::Success,
            AirflowStateColor::UpstreamFailed,
        ] {
            assert!(colors.contains(&Some(state.into())));
        }
    }
}
