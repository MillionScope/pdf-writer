use super::*;

/// A stream of text operations.
pub struct TextStream {
    buf: Vec<u8>,
}

impl TextStream {
    /// Create a new, empty text stream.
    pub fn new() -> Self {
        let mut buf = Vec::new();
        buf.push_bytes(b"BT\n");
        Self { buf }
    }

    /// `Tf` operator: Select a font by name and set the font size as a scale factor.
    pub fn tf(mut self, font: Name, size: f32) -> Self {
        self.buf.push_val(font);
        self.buf.push(b' ');
        self.buf.push_val(size);
        self.buf.push_bytes(b" Tf\n");
        self
    }

    /// `Td` operator: Move to the start of the next line.
    pub fn td(mut self, x: f32, y: f32) -> Self {
        self.buf.push_val(x);
        self.buf.push(b' ');
        self.buf.push_val(y);
        self.buf.push_bytes(b" Td\n");
        self
    }

    /// `Tm` operator: Set the text matrix.
    pub fn tm(mut self, a: f32, b: f32, c: f32, d: f32, e: f32, f: f32) -> Self {
        self.buf.push_val(a);
        self.buf.push(b' ');
        self.buf.push_val(b);
        self.buf.push(b' ');
        self.buf.push_val(c);
        self.buf.push(b' ');
        self.buf.push_val(d);
        self.buf.push(b' ');
        self.buf.push_val(e);
        self.buf.push(b' ');
        self.buf.push_val(f);
        self.buf.push_bytes(b" Tm\n");
        self
    }

    /// `Tj` operator: Write text.
    ///
    /// This function takes raw bytes. The encoding is up to the caller.
    pub fn tj(mut self, text: &[u8]) -> Self {
        // TODO: Move to general string formatting.
        self.buf.push(b'<');
        for &byte in text {
            self.buf.push_hex(byte);
        }
        self.buf.push_bytes(b"> Tj\n");
        self
    }

    /// Return the raw constructed byte stream.
    pub fn end(mut self) -> Vec<u8> {
        self.buf.push_bytes(b"ET");
        self.buf
    }
}

/// Writer for a _Type-1 font_.
pub struct Type1Font<'a> {
    dict: Dict<'a, IndirectGuard>,
}

impl<'a> Type1Font<'a> {
    pub(crate) fn start(any: Any<'a, IndirectGuard>) -> Self {
        let mut dict = any.dict();
        dict.pair(Name(b"Type"), Name(b"Font"));
        dict.pair(Name(b"Subtype"), Name(b"Type1"));
        Self { dict }
    }

    /// Write the `/BaseFont` attribute.
    pub fn base_font(&mut self, name: Name) -> &mut Self {
        self.dict.pair(Name(b"BaseFont"), name);
        self
    }
}

/// Writer for a _Type-0 (composite) font_.
pub struct Type0Font<'a> {
    dict: Dict<'a, IndirectGuard>,
}

impl<'a> Type0Font<'a> {
    pub(crate) fn start(any: Any<'a, IndirectGuard>) -> Self {
        let mut dict = any.dict();
        dict.pair(Name(b"Type"), Name(b"Font"));
        dict.pair(Name(b"Subtype"), Name(b"Type0"));
        Self { dict }
    }

    /// Write the `/BaseFont` attribute.
    pub fn base_font(&mut self, name: Name) -> &mut Self {
        self.dict.pair(Name(b"BaseFont"), name);
        self
    }

    /// Write the `/Encoding` attribute as a predefined encoding.
    pub fn encoding_predefined(&mut self, encoding: Name) -> &mut Self {
        self.dict.pair(Name(b"Encoding"), encoding);
        self
    }

    /// Write the `/Encoding` attribute as a reference to a [character map stream].
    ///
    /// [character map stream]: ../struct.PdfWriter.html#method.char_map
    pub fn encoding_cmap(&mut self, cmap: Ref) -> &mut Self {
        self.dict.pair(Name(b"Encoding"), cmap);
        self
    }

    /// Write the `/DescendantFonts` attribute as a one-element array containing a
    /// reference to a [CID font].
    ///
    /// [CID font]: struct.CIDFont.html
    pub fn descendant_font(&mut self, cid_font: Ref) -> &mut Self {
        self.dict.key(Name(b"DescendantFonts")).array().item(cid_font);
        self
    }

    /// Write the `/ToUnicode` attribute as a reference to a [character map stream].
    ///
    /// [character map stream]: ../struct.PdfWriter.html#method.char_map
    pub fn to_unicode(&mut self, cmap: Ref) -> &mut Self {
        self.dict.pair(Name(b"ToUnicode"), cmap);
        self
    }
}

/// Writer for a _CID font_, a descendant of a [Type 0 font].
///
/// [Type 0 font]: struct.Type0Font.html
pub struct CIDFont<'a> {
    dict: Dict<'a, IndirectGuard>,
}

