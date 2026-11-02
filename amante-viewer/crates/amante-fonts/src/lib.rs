//! Font metadata, outlines and embedded emoji using maintained fontations.
//! No legacy ttf-parser code is included.
use skrifa::{
    FontRef, GlyphId, MetadataProvider,
    bitmap::{BitmapData, BitmapStrikes},
    instance::{LocationRef, Size},
    outline::{DrawSettings, OutlinePen},
    raw::{FileRef, ReadError, TableProvider},
    string::StringId,
};

pub struct Face<'a> {
    font: FontRef<'a>,
    metrics: skrifa::metrics::Metrics,
}

impl<'a> Face<'a> {
    pub fn parse(data: &'a [u8], index: u32) -> Result<Self, ReadError> {
        let font = FontRef::from_index(data, index)?;
        for table in font.table_directory().table_records() {
            let end = (table.offset() as usize)
                .checked_add(table.length() as usize)
                .ok_or(ReadError::OutOfBounds)?;
            if end > data.len() {
                return Err(ReadError::OutOfBounds);
            }
        }
        // Check the required tables before exposing the face to egui.
        font.head()?;
        font.hhea()?;
        font.maxp()?;
        let metrics = font.metrics(Size::unscaled(), LocationRef::default());
        Ok(Self { font, metrics })
    }

    pub fn name(&self, id: u16) -> Option<String> {
        self.font
            .localized_strings(StringId::new(id))
            .english_or_first()
            .map(|s| s.to_string())
    }

    pub fn weight(&self) -> u16 {
        self.font.os2().map(|os2| os2.us_weight_class()).unwrap_or(400)
    }

    pub fn is_bold(&self) -> bool {
        self.weight() >= 700
    }

    pub fn is_italic(&self) -> bool {
        self.metrics.italic_angle != 0.0
            || self
                .font
                .os2()
                .is_ok_and(|t| t.fs_selection().contains(skrifa::raw::tables::os2::SelectionFlags::ITALIC))
    }

    pub fn units_per_em(&self) -> u16 {
        self.metrics.units_per_em
    }
    pub fn ascender(&self) -> f32 {
        self.metrics.ascent
    }
    pub fn descender(&self) -> f32 {
        self.metrics.descent
    }
    pub fn line_gap(&self) -> f32 {
        self.metrics.leading
    }
    pub fn capital_height(&self) -> Option<f32> {
        self.metrics.cap_height
    }
    pub fn x_height(&self) -> Option<f32> {
        self.metrics.x_height
    }
    pub fn glyph_index(&self, c: char) -> Option<GlyphId> {
        self.font.charmap().map(c).filter(|g| g.to_u32() != 0)
    }
    pub fn has_weight_axis(&self) -> bool {
        self.font.axes().iter().any(|axis| axis.tag() == skrifa::Tag::new(b"wght"))
    }
    pub fn has_outline(&self, glyph: GlyphId) -> bool {
        struct Sink(bool);
        impl OutlinePen for Sink {
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
        let Some(outline) = self.font.outline_glyphs().get(glyph) else {
            return false;
        };
        let mut sink = Sink(false);
        outline
            .draw(DrawSettings::unhinted(Size::unscaled(), LocationRef::default()), &mut sink)
            .is_ok()
            && sink.0
    }
    /// Prefer the closest strike, retaining the native PNG bytes and size.
    pub fn glyph_png(&self, glyph: GlyphId, pixels_per_em: u16) -> Option<&'a [u8]> {
        let strikes = BitmapStrikes::new(&self.font);
        let bitmap = strikes.glyph_for_size(Size::new(f32::from(pixels_per_em)), glyph)?;
        match bitmap.data {
            BitmapData::Png(bytes) => Some(bytes),
            _ => None,
        }
    }
}

