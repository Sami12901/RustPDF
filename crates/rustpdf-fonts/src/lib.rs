//! rustpdf-fonts — Standard 14 PDF font metrics and font inventory.

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum StandardFont {
    Helvetica,
    HelveticaBold,
    HelveticaOblique,
    HelveticaBoldOblique,
    TimesRoman,
    TimesBold,
    TimesItalic,
    TimesBoldItalic,
    Courier,
    CourierBold,
    CourierOblique,
    CourierBoldOblique,
    Symbol,
    ZapfDingbats,
}

impl StandardFont {
    pub fn postscript_name(self) -> &'static str {
        match self {
            Self::Helvetica => "Helvetica",
            Self::HelveticaBold => "Helvetica-Bold",
            Self::HelveticaOblique => "Helvetica-Oblique",
            Self::HelveticaBoldOblique => "Helvetica-BoldOblique",
            Self::TimesRoman => "Times-Roman",
            Self::TimesBold => "Times-Bold",
            Self::TimesItalic => "Times-Italic",
            Self::TimesBoldItalic => "Times-BoldItalic",
            Self::Courier => "Courier",
            Self::CourierBold => "Courier-Bold",
            Self::CourierOblique => "Courier-Oblique",
            Self::CourierBoldOblique => "Courier-BoldOblique",
            Self::Symbol => "Symbol",
            Self::ZapfDingbats => "ZapfDingbats",
        }
    }

    pub fn is_monospace(self) -> bool {
        matches!(
            self,
            Self::Courier | Self::CourierBold | Self::CourierOblique | Self::CourierBoldOblique
        )
    }

    /// Estimates the width of a string in points for a given font size.
    pub fn estimate_width(self, text: &str, size_pt: f32) -> f32 {
        if self.is_monospace() {
            text.chars().count() as f32 * (size_pt * 0.6)
        } else {
            // Average proportional glyph width ~ 0.5 * size_pt
            text.chars().count() as f32 * (size_pt * 0.5)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_standard_fonts() {
        let f = StandardFont::Helvetica;
        assert_eq!(f.postscript_name(), "Helvetica");
        assert!(!f.is_monospace());

        let c = StandardFont::Courier;
        assert!(c.is_monospace());
        assert!((c.estimate_width("hello", 10.0) - 30.0).abs() < 1e-4);
    }
}
