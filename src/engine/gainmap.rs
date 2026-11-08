//! HDR gain-map JPEGs (Ultra HDR / Adobe gain map, with ISO 21496-1).
//!
//! The file is the SDR export unchanged, which every viewer shows, followed
//! by a quarter-size greyscale JPEG of the log2 gain that takes it to its HDR
//! rendition (`Renderer::hdr_gain`). HDR displays apply the gain as far as
//! their headroom allows. The primary image carries the `hdrgm` XMP with the
//! container directory, the ISO 21496-1 version block and an MPF index
//! pointing at the gain map; the gain map carries the `hdrgm` XMP values and
//! the ISO 21496-1 metadata in binary.

/// Gain-map resolution: one sample per `SCALE` × `SCALE` pixels.
pub const SCALE: usize = 4;
/// Offsets added to SDR and HDR linear values before the ratio (both).
const OFFSET: f64 = 1.0 / 64.0;
/// Below this log2 gain (about 3 %) a frame has no highlights to lift.
const MIN_LOG2: f32 = 0.05;
/// Denominator of the ISO 21496-1 fractions.
const DENOM: u32 = 1_000_000;

const XMP_NS: &[u8] = b"http://ns.adobe.com/xap/1.0/\0";
const ISO_NS: &[u8] = b"urn:iso:std:iso:ts:21496:-1\0";

/// The gain map's range: log2 gains 0 … `max_log2`, linearly in 0 … 255.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct GainMeta {
    pub max_log2: f32,
}

/// The quarter-size log2 gain map of a `w` × `h` gain buffer, as 8-bit
/// values with its range; `None` when nothing in the frame is lifted.
pub fn encode_gain_map(gain: &[f32], w: usize, h: usize) -> Option<(Vec<u8>, usize, usize, GainMeta)> {
    let (gw, gh) = (w.div_ceil(SCALE), h.div_ceil(SCALE));
    let mut log = vec![0f32; gw * gh];
    for gy in 0..gh {
        for gx in 0..gw {
            let (mut sum, mut n) = (0f32, 0f32);
            for y in gy * SCALE..((gy + 1) * SCALE).min(h) {
                for x in gx * SCALE..((gx + 1) * SCALE).min(w) {
                    sum += gain[y * w + x].max(1.0).log2();
                    n += 1.0;
                }
            }
            log[gy * gw + gx] = sum / n;
        }
    }
    let max_log2 = log.iter().copied().fold(0.0, f32::max);
    if max_log2 < MIN_LOG2 {
        return None;
    }
    let q = log
        .iter()
        .map(|&v| (v / max_log2 * 255.0).round().clamp(0.0, 255.0) as u8)
        .collect();
    Some((q, gw, gh, GainMeta { max_log2 }))
}

/// One APPn segment.
fn segment(marker: u8, payload: &[u8]) -> Vec<u8> {
    let len = payload.len() + 2;
    assert!(len <= u16::MAX as usize, "JPEG segment too long");
    let mut v = vec![0xFF, marker, (len >> 8) as u8, len as u8];
    v.extend_from_slice(payload);
    v
}

fn ns_segment(marker: u8, ns: &[u8], body: &[u8]) -> Vec<u8> {
    segment(marker, &[ns, body].concat())
}

/// Where the leading APPn segments of a JPEG end (after SOI, JFIF, ICC,
/// EXIF): metadata goes there, before the tables.
fn after_app_segments(jpeg: &[u8]) -> usize {
    let mut i = 2;
    while i + 4 <= jpeg.len() && jpeg[i] == 0xFF && (0xE0..=0xEF).contains(&jpeg[i + 1]) {
        i += 2 + u16::from_be_bytes([jpeg[i + 2], jpeg[i + 3]]) as usize;
    }
    i
}

fn insert(jpeg: &[u8], at: usize, segs: &[&[u8]]) -> Vec<u8> {
    let mut v = Vec::with_capacity(jpeg.len() + segs.iter().map(|s| s.len()).sum::<usize>());
    v.extend_from_slice(&jpeg[..at]);
    for s in segs {
        v.extend_from_slice(s);
    }
    v.extend_from_slice(&jpeg[at..]);
    v
}