pub fn fonts_in_collection(data: &[u8]) -> Option<u32> {
    match FileRef::new(data).ok()? {
        FileRef::Collection(collection) => Some(collection.len()),
        FileRef::Font(_) => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const INTER: &[u8] = include_bytes!("../../aurora-viewer/assets/fonts/Inter-Regular.ttf");
    const BOLD: &[u8] = include_bytes!("../../aurora-viewer/assets/fonts/Inter-SemiBold.ttf");
    const EMOJI: &[u8] = include_bytes!("../../../assets/emoji/NotoColorEmoji.ttf");

    #[test]
    fn every_embedded_face_keeps_its_family_weight_and_scripts() {
        let fonts: &[(&[u8], &str, u16)] = &[
            (INTER, "Inter", 400),
            (BOLD, "Inter", 600),
            (
                include_bytes!("../../aurora-viewer/assets/fonts/NotoSans-Regular.ttf"),
                "Noto Sans",
                400,
            ),
            (
                include_bytes!("../../aurora-viewer/assets/fonts/NotoSans-SemiBold.ttf"),
                "Noto Sans",
                600,
            ),
            (include_bytes!("../../aurora-viewer/assets/fonts/Roboto-Regular.ttf"), "Roboto", 400),
            (include_bytes!("../../aurora-viewer/assets/fonts/Roboto-Medium.ttf"), "Roboto", 500),
            (
                include_bytes!("../../aurora-viewer/assets/fonts/OpenDyslexic-Regular.otf"),
                "OpenDyslexic",
                400,
            ),
            (
                include_bytes!("../../aurora-viewer/assets/fonts/OpenDyslexic-Bold.otf"),
                "OpenDyslexic",
                800,
            ),
        ];
        for &(bytes, family, weight) in fonts {
            let face = Face::parse(bytes, 0).unwrap();
            assert_eq!(face.name(16).or_else(|| face.name(1)).as_deref(), Some(family));
            assert_eq!(face.weight(), weight);
            assert!(face.units_per_em() > 0);
            assert!(face.ascender() > face.descender());
            for c in "AgéΩЯ".chars() {
                assert!(face.glyph_index(c).is_some(), "{family}: {c}");
            }
            let glyph = face.glyph_index('A').unwrap();
            assert!(face.has_outline(glyph), "{family}: no outline");
        }
    }

    #[test]
    fn bitmap_emoji_remain_native_pngs() {
        let face = Face::parse(EMOJI, 0).unwrap();
        for c in ['😀', '❤', '🐺', '🎉'] {
            let glyph = face.glyph_index(c).unwrap();
            let png = face.glyph_png(glyph, 128).expect("color PNG");
            assert!(png.starts_with(b"\x89PNG\r\n\x1a\n"));
            assert!(png.len() > 24);
            let width = u32::from_be_bytes(png[16..20].try_into().unwrap());
            let height = u32::from_be_bytes(png[20..24].try_into().unwrap());
            assert!((1..=512).contains(&width) && (1..=512).contains(&height));
            assert!(!face.has_outline(glyph), "bitmap-only glyph must not mask outline fallback");
        }
    }

    #[test]
    fn missing_glyphs_and_space_do_not_claim_visible_outlines() {
        let face = Face::parse(INTER, 0).unwrap();
        assert!(face.glyph_index('\u{10ffff}').is_none());
        assert!(!face.has_outline(face.glyph_index(' ').unwrap()));
        assert!(face.glyph_png(face.glyph_index('A').unwrap(), 128).is_none());
    }

    #[test]
    fn truncated_fonts_never_panic() {
        for len in 0..512 {
            assert!(Face::parse(&INTER[..len], 0).is_err());
            assert!(Face::parse(&EMOJI[..len], 0).is_err());
        }
        assert!(Face::parse(&[0xff; 1024], 0).is_err());
    }

    fn collection(fonts: &[&[u8]]) -> Vec<u8> {
        let mut out = b"ttcf\0\x01\0\0".to_vec();
        out.extend_from_slice(&(fonts.len() as u32).to_be_bytes());
        out.resize(12 + 4 * fonts.len(), 0);
        for (i, font) in fonts.iter().enumerate() {
            while !out.len().is_multiple_of(4) {
                out.push(0);
            }
            let offset = out.len();
            out[12 + i * 4..16 + i * 4].copy_from_slice(&(offset as u32).to_be_bytes());
            out.extend_from_slice(font);
            let tables = u16::from_be_bytes([font[4], font[5]]) as usize;
            for table in 0..tables {
                let pos = offset + 12 + table * 16 + 8;
                let old = u32::from_be_bytes(out[pos..pos + 4].try_into().unwrap());
                out[pos..pos + 4].copy_from_slice(&(old + offset as u32).to_be_bytes());
            }
        }
        out
    }

    #[test]
    fn collections_keep_the_selected_face() {
        let bytes = collection(&[INTER, BOLD]);
        assert_eq!(fonts_in_collection(&bytes), Some(2));
        assert_eq!(Face::parse(&bytes, 0).unwrap().weight(), 400);
        assert_eq!(Face::parse(&bytes, 1).unwrap().weight(), 600);
        assert!(Face::parse(&bytes, 2).is_err());
        assert_eq!(fonts_in_collection(INTER), None);
    }
}
