//! The metadata/bitmap subset used by Amante's UI, backed by Fontations.
//! Actual text rendering remains with egui. Invalid files fail explicitly.

use skrifa::{
    FontRef, GlyphId, MetadataProvider, Tag,
    attribute::Style,
    bitmap::BitmapData,
    instance::{LocationRef, Size},
    outline::{DrawSettings, OutlinePen},
    raw::{FileRef, ReadError, TableProvider},
    string::StringId,
};

pub struct Face<'a>(FontRef<'a>);

impl<'a> Face<'a> {
    pub fn parse(data: &'a [u8], index: u32) -> Result<Self, ReadError> {
        let font = FontRef::from_index(data, index)?;
        // Keep damaged system fonts away from the renderer, not merely files
        // with a valid sfnt directory.
        font.head()?;
        font.hhea()?;
        font.maxp()?;
        Ok(Self(font))
    }

    pub fn name(&self, id: u16) -> Option<String> {
        self.0
            .localized_strings(StringId::new(id))
            .english_or_first()
            .map(|s| s.to_string())
    }

    pub fn weight(&self) -> u16 {
        self.0.attributes().weight.value() as u16
    }

    pub fn is_italic(&self) -> bool {
        self.0.attributes().style != Style::Normal
    }

    pub fn is_bold(&self) -> bool {
        use skrifa::raw::tables::{head::MacStyle, os2::SelectionFlags};
        self.weight() >= 700
            || self.0.os2().is_ok_and(|os2| os2.fs_selection().contains(SelectionFlags::BOLD))
            || self.0.head().is_ok_and(|head| head.mac_style().contains(MacStyle::BOLD))
    }

    pub fn metrics(&self) -> skrifa::metrics::Metrics {
        self.0.metrics(Size::unscaled(), LocationRef::default())
    }

    pub fn units_per_em(&self) -> u16 {
        self.metrics().units_per_em
    }
    pub fn ascender(&self) -> f32 {
        self.metrics().ascent
    }
    pub fn descender(&self) -> f32 {
        self.metrics().descent
    }
    pub fn line_gap(&self) -> f32 {
        self.metrics().leading
    }
    pub fn capital_height(&self) -> Option<f32> {
        self.metrics().cap_height
    }
    pub fn x_height(&self) -> Option<f32> {
        self.metrics().x_height
    }

    pub fn has_weight_axis(&self) -> bool {
        self.0.axes().iter().any(|axis| axis.tag() == Tag::new(b"wght"))
    }

    pub fn glyph_index(&self, c: char) -> Option<GlyphId> {
        self.0.charmap().map(c)
    }

    pub fn png_glyph(&self, glyph: GlyphId, ppem: u16) -> Option<&'a [u8]> {
        match self.0.bitmap_strikes().glyph_for_size(Size::new(f32::from(ppem)), glyph)?.data {
            BitmapData::Png(bytes) => Some(bytes),
            _ => None,
        }
    }

    pub fn has_outline(&self, c: char) -> bool {
        let Some(glyph) = self.glyph_index(c).and_then(|id| self.0.outline_glyphs().get(id)) else {
            return false;
        };
        let mut pen = Probe(false);
        glyph
            .draw(DrawSettings::unhinted(Size::unscaled(), LocationRef::default()), &mut pen)
            .is_ok()
            && pen.0
    }
}

/// Collection count, bounded by the caller before scanning system files.
pub fn face_count(data: &[u8]) -> u32 {
    match FileRef::new(data) {
        Ok(FileRef::Font(_)) => 1,
        Ok(FileRef::Collection(fonts)) => fonts.len(),
        Err(_) => 0,
    }
}