fn xmp_primary(gain_len: usize) -> String {
    format!(
        r#"<x:xmpmeta xmlns:x="adobe:ns:meta/" x:xmptk="NLP Studio">
 <rdf:RDF xmlns:rdf="http://www.w3.org/1999/02/22-rdf-syntax-ns#">
  <rdf:Description rdf:about=""
    xmlns:Container="http://ns.google.com/photos/1.0/container/"
    xmlns:Item="http://ns.google.com/photos/1.0/container/item/"
    xmlns:hdrgm="http://ns.adobe.com/hdr-gain-map/1.0/"
   hdrgm:Version="1.0">
   <Container:Directory>
    <rdf:Seq>
     <rdf:li rdf:parseType="Resource">
      <Container:Item Item:Semantic="Primary" Item:Mime="image/jpeg"/>
     </rdf:li>
     <rdf:li rdf:parseType="Resource">
      <Container:Item Item:Semantic="GainMap" Item:Mime="image/jpeg" Item:Length="{gain_len}"/>
     </rdf:li>
    </rdf:Seq>
   </Container:Directory>
  </rdf:Description>
 </rdf:RDF>
</x:xmpmeta>"#
    )
}

fn xmp_gain(m: GainMeta) -> String {
    format!(
        r#"<x:xmpmeta xmlns:x="adobe:ns:meta/" x:xmptk="NLP Studio">
 <rdf:RDF xmlns:rdf="http://www.w3.org/1999/02/22-rdf-syntax-ns#">
  <rdf:Description rdf:about=""
    xmlns:hdrgm="http://ns.adobe.com/hdr-gain-map/1.0/"
   hdrgm:Version="1.0"
   hdrgm:GainMapMin="0"
   hdrgm:GainMapMax="{max:.6}"
   hdrgm:Gamma="1"
   hdrgm:OffsetSDR="{OFFSET:.6}"
   hdrgm:OffsetHDR="{OFFSET:.6}"
   hdrgm:HDRCapacityMin="0"
   hdrgm:HDRCapacityMax="{max:.6}"
   hdrgm:BaseRenditionIsHDR="False"/>
 </rdf:RDF>
</x:xmpmeta>"#,
        max = m.max_log2
    )
}

/// ISO 21496-1 metadata: versions only in the primary image, the full
/// single-channel record (common denominator) in the gain map.
fn iso21496(m: Option<GainMeta>) -> Vec<u8> {
    let mut v = vec![0, 0, 0, 0]; // minimum and writer version 0
    let Some(m) = m else {
        return v;
    };
    let n = |x: f64| (x * DENOM as f64).round() as i32;
    // The gain in the base image's colour space; each value its own
    // numerator and denominator (Apple's ImageIO reads only this form).
    v.push(0x40);
    let mut frac = |num: i32| {
        v.extend_from_slice(&num.to_be_bytes());
        v.extend_from_slice(&DENOM.to_be_bytes());
    };
    frac(0); // base headroom
    frac(n(m.max_log2 as f64)); // alternate headroom
    frac(0); // gain map min
    frac(n(m.max_log2 as f64)); // gain map max
    frac(DENOM as i32); // gamma 1
    frac(n(OFFSET)); // base offset
    frac(n(OFFSET)); // alternate offset
    v
}

/// The MPF APP2 payload: two images, the gain map `offset` bytes after the
/// MPF header's TIFF start.
fn mpf(primary_len: usize, gain_len: usize, offset: usize) -> Vec<u8> {
    let mut v = b"MPF\0MM\0\x2A".to_vec();
    v.extend_from_slice(&8u32.to_be_bytes());
    v.extend_from_slice(&3u16.to_be_bytes());
    let entry = |v: &mut Vec<u8>, tag: u16, ty: u16, count: u32, value: [u8; 4]| {
        v.extend_from_slice(&tag.to_be_bytes());
        v.extend_from_slice(&ty.to_be_bytes());
        v.extend_from_slice(&count.to_be_bytes());
        v.extend_from_slice(&value);
    };
    entry(&mut v, 0xB000, 7, 4, *b"0100");
    entry(&mut v, 0xB001, 4, 1, 2u32.to_be_bytes());
    // MP entries follow the IFD: 8 + 2 + 3·12 + 4.
    entry(&mut v, 0xB002, 7, 32, 50u32.to_be_bytes());
    v.extend_from_slice(&0u32.to_be_bytes());
    for (attr, size, off) in [(0x0003_0000u32, primary_len, 0), (0, gain_len, offset)] {
        v.extend_from_slice(&attr.to_be_bytes());
        v.extend_from_slice(&(size as u32).to_be_bytes());
        v.extend_from_slice(&(off as u32).to_be_bytes());
        v.extend_from_slice(&[0; 4]);
    }
    v
}