impl<'a> CIDFont<'a> {
    pub(crate) fn start(any: Any<'a, IndirectGuard>, subtype: CIDFontType) -> Self {
        let mut dict = any.dict();
        dict.pair(Name(b"Type"), Name(b"Font"));
        dict.pair(Name(b"Subtype"), subtype.name());
        Self { dict }
    }

    /// Write the `/BaseFont` attribute.
    pub fn base_font(&mut self, name: Name) -> &mut Self {
        self.dict.pair(Name(b"BaseFont"), name);
        self
    }

    /// Write the `/CIDSystemInfo` dictionary.
    pub fn system_info(&mut self, info: SystemInfo) -> &mut Self {
        info.write(self.dict.key(Name(b"CIDSystemInfo")));
        self
    }

    /// Write the `/FontDescriptor` attribute as a reference to a font descriptor.
    pub fn font_descriptor(&mut self, cid_font: Ref) -> &mut Self {
        self.dict.pair(Name(b"FontDescriptor"), cid_font);
        self
    }

    /// Start writing the `/W` (widths) array.
    pub fn widths(&mut self) -> Widths<'_> {
        Widths::start(self.dict.key(Name(b"W")))
    }
}

/// Writer for the _width array_ in a [CID font].
///
/// [CID font]: struct.CIDFont.html
pub struct Widths<'a> {
    array: Array<'a>,
}

impl<'a> Widths<'a> {
    pub(crate) fn start(any: Any<'a>) -> Self {
        Self { array: any.array() }
    }

    /// Specifies individual widths for a range of CIDs starting at `start`.
    pub fn individual(
        &mut self,
        start: u16,
        widths: impl IntoIterator<Item = f32>,
    ) -> &mut Self {
        self.array.item(i32::from(start));
        self.array.any().array().typed().items(widths);
        self
    }

    /// Specifies the same width for all CIDs in the (inclusive) range from `first` to
    /// `last`.
    pub fn same(&mut self, first: u16, last: u16, width: f32) -> &mut Self {
        self.array.item(i32::from(first));
        self.array.item(i32::from(last));
        self.array.item(width);
        self
    }
}

/// Writer for a _font descriptor_.
///
/// [Type 0 font]: struct.Type0Font.html
pub struct FontDescriptor<'a> {
    dict: Dict<'a, IndirectGuard>,
}

impl<'a> FontDescriptor<'a> {
    pub(crate) fn start(any: Any<'a, IndirectGuard>) -> Self {
        let mut dict = any.dict();
        dict.pair(Name(b"Type"), Name(b"FontDescriptor"));
        Self { dict }
    }

    /// Write the `/FontName` attribute.
    pub fn font_name(&mut self, name: Name) -> &mut Self {
        self.dict.pair(Name(b"FontName"), name);
        self
    }

    /// Write the `/Flags` attribute.
    pub fn font_flags(&mut self, flags: FontFlags) -> &mut Self {
        self.dict.pair(Name(b"Flags"), flags.bits() as i32);
        self
    }

    /// Write the `/FontBBox` attribute.
    pub fn font_bbox(&mut self, bbox: Rect) -> &mut Self {
        self.dict.pair(Name(b"FontBBox"), bbox);
        self
    }

    /// Write the `/ItalicAngle` attribute.
    pub fn italic_angle(&mut self, angle: f32) -> &mut Self {
        self.dict.pair(Name(b"ItalicAngle"), angle);
        self
    }

    /// Write the `/Ascent` attribute.
    pub fn ascent(&mut self, ascent: f32) -> &mut Self {
        self.dict.pair(Name(b"Ascent"), ascent);
        self
    }

    /// Write the `/Descent` attribute.
    pub fn descent(&mut self, descent: f32) -> &mut Self {
        self.dict.pair(Name(b"Descent"), descent);
        self
    }

    /// Write the `/CapHeight` attribute.
    pub fn cap_height(&mut self, cap_height: f32) -> &mut Self {
        self.dict.pair(Name(b"CapHeight"), cap_height);
        self
    }

    /// Write the `/StemV` attribute.
    pub fn stem_v(&mut self, stem_v: f32) -> &mut Self {
        self.dict.pair(Name(b"StemV"), stem_v);
        self
    }

    /// Write the `/FontFile2` attribute as a reference to a stream containing a TrueType
    /// font program.
    pub fn font_file2(&mut self, true_type_stream: Ref) -> &mut Self {
        self.dict.pair(Name(b"FontFile2"), true_type_stream);
        self
    }
}

/// The subtype of a CID font.
#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
pub enum CIDFontType {
    /// A CID font containing CFF glyph descriptions.
    Type0,
    /// A CID font containing TrueType glyph descriptions.
    Type2,
}

impl CIDFontType {
    fn name(self) -> Name<'static> {
        match self {
            Self::Type0 => Name(b"CIDFontType0"),
            Self::Type2 => Name(b"CIDFontType2"),
        }
    }
}