struct Probe(bool);
impl OutlinePen for Probe {
    fn move_to(&mut self, _: f32, _: f32) {}
    fn line_to(&mut self, _: f32, _: f32) {
        self.0 = true;
    }
    fn quad_to(&mut self, _: f32, _: f32, _: f32, _: f32) {
        self.0 = true;
    }
    fn curve_to(&mut self, _: f32, _: f32, _: f32, _: f32, _: f32, _: f32) {
        self.0 = true;
    }
    fn close(&mut self) {}
}

#[cfg(test)]
mod tests {
    use super::*;
    const REGULAR: &[u8] = include_bytes!("../../aurora-viewer/assets/fonts/Inter-Regular.ttf");
    const SEMIBOLD: &[u8] = include_bytes!("../../aurora-viewer/assets/fonts/Inter-SemiBold.ttf");
    const EMOJI: &[u8] = include_bytes!("../../../assets/emoji/NotoColorEmoji.ttf");

    #[test]
    fn metadata_and_outlines_preserve_regular_and_semibold() {
        let regular = Face::parse(REGULAR, 0).unwrap();
        let semi = Face::parse(SEMIBOLD, 0).unwrap();
        assert_eq!(regular.name(16).or_else(|| regular.name(1)).as_deref(), Some("Inter"));
        assert_eq!(regular.weight(), 400);
        assert_eq!(semi.weight(), 600);
        assert!(!regular.is_italic());
        assert!(regular.has_outline('A'));
        assert!(!regular.has_outline(' '));
        let m = regular.metrics();
        assert!(m.units_per_em > 0 && m.ascent > 0.0 && m.descent < 0.0);
        assert!(regular.glyph_index('\u{10ffff}').is_none());
    }

    #[test]
    fn emoji_bitmap_is_still_a_png() {
        let face = Face::parse(EMOJI, 0).unwrap();
        let glyph = face.glyph_index('😀').unwrap();
        let png = face.png_glyph(glyph, 128).unwrap();
        assert!(png.starts_with(b"\x89PNG\r\n\x1a\n"));
        assert!(!face.has_outline('😀'));
    }

    #[test]
    fn damaged_files_and_invalid_collection_indices_are_rejected() {
        for data in [&b"not a font"[..], &REGULAR[..64], &[][..]] {
            assert!(Face::parse(data, 0).is_err());
        }
        assert_eq!(face_count(REGULAR), 1);
        assert_eq!(face_count(b""), 0);
        assert!(Face::parse(REGULAR, 1).is_err());
    }

    #[test]
    fn all_bundled_families_keep_their_names_weights_and_script_coverage() {
        for (name, regular, emphasis, max_weight) in [
            ("Inter", REGULAR, SEMIBOLD, 600),
            (
                "Noto Sans",
                &include_bytes!("../../aurora-viewer/assets/fonts/NotoSans-Regular.ttf")[..],
                &include_bytes!("../../aurora-viewer/assets/fonts/NotoSans-SemiBold.ttf")[..],
                600,
            ),
            (
                "Roboto",
                &include_bytes!("../../aurora-viewer/assets/fonts/Roboto-Regular.ttf")[..],
                &include_bytes!("../../aurora-viewer/assets/fonts/Roboto-Medium.ttf")[..],
                600,
            ),
            (
                "OpenDyslexic",
                &include_bytes!("../../aurora-viewer/assets/fonts/OpenDyslexic-Regular.otf")[..],
                &include_bytes!("../../aurora-viewer/assets/fonts/OpenDyslexic-Bold.otf")[..],
                800,
            ),
        ] {
            for (bytes, is_emphasis) in [(regular, false), (emphasis, true)] {
                let face = Face::parse(bytes, 0).unwrap();
                assert_eq!(face.name(16).or_else(|| face.name(1)).as_deref(), Some(name));
                if is_emphasis {
                    assert!((500..=max_weight).contains(&face.weight()), "{name}");
                } else {
                    assert_eq!(face.weight(), 400, "{name}");
                }
                for c in "AaéñΩЖ".chars() {
                    assert!(face.has_outline(c), "{name}: {c}");
                }
            }
        }
    }
}
