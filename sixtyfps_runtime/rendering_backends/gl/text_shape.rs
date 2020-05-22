use font_kit::font::Font;
use harfbuzz::{Blob, Buffer};

pub fn shape_text(text: &str, font: &mut Font) -> Vec<u32> {
    let mut buffer = Buffer::with(text);
    buffer.guess_segment_properties();

    let font_data = font.copy_font_data().expect("need access to all font tables");
    let blob = Blob::new_from_arc_vec(font_data);

    let result = unsafe {
        let hb_face = harfbuzz::sys::hb_face_create(blob.as_raw(), 0);
        let hb_font = harfbuzz::sys::hb_font_create(hb_face);
        harfbuzz::sys::hb_shape(hb_font, buffer.as_ptr(), std::ptr::null(), 0);
        harfbuzz::sys::hb_font_destroy(hb_font);
        let mut glyph_count = 0;
        let glyph_infos =
            harfbuzz::sys::hb_buffer_get_glyph_infos(buffer.as_ptr(), &mut glyph_count);
        let glyph_infos = std::slice::from_raw_parts(glyph_infos, glyph_count as usize);
        let result = glyph_infos.iter().map(|glyph_info| glyph_info.codepoint).collect();

        harfbuzz::sys::hb_face_destroy(hb_face);

        result
    };

    result
}