/// The Ultra HDR file of an encoded SDR JPEG and its encoded gain map.
pub fn assemble(primary: &[u8], gain: &[u8], meta: GainMeta) -> Vec<u8> {
    let gain = insert(
        gain,
        after_app_segments(gain),
        &[
            &ns_segment(0xE1, XMP_NS, xmp_gain(meta).as_bytes()),
            &ns_segment(0xE2, ISO_NS, &iso21496(Some(meta))),
        ],
    );
    let xmp = ns_segment(0xE1, XMP_NS, xmp_primary(gain.len()).as_bytes());
    let iso = ns_segment(0xE2, ISO_NS, &iso21496(None));
    let mpf_len = segment(0xE2, &mpf(0, 0, 0)).len();
    let at = after_app_segments(primary);
    let primary_len = primary.len() + xmp.len() + iso.len() + mpf_len;
    // The MPF offsets count from its TIFF header, after FF E2, length and "MPF\0".
    let tiff = at + xmp.len() + iso.len() + 8;
    let mpf = segment(0xE2, &mpf(primary_len, gain.len(), primary_len - tiff));
    let mut out = insert(primary, at, &[&xmp, &iso, &mpf]);
    debug_assert_eq!(out.len(), primary_len);
    out.extend_from_slice(&gain);
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use image::ImageEncoder;

    fn jpeg(w: u32, h: u32, ct: image::ExtendedColorType, data: &[u8]) -> Vec<u8> {
        let mut v = Vec::new();
        let mut enc = image::codecs::jpeg::JpegEncoder::new_with_quality(&mut v, 90);
        enc.set_icc_profile(vec![0; 64]).unwrap();
        enc.write_image(data, w, h, ct).unwrap();
        v
    }

    fn find(hay: &[u8], needle: &[u8]) -> Option<usize> {
        hay.windows(needle.len()).position(|w| w == needle)
    }

    #[test]
    fn flat_gain_writes_no_map() {
        assert!(encode_gain_map(&[1.0; 64], 8, 8).is_none());
    }

    #[test]
    fn gain_map_is_quarter_size_log2() {
        let mut g = vec![1.0f32; 10 * 6];
        g[9] = 4.0; // top right block: one pixel at +2 stops
        let (q, w, h, m) = encode_gain_map(&g, 10, 6).unwrap();
        assert_eq!((w, h), (3, 2));
        assert!((m.max_log2 - 2.0 / 8.0).abs() < 1e-6, "{m:?}"); // averaged over 2×4
        assert_eq!(q, vec![0, 0, 255, 0, 0, 0]);
    }

    /// The assembled file is the SDR JPEG with metadata, and its MPF index
    /// leads to a gain map that decodes at the stored size.
    #[test]
    fn assembled_file_parses_back() {
        let sdr: Vec<u8> = (0..32 * 16 * 3).map(|i| (i % 251) as u8).collect();
        let primary = jpeg(32, 16, image::ExtendedColorType::Rgb8, &sdr);
        let gain = jpeg(8, 4, image::ExtendedColorType::L8, &[128; 32]);
        let meta = GainMeta { max_log2: 1.5 };
        let file = assemble(&primary, &gain, meta);

        let tiff = find(&file, b"MPF\0").unwrap() + 4;
        let at = |i: usize| u32::from_be_bytes(file[i..i + 4].try_into().unwrap()) as usize;
        let entries = tiff + 50;
        let (primary_len, gain_len, gain_off) = (at(entries + 4), at(entries + 20), at(entries + 24));
        assert_eq!(at(entries + 8), 0);
        assert_eq!(primary_len + gain_len, file.len());
        assert_eq!(tiff + gain_off, primary_len);
        assert_eq!(&file[primary_len..primary_len + 2], &[0xFF, 0xD8]);

        let a = image::load_from_memory(&file[..primary_len]).unwrap();
        assert_eq!((a.width(), a.height()), (32, 16));
        let b = image::load_from_memory(&file[primary_len..]).unwrap();
        assert_eq!((b.width(), b.height()), (8, 4));

        // The SDR image decodes exactly as it did without the gain map.
        let plain = image::load_from_memory(&primary).unwrap().to_rgb8();
        assert_eq!(a.to_rgb8().as_raw(), plain.as_raw());

        let head = String::from_utf8_lossy(&file[..primary_len]);
        assert!(head.contains(&format!("Item:Length=\"{gain_len}\"")));
        let tail = String::from_utf8_lossy(&file[primary_len..]);
        assert!(tail.contains("hdrgm:GainMapMax=\"1.500000\""));
        assert!(tail.contains("hdrgm:HDRCapacityMax=\"1.500000\""));
        let iso = primary_len + find(&file[primary_len..], ISO_NS).unwrap() + ISO_NS.len();
        assert_eq!(&file[iso..iso + 5], &[0, 0, 0, 0, 0x40]);
        assert_eq!((at(iso + 13), at(iso + 17)), (1_500_000, DENOM as usize)); // alternate headroom
    }
}
