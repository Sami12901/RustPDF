//! rustpdf-ui/search — Whole-document text search, hit bounding boxes, and navigation.

use egui::{vec2, Align, Color32, Key, Layout, RichText, TextEdit, Ui};
use rustpdf_engine::DocumentSession;

/// An individual hit within the document.
#[derive(Clone, Debug, PartialEq)]
pub struct SearchHit {
    pub page_index: usize,
    /// Bounding rectangle in PDF points: [x, y, width, height]
    pub rect: [f32; 4],
    pub text: String,
}

#[derive(Clone, Debug, Default)]
pub struct SearchState {
    pub is_open: bool,
    pub query: String,
    pub case_sensitive: bool,
    pub whole_words: bool,
    pub active_match_index: usize,
    pub hits: Vec<SearchHit>,
    pub last_searched_query: String,
    pub last_searched_case: bool,
    pub last_searched_words: bool,
    pub last_doc_id: Option<u64>,
}

impl SearchState {
    pub fn open(&mut self) {
        self.is_open = true;
    }

    pub fn close(&mut self) {
        self.is_open = false;
        self.query.clear();
        self.hits.clear();
        self.active_match_index = 0;
        self.last_searched_query.clear();
    }

    pub fn next_match(&mut self) -> Option<usize> {
        if !self.hits.is_empty() {
            self.active_match_index = (self.active_match_index + 1) % self.hits.len();
            return self.hits.get(self.active_match_index).map(|h| h.page_index);
        }
        None
    }

    pub fn prev_match(&mut self) -> Option<usize> {
        if !self.hits.is_empty() {
            if self.active_match_index == 0 {
                self.active_match_index = self.hits.len() - 1;
            } else {
                self.active_match_index -= 1;
            }
            return self.hits.get(self.active_match_index).map(|h| h.page_index);
        }
        None
    }

    pub fn current_match(&self) -> Option<&SearchHit> {
        self.hits.get(self.active_match_index)
    }

    pub fn total_matches(&self) -> usize {
        self.hits.len()
    }

    /// Scans the document session for matching text across all pages.
    pub fn update_search(
        &mut self,
        session: &DocumentSession,
        doc_id: u64,
        force: bool,
    ) {
        let q = self.query.trim();
        if q.is_empty() {
            self.hits.clear();
            self.last_searched_query.clear();
            self.active_match_index = 0;
            return;
        }

        if !force
            && self.last_doc_id == Some(doc_id)
            && self.last_searched_query == q
            && self.last_searched_case == self.case_sensitive
            && self.last_searched_words == self.whole_words
        {
            return;
        }

        self.last_doc_id = Some(doc_id);
        self.last_searched_query = q.to_string();
        self.last_searched_case = self.case_sensitive;
        self.last_searched_words = self.whole_words;
        self.hits.clear();

        let num_pages = session.page_count();
        let target_query = if self.case_sensitive {
            q.to_string()
        } else {
            q.to_lowercase()
        };

        for page_idx in 0..num_pages {
            let blocks = session.get_page_text_blocks(page_idx);
            for block in &blocks {
                let text_to_check = if self.case_sensitive {
                    block.text.clone()
                } else {
                    block.text.to_lowercase()
                };

                let mut search_idx = 0;
                while let Some(found_pos) = text_to_check[search_idx..].find(&target_query) {
                    let abs_pos = search_idx + found_pos;
                    let match_len = target_query.len();
                    let end_pos = abs_pos + match_len;

                    let is_whole_word = if self.whole_words {
                        let prev_boundary = if abs_pos == 0 {
                            true
                        } else {
                            let prev_ch = text_to_check[..abs_pos].chars().next_back().unwrap();
                            !prev_ch.is_alphanumeric()
                        };
                        let next_boundary = if end_pos >= text_to_check.len() {
                            true
                        } else {
                            let next_ch = text_to_check[end_pos..].chars().next().unwrap();
                            !next_ch.is_alphanumeric()
                        };
                        prev_boundary && next_boundary
                    } else {
                        true
                    };

                    if is_whole_word {
                        let block_chars = block.text.chars().count().max(1) as f32;
                        let char_width = block.rect[2] / block_chars;
                        let start_char_offset = block.text[..abs_pos].chars().count() as f32;
                        let match_char_count =
                            block.text[abs_pos..end_pos].chars().count().max(1) as f32;

                        let hit_x = block.rect[0] + (start_char_offset * char_width);
                        let hit_w = (match_char_count * char_width).max(8.0);
                        let hit_rect = [hit_x, block.rect[1], hit_w, block.rect[3]];

                        self.hits.push(SearchHit {
                            page_index: page_idx,
                            rect: hit_rect,
                            text: block.text[abs_pos..end_pos].to_string(),
                        });
                    }

                    search_idx = abs_pos + match_len.max(1);
                    if search_idx >= text_to_check.len() {
                        break;
                    }
                }
            }
        }

        if self.active_match_index >= self.hits.len() {
            self.active_match_index = 0;
        }
    }
}

pub struct SearchBarComponent;

impl SearchBarComponent {
    /// Renders the search bar and returns `Some(target_page_index)` if navigation is triggered.
    pub fn render(
        ui: &mut Ui,
        state: &mut SearchState,
        accent_color: Color32,
    ) -> Option<usize> {
        if !state.is_open {
            return None;
        }

        let mut nav_page = None;

        ui.horizontal(|ui| {
            ui.spacing_mut().item_spacing = vec2(6.0, 4.0);

            ui.label(RichText::new("🔍").size(14.0));

            let response = ui.add(
                TextEdit::singleline(&mut state.query)
                    .hint_text("Find in document...")
                    .desired_width(180.0),
            );

            if response.changed() {
                state.active_match_index = 0;
                // If hits exist on text change, navigate to the first hit's page
                if let Some(first_hit) = state.hits.first() {
                    nav_page = Some(first_hit.page_index);
                }
            }

            if response.lost_focus() && ui.input(|i| i.key_pressed(Key::Enter)) {
                if ui.input(|i| i.modifiers.shift) {
                    nav_page = state.prev_match();
                } else {
                    nav_page = state.next_match();
                }
            }

            let total = state.total_matches();
            if total > 0 {
                ui.label(
                    RichText::new(format!("{}/{}", state.active_match_index + 1, total))
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

            if ui
                .button("▲")
                .on_hover_text("Previous match (Shift+Enter)")
                .clicked()
            {
                nav_page = state.prev_match();
            }

            if ui.button("▼").on_hover_text("Next match (Enter)").clicked() {
                nav_page = state.next_match();
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

        nav_page
    }
}

