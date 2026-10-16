//! The visible product family. Only the PDF has a qualified public installer.
pub struct Product {
    pub id: &'static str,
    pub name: &'static str,
    pub description: [&'static str; 3],
    pub icon: &'static [u8],
}

macro_rules! product {
    ($id:literal, $name:literal, $en:literal, $pt:literal, $es:literal) => {
        Product {
            id: $id,
            name: $name,
            description: [$en, $pt, $es],
            icon: include_bytes!(concat!("../../../assets/products/", $id, "/icon-128.png")),
        }
    };
}

pub const PRODUCTS: [Product; 13] = [
    product!("pdf", "Plervo PDF", "PDF documents", "Documentos PDF", "Documentos PDF"),
    product!("photo", "Plervo Photo", "Image editing", "Edição de imagens", "Edición de imágenes"),
    product!("vector", "Plervo Vector", "Vector illustration", "Ilustração vetorial", "Ilustración vectorial"),
    product!("video", "Plervo Video", "Video editing", "Edição de vídeo", "Edición de vídeo"),
    product!("develop", "Plervo Develop", "RAW photography", "Fotografia RAW", "Fotografía RAW"),
    product!("motion", "Plervo Motion", "Motion and effects", "Animação e efeitos", "Animación y efectos"),
    product!("publish", "Plervo Publish", "Page layout", "Diagramação", "Maquetación"),
    product!("write", "Plervo Write", "Writing", "Textos e documentos", "Textos y documentos"),
    product!("slides", "Plervo Slides", "Presentations", "Apresentações", "Presentaciones"),
    product!("sheets", "Plervo Sheets", "Spreadsheets", "Planilhas", "Hojas de cálculo"),
    product!("cad", "Plervo CAD", "Technical drawing", "Desenho técnico", "Dibujo técnico"),
    product!("audio", "Plervo Audio", "Audio editing", "Edição de áudio", "Edición de audio"),
    product!("canvas", "Plervo Canvas", "Interface design", "Design de interfaces", "Diseño de interfaces"),
];

pub const PDF_ICON: &[u8] = include_bytes!("../../../assets/products/pdf/icon-128.png");