pub use flags::*;

#[allow(missing_docs)]
mod flags {
    bitflags::bitflags! {
        /// Bitflags describing various characteristics of fonts.
        pub struct FontFlags: u32 {
            const FIXED_PITCH = 1 << 0;
            const SERIF = 1 << 1;
            const SYMBOLIC = 1 << 2;
            const SCRIPT = 1 << 3;
            const NON_SYMBOLIC = 1 << 5;
            const ITALIC = 1 << 6;
            const ALL_CAP = 1 << 16;
            const SMALL_CAP = 1 << 17;
            const FORCE_BOLD = 1 << 18;
        }
    }
}

/// Specifics about a character collection.
#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
pub struct SystemInfo<'a> {
    /// The issuer of the collection.
    pub registry: Str<'a>,
    /// A unique name of the collection within the registry.
    pub ordering: Str<'a>,
    /// The supplement number (i.e. the version).
    pub supplement: i32,
}

impl SystemInfo<'_> {
    fn write(&self, any: Any<'_>) {
        any.dict()
            .pair(Name(b"Registry"), self.registry)
            .pair(Name(b"Ordering"), self.ordering)
            .pair(Name(b"Supplement"), self.supplement);
    }
}

/// Writer a character map object.
///
/// Defined here:
/// https://www.adobe.com/content/dam/acom/en/devnet/font/pdfs/5014.CIDFont_Spec.pdf
pub(crate) fn write_cmap(
    w: &mut PdfWriter,
    id: Ref,
    name: Name,
    info: SystemInfo,
    mapping: impl ExactSizeIterator<Item = (u16, char)>,
) {
    let mut buf = Vec::new();

    // Static header.
    buf.push_bytes(b"%!PS-Adobe-3.0 Resource-CMap\n");
    buf.push_bytes(b"%%DocumentNeededResources: procset CIDInit\n");
    buf.push_bytes(b"%%IncludeResource: procset CIDInit\n");

    // Dynamic header.
    buf.push_bytes(b"%%BeginResource: CMap ");
    buf.push_bytes(name.0);
    buf.push(b'\n');
    buf.push_bytes(b"%%Title: (");
    buf.push_bytes(name.0);
    buf.push(b' ');
    buf.push_bytes(info.registry.0);
    buf.push(b' ');
    buf.push_bytes(info.ordering.0);
    buf.push(b' ');
    buf.push_int(info.supplement);
    buf.push_bytes(b")\n");
    buf.push_bytes(b"%%Version: 1\n");
    buf.push_bytes(b"%%EndComments\n");

    // General body.
    buf.push_bytes(b"/CIDInit /ProcSet findresource begin\n");
    buf.push_bytes(b"9 dict begin\n");
    buf.push_bytes(b"begincmap\n");
    buf.push_bytes(b"/CIDSystemInfo 3 dict dup begin\n");
    buf.push_bytes(b"    /Registry ");
    buf.push_val(info.registry);
    buf.push_bytes(b" def\n");
    buf.push_bytes(b"    /Ordering ");
    buf.push_val(info.ordering);
    buf.push_bytes(b" def\n");
    buf.push_bytes(b"    /Supplement ");
    buf.push_val(info.supplement);
    buf.push_bytes(b" def\n");
    buf.push_bytes(b"end def\n");
    buf.push_bytes(b"/CMapName ");
    buf.push_val(name);
    buf.push_bytes(b" def\n");
    buf.push_bytes(b"/CMapVersion 1 def\n");
    buf.push_bytes(b"/CMapType 0 def\n");

    // We just cover the whole unicode codespace.
    buf.push_bytes(b"1 begincodespacerange\n");
    buf.push_bytes(b"<0000> <ffff>\n");
    buf.push_bytes(b"endcodespacerange\n");

    // The mappings.
    buf.push_int(mapping.len());
    buf.push_bytes(b" beginbfchar\n");

    for (cid, c) in mapping {
        buf.push(b'<');
        buf.push_hex_u16(cid);
        buf.push_bytes(b"> <");

        let mut utf16 = [0u16; 2];
        for &mut part in c.encode_utf16(&mut utf16) {
            buf.push_hex_u16(part);
        }

        buf.push_bytes(b">\n");
    }
    buf.push_bytes(b"endbfchar\n");

    // End of body.
    buf.push_bytes(b"endcmap\n");
    buf.push_bytes(b"CMapName currentdict /CMap defineresource pop\n");
    buf.push_bytes(b"end\n");
    buf.push_bytes(b"end\n");
    buf.push_bytes(b"%%EndResource\n");
    buf.push_bytes(b"%%EOF");

    let mut dict = w.stream(id, &buf);
    dict.pair(Name(b"Type"), Name(b"CMap"));
    dict.pair(Name(b"CMapName"), name);
    info.write(dict.key(Name(b"CIDSystemInfo")));
}
