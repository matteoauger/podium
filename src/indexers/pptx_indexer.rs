use super::Indexer;
use super::DocumentSchema;
use std::path::Path;
use std::ffi::OsStr;
use std::fs;

use msoffice_pptx::document::PPTXDocument;
use msoffice_pptx::pml::{ShapeGroup, Shape};

use msoffice_shared::drawingml::{TextBody,TextRun};

pub struct PptxIndexer;

impl Indexer for PptxIndexer {
    fn supports_extension(&self, extension: &OsStr) -> bool {
        extension == OsStr::new("pptx")
    }

    fn index_file(&self, path: &Path) -> DocumentSchema {
        let mut total_text = String::new();
        let document = PPTXDocument::from_file(path).unwrap();
  
        for (slide_path, slide) in &document.slide_map {
            let shape_group = &(*(*slide.common_slide_data).shape_tree).shape_array;
            for s_g in shape_group {
                if let Some(res_text) = extract_text(s_g) {
                    total_text.push_str(&res_text);
                }
            }
        }

        dbg!(&total_text);

        DocumentSchema {
            name: String::new(),
            body: total_text,
        }
    }
}

fn extract_text(shape_group: &ShapeGroup) -> Option<String> {
    let mut total_text = String::new();
    match shape_group {
        ShapeGroup::Shape(shape) => {
            if let Some(text_body) = &shape.text_body {
                for paragraph in &text_body.paragraph_array {
                    for text_run in &paragraph.text_run_list {
                        match text_run {
                            TextRun::RegularTextRun(regular_text_run) => {
                                total_text.push_str(&regular_text_run.text);
                                total_text.push_str(" ");
                            },
                            _ => {}
                        }
                    }
                }
            }
        },
        ShapeGroup::GroupShape(group_shape) => {
            let res_text = group_shape
                .shape_array
                .iter()
                .map(|s_g| extract_text(s_g))
                .filter_map(|r_t| r_t)
                .fold(String::new(), |mut acc, x| { acc.push_str(&x); acc.push_str(" "); acc });

            total_text.push_str(&res_text);
        },
        _ => {}
    }
    if total_text != String::new() {
        Some(total_text)
    }
    else {
        None
    }

}

#[cfg(test)]
mod tests {
    use super::*;
    use test::Bencher;

    #[test]
    fn test_indexing_pptx_file() {
        let test_file_path = Path::new("./test_files/Cats.pptx");
        let indexed_document = PptxIndexer.index_file(test_file_path);

        assert_eq!(indexed_document.name, "");
        assert!(indexed_document.body.contains("Cats"));
        assert!(indexed_document.body.contains("quick"));
        assert!(indexed_document.body.contains("story"));
        assert!(indexed_document.body.contains("Paws"));
        assert!(indexed_document.body.contains("cool"));
    }

    #[test]
    fn test_supports_pptgx_extension() {
        assert_eq!(true, PptxIndexer.supports_extension(OsStr::new("pptx")));
        assert_eq!(false, PptxIndexer.supports_extension(OsStr::new("ppt")));
    }

    #[bench]
    fn bench_indexing_pptx_file(b: &mut Bencher) {
        b.iter(|| {
            let bench_file_path = test::black_box(Path::new("./test_files/Cats.pptx"));
            PptxIndexer.index_file(bench_file_path)
        });
    }
}