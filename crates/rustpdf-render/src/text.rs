//! Text extraction and search engine using hayro page interpretation.

use std::ops::Range;
use serde::{Deserialize, Serialize};

/// One extracted glyph with its Unicode representation and view-space bounding box.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct TextGlyph {
    pub text: String,
    /// Bounding rectangle in page points: [x0, y0, x1, y1]
    pub rect: [f32; 4],
}

/// The extracted text of a page with reading order and layout information.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct PageText {
    pub glyphs: Vec<TextGlyph>,
    pub line_of: Vec<u32>,
    pub space_before: Vec<bool>,
}

impl PageText {
    pub fn is_empty(&self) -> bool {
        self.glyphs.is_empty()
    }

    pub fn plain_text(&self) -> String {
        self.text_of(0..self.glyphs.len())
    }

    /// Reconstructs plain text for a range of glyphs with spaces and linebreaks.
    pub fn text_of(&self, range: Range<usize>) -> String {
        let mut s = String::new();
        let mut prev_line = None;

        for i in range {
            let Some(g) = self.glyphs.get(i) else { break };
            if !s.is_empty() {
                if prev_line != Some(self.line_of.get(i).copied().unwrap_or(0)) {
                    s.push('\n');
                } else if self.space_before.get(i).copied().unwrap_or(false) {
                    s.push(' ');
                }
            }
            s.push_str(&g.text);
            prev_line = self.line_of.get(i).copied();
        }

        s
    }

    /// Searches for needle within the page, returning glyph index ranges for each match.
    pub fn find(&self, needle: &str) -> Vec<Range<usize>> {
        self.find_opts(needle, false, false)
    }

    /// In-document search with case-sensitivity and whole-word matching.
    pub fn find_opts(
        &self,
        needle: &str,
        case_sensitive: bool,
        whole_words: bool,
    ) -> Vec<Range<usize>> {
        let fold = |s: &str| {
            if case_sensitive {
                s.to_string()
            } else {
                s.to_lowercase()
            }
        };

        let needle_chars: Vec<char> = fold(needle)
            .split_whitespace()
            .collect::<Vec<_>>()
            .join(" ")
            .chars()
            .collect();

        if needle_chars.is_empty() {
            return Vec::new();
        }

        let mut chars: Vec<(char, usize)> = Vec::new();
        for (i, g) in self.glyphs.iter().enumerate() {
            if i > 0
                && (self.space_before.get(i).copied().unwrap_or(false)
                    || self.line_of.get(i) != self.line_of.get(i - 1))
                && chars.last().is_some_and(|c| c.0 != ' ')
            {
                chars.push((' ', i));
            }
            for c in fold(&g.text).chars() {
                chars.push((if c.is_whitespace() { ' ' } else { c }, i));
            }
        }

        let is_word_boundary = |k: Option<&(char, usize)>| k.is_some_and(|c| c.0.is_alphanumeric());

        let mut out = Vec::new();
        let mut i = 0;
        while i + needle_chars.len() <= chars.len() {
            let matches = chars[i..i + needle_chars.len()]
                .iter()
                .map(|c| c.0)
                .eq(needle_chars.iter().copied());

            let word_ok = !whole_words
                || (!is_word_boundary(i.checked_sub(1).and_then(|p| chars.get(p)))
                    && !is_word_boundary(chars.get(i + needle_chars.len())));

            if matches && word_ok {
                let start_glyph = chars[i].1;
                let end_glyph = chars[i + needle_chars.len() - 1].1 + 1;
                out.push(start_glyph..end_glyph);
                i += needle_chars.len();
            } else {
                i += 1;
            }
        }

        out
    }

    /// Finds the glyph index nearest to view coordinates (x, y).
    pub fn nearest(&self, x: f32, y: f32) -> Option<usize> {
        self.glyphs
            .iter()
            .enumerate()
            .map(|(i, g)| {
                let dx = if x < g.rect[0] {
                    g.rect[0] - x
                } else if x > g.rect[2] {
                    x - g.rect[2]
                } else {
                    0.0
                };
                let dy = if y < g.rect[1] {
                    g.rect[1] - y
                } else if y > g.rect[3] {
                    y - g.rect[3]
                } else {
                    0.0
                };
                (i, dx + dy * 3.0)
            })
            .min_by(|a, b| a.1.total_cmp(&b.1))
            .map(|(i, _)| i)
    }
}
