use std::io::{Read, Seek};
use std::path::Path;
use std::{fs, mem};

use postscript::compact::FontSet;
use truetype::{self, Fixed, HorizontalMetrics, Tag};

use Result;
use font::Font;

/// A font file.
pub struct File {
    /// Fonts.
    pub fonts: Vec<Font>,
}

macro_rules! checksum_and_jump(
    ($record:ident, $tape:ident, $process:expr) => ({
        if !try!($record.checksum($tape, $process)) {
            raise!("found a corrupted font table");
        }
        try!(truetype::Tape::jump($tape, $record.offset as u64));
    });
    ($record:ident, $tape:ident) => (
        checksum_and_jump!($record, $tape, |_, word| word);
    );
);

impl File {
    /// Open a file.
    #[inline]
    pub fn open<T: AsRef<Path>>(path: T) -> Result<File> {
        let mut file = try!(fs::File::open(path));
        File::read(&mut file)
    }

    /// Read a file.
    pub fn read<T: Read + Seek>(tape: &mut T) -> Result<File> {
        macro_rules! sort(
            ($records:expr) => ({
                let mut records = $records.iter().collect::<Vec<_>>();
                records.sort_by(|one, two| priority(Tag(one.tag)).cmp(&priority(Tag(two.tag))));
                records
            });
        );

        match try!(truetype::Tape::peek::<Fixed>(tape)) {
            Fixed(0x00010000) => {},
            version => {
                let tag = Tag::from(version);
                if tag == Tag::from(b"OTTO") {
                } else if tag == Tag::from(b"ttcf") {
                    raise!("TrueType collections are not supported yet");
                } else {
                    raise!("the font format is invalid");
                }
            },
        }

        let mut font = Font {
            offset_table: try!(truetype::Value::read(tape)),
            .. Font::default()
        };
        for record in sort!(font.offset_table.records) {
            macro_rules! set(
                ($field:ident, $value:expr) => ({
                    checksum_and_jump!(record, tape);
                    font.$field = Some(try!($value));
                });
                ($field:ident) => (set!($field, truetype::Value::read(tape)));
            );
            match &Tag(record.tag).into() {
                b"cmap" => set!(char_mapping),
                b"head" => {
                    checksum_and_jump!(record, tape, |i, word| if i == 2 { 0 } else { word });
                    font.font_header = Some(try!(truetype::Value::read(tape)));
                },
                b"hhea" => set!(horizontal_header),
                b"hmtx" => {
                    let header = match font.horizontal_header {
                        Some(ref table) => table,
                        _ => continue,
                    };
                    let profile = match font.maximum_profile {
                        Some(ref table) => table,
                        _ => continue,
                    };
                    set!(horizontal_metrics, HorizontalMetrics::read(tape, header, profile));
                },
                b"maxp" => set!(maximum_profile),
                b"name" => set!(naming_table),
                b"post" => set!(postscript_info),
                b"OS/2" => set!(windows_metrics),
                b"CFF " => set!(postscript_fontset, FontSet::read(tape)),
                _ => {},
            }
        }

        Ok(File { fonts: vec![font] })
    }
}

fn priority(tag: Tag) -> usize {
    use std::collections::HashMap;
    use std::sync::{Once, ONCE_INIT};

    unsafe {
        static mut PRIORITY: *const HashMap<Tag, usize> = 0 as *const _;
        static ONCE: Once = ONCE_INIT;
        ONCE.call_once(|| {
            let mut map: HashMap<Tag, usize> = HashMap::new();
            map.insert(Tag::from(b"hmtx"), 42);
            PRIORITY = mem::transmute(Box::new(map));
        });
        *(&*PRIORITY).get(&tag).unwrap_or(&0)
    }
}
