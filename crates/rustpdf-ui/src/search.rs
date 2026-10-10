//! rustpdf-ui/search — In-document find bar and match navigation.

use egui::{vec2, Align, Color32, Layout, RichText, TextEdit, Ui};
use std::ops::Range;

#[derive(Clone, Debug, Default)]
pub struct SearchState {
    pub is_open: bool,
    pub query: String,
    pub case_sensitive: bool,
    pub whole_words: bool,
    pub active_match_index: usize,
    pub total_matches: usize,
    pub page_matches: Vec<(usize, Vec<Range<usize>>)>, // (page_index, glyph_ranges)
}

impl SearchState {
    pub fn open(&mut self) {
        self.is_open = true;
    }

    pub fn close(&mut self) {
        self.is_open = false;
        self.query.clear();
        self.page_matches.clear();
        self.total_matches = 0;
        self.active_match_index = 0;
    }

    pub fn next_match(&mut self) {
        if self.total_matches > 0 {
            self.active_match_index = (self.active_match_index + 1) % self.total_matches;
        }
    }

    pub fn prev_match(&mut self) {
        if self.total_matches > 0 {
            if self.active_match_index == 0 {
                self.active_match_index = self.total_matches - 1;
            } else {
                self.active_match_index -= 1;
            }
        }
    }
}

pub struct SearchBarComponent;

impl SearchBarComponent {
    pub fn render(
        ui: &mut Ui,
        state: &mut SearchState,
        accent_color: Color32,
    ) {
        if !state.is_open {
            return;
        }

        ui.horizontal(|ui| {
            ui.spacing_mut().item_spacing = vec2(6.0, 4.0);

            ui.label(RichText::new("🔍").size(14.0));

            let response = ui.add(
                TextEdit::singleline(&mut state.query)
                    .hint_text("Find in document...")
                    .desired_width(180.0),
            );

            if response.changed() {
                // Reset match counter
                state.active_match_index = 0;
            }

            if state.total_matches > 0 {
                ui.label(
                    RichText::new(format!(
                        "{}/{}",
                        state.active_match_index + 1,
                        state.total_matches
                    ))
                    .size(12.0)
                    .color(accent_color),
                );
            } else if !state.query.trim().is_empty() {
                ui.label(
                    RichText::new("0 matches")
                        .size(12.0)
                        .color(Color32::from_gray(140)),
                );
            }

            if ui.button("▲").on_hover_text("Previous match (Shift+Enter)").clicked() {
                state.prev_match();
            }

            if ui.button("▼").on_hover_text("Next match (Enter)").clicked() {
                state.next_match();
            }

            ui.checkbox(&mut state.case_sensitive, "Aa")
                .on_hover_text("Match case");

            ui.checkbox(&mut state.whole_words, "W")
                .on_hover_text("Match whole words");

            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                if ui.button("✖").clicked() {
                    state.close();
                }
            });
        });

        ui.separator();
    }
}
