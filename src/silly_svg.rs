use std::fs;
use itertools::Itertools;
use euclid::default::{Point2D, Vector2D, Size2D};
use crate::book::{Book, Page, GlyphKind};
use crate::glyph::Glyph;

pub fn write_book_as_multiple_svg_files(book: &Book) {
    let shared_contents = get_shared_contents(book);
    let pages_contents = book.pages().map(|page| get_page_contents(&page));
    
    if let Some(contents) = shared_contents {
        fs::write("output/_.svg", contents).unwrap();
    }
    
    for (i, page_contents) in pages_contents.enumerate() {
        fs::write(format!("output/{:03}.svg", i + 1), page_contents).unwrap();
    }
}

fn get_shared_contents(book: &Book) -> Option<String> {
    let mut glyph_definitions = book.shared_glyphs()
        .map(|g| get_path(Point2D::zero(), g.glyph(), Some(g.id())));
    let definition_lines = glyph_definitions.join("\n  ");
    if definition_lines.len() > 0 {
        Some(format!(r#"<svg version="1.1" xmlns="http://www.w3.org/2000/svg">
 <defs>
  {definition_lines}
 </defs>
</svg>"#))
    } else {
        None
    }
}

fn get_page_contents(page: &Page) -> String {
    let Size2D { width, height, .. } = page.size();
    let mut glyph_definitions = page.shared_glyphs()
        .map(|g| get_path(Point2D::zero(), g.glyph(), Some(g.id())));
    let mut glyphs = page.glyph_entries()
        .map(|entry| match entry.kind() {
            GlyphKind::Unique => get_path(entry.location(), &entry.glyph(), None),
            GlyphKind::PageShared => format!("<use x=\"{}\" y=\"{}\" href=\"#{}\" fill=\"red\"/>", entry.location().x, entry.location().y, entry.id()),
            GlyphKind::BookShared => format!("<use x=\"{}\" y=\"{}\" href=\"_.svg#{}\" fill=\"blue\"/>", entry.location().x, entry.location().y, entry.id()),
        });
    let definition_lines = glyph_definitions.join("\n  ");
    let defs = if definition_lines.len() > 0 {
        format!("\n <defs>\n  {definition_lines}\n </defs>")
    } else {
        String::new()
    };
    let glyph_lines = glyphs.join("\n ");
    format!(r#"<svg version="1.1" width="{width}" height="{height}" xmlns="http://www.w3.org/2000/svg">{defs}
 {glyph_lines}
</svg>"#)
}

fn get_path(location: Point2D<i32>, glyph: &Glyph, object_id: Option<usize>) -> String {
    let mut nodes = Vec::new();
    let mut previous_point = None;
    
    for contour in glyph.contours() {
        let mut start = None;
        for &point in contour.even_points() {
            if let Some(previous) = previous_point {
                let relative: Vector2D<i32> = point - previous;
                if let None = start {
                    start = Some(point);
                    nodes.push(format!("m{},{}", relative.x, relative.y));
                } else {
                    nodes.push(format!("h{}v{}", relative.x, relative.y));
                }
            } else {
                let absolute = location + point.to_vector();
                start = Some(point);
                nodes.push(format!("M{},{}", absolute.x, absolute.y));
            }
            previous_point = Some(point);
        }
        let relative = start.unwrap() - previous_point.unwrap();
        nodes.push(format!("h{}z", relative.x));
        previous_point = start;
    }
    
    let data = nodes.concat();
    let id_parameter = if let Some(id) = object_id { format!(r#"id="{id}" "#) } else { String::new() };
    format!(r#"<path {id_parameter}d="{data}"/>"#)
}
