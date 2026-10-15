use plervo_suite_core::{AppGroup, CraftApp};
const fn app(
    id: &'static str,
    name: &'static str,
    tagline: &'static str,
    group: AppGroup,
    [color, ink_light, ink_dark]: [[u8; 3]; 3],
    repo: &'static str,
) -> CraftApp {
    CraftApp { id, name, tagline, group, color, ink_light, ink_dark, repo, former_ids: &[], former_names: &[] }
}

/// The Craft App Suite, in display order: the seven classic apps, then the six new ones.
pub const SUITE: [CraftApp; 13] = [
    app(
        "photocraft",
        "PhotoCraft",
        "Image editing",
        AppGroup::Classic,
        [[0x2f, 0x7b, 0xf5], [0x1a, 0x5b, 0xc4], [0x7f, 0xb0, 0xff]],
        "storytold/photocraft",
    ),
    app(
        "vectorcraft",
        "VectorCraft",
        "Vector illustration",
        AppGroup::Classic,
        [[0xe8, 0x57, 0x3f], [0xb8, 0x3a, 0x24], [0xff, 0x8a, 0x70]],
        "storytold/vectorcraft",
    ),
    app(
        "filmcraft",
        "FilmCraft",
        "Video editing, color and sound",
        AppGroup::Classic,
        [[0x8b, 0x5c, 0xf6], [0x6a, 0x3f, 0xd6], [0xb6, 0x9c, 0xff]],
        "storytold/filmcraft",
    ),
    app(
        "lightcraft",
        "LightCraft",
        "Photo library and raw development",
        AppGroup::Classic,
        [[0xf2, 0xa5, 0x16], [0x8a, 0x58, 0x00], [0xff, 0xc1, 0x4d]],
        "storytold/lightcraft",
    ),
    CraftApp {
        former_ids: &["printcraft"],
        former_names: &["PrintCraft"],
        ..app(
            "pdfcraft",
            "PdfCraft",
            "Reading, organizing and protecting PDFs",
            AppGroup::Classic,
            [[0x12, 0xa5, 0x8a], [0x0a, 0x75, 0x63], [0x3f, 0xd6, 0xb8]],
            "storytold/pdfcraft",
        )
    },
    app(
        "effectcraft",
        "EffectCraft",
        "Motion graphics and visual effects",
        AppGroup::Classic,
        [[0xe0, 0x36, 0x8f], [0xb0, 0x20, 0x6c], [0xff, 0x7a, 0xb8]],
        "storytold/effectcraft",
    ),
    app(
        "designcraft",
        "DesignCraft",
        "Page layout and publishing",
        AppGroup::Classic,
        [[0x7b, 0xb5, 0x1c], [0x45, 0x70, 0x08], [0xa6, 0xe0, 0x4a]],
        "storytold/designcraft",
    ),
    app(
        "wordcraft",
        "WordCraft",
        "Writing and document design",
        AppGroup::New,
        [[0x3f, 0x5f, 0xe0], [0x2a, 0x44, 0xb8], [0x8e, 0xa2, 0xff]],
        "storytold/wordcraft",
    ),
    app(
        "deckcraft",
        "DeckCraft",
        "Presentations and slide shows",
        AppGroup::New,
        [[0xf0, 0x7a, 0x1e], [0xa8, 0x4f, 0x0a], [0xff, 0xa0, 0x60]],
        "storytold/deckcraft",
    ),
    app(
        "gridcraft",
        "GridCraft",
        "Spreadsheets and data",
        AppGroup::New,
        [[0x1f, 0x9d, 0x55], [0x12, 0x70, 0x3a], [0x4f, 0xd6, 0x8a]],
        "storytold/gridcraft",
    ),
    app(
        "cadcraft",
        "CADCraft",
        "Computer-aided design and drafting",
        AppGroup::New,
        [[0x3e, 0xa6, 0xe8], [0x1a, 0x6f, 0xa8], [0x7c, 0xc8, 0xff]],
        "storytold/cadcraft",
    ),
    app(
        "soundcraft",
        "SoundCraft",
        "Recording, editing and mixing audio",
        AppGroup::New,
        [[0x1f, 0xa5, 0xc9], [0x0e, 0x72, 0x90], [0x5f, 0xd0, 0xee]],
        "storytold/soundcraft",
    ),
    app(
        "figcraft",
        "FigCraft",
        "Interface design and prototyping",
        AppGroup::New,
        [[0x02, 0xda, 0x96], [0x04, 0x7a, 0x56], [0x4f, 0xe8, 0xb6]],
        "storytold/figcraft",
    ),
];

pub fn by_id(id: &str) -> Option<&'static CraftApp> {
    SUITE.iter().find(|a| a.id == id)
}
