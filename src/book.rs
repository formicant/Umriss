use std::collections::{HashMap, HashSet};
use euclid::default::{Point2D, Size2D};
use itertools::Itertools;
use crate::image_contour_collection::ImageContourCollection;
use crate::glyph::Glyph;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GlyphKind {
    Unique,
    PageShared,
    BookShared,
}

/// Represents an occurrence of a glyph on a page.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct GlyphEntry<'a> {
    location: Point2D<i32>,
    kind: GlyphKind,
    id: usize,
    glyph: &'a Glyph,
}

impl<'a> GlyphEntry<'a> {
    /// Coordinates of the upper-left corner
    /// of the glyph’s bounding box in the page.
    pub fn location(&self) -> Point2D<i32> { self.location }
    pub fn kind(&self) -> GlyphKind { self.kind }
    pub fn id(&self) -> usize { self.id }
    pub fn glyph(&self) -> &'a Glyph { self.glyph }
}

#[derive(Debug)]
pub struct SharedGlyph<'a> {
    id: usize,
    occurrence_count: usize,
    glyph: &'a Glyph,
}

impl<'a> SharedGlyph<'a> {
    pub fn id(&self) -> usize { self.id }
    pub fn occurrence_count(&self) -> usize { self.occurrence_count }
    pub fn glyph(&self) -> &'a Glyph { self.glyph }
}

#[derive(Debug)]
pub struct Page<'a> {
    content: &'a PageContent,
    glyphs: &'a[Glyph],
    book_dictionary: &'a HashMap<usize, usize>,
}

impl<'a> Page<'a> {
    /// Page width and height.
    pub fn size(&self) -> Size2D<i32> { self.content.size }
      
    /// Glyphs that appear only on this page and more than once.
    pub fn shared_glyphs(&'a self) -> impl Iterator<Item = SharedGlyph<'a>> {
        self.content.dictionary.iter()
            .map(|(&index, &occurrence_count)|
                SharedGlyph { id: index, occurrence_count, glyph: &self.glyphs[index] })
    }
    
    /// All glyph occurrences on the page.
    pub fn glyph_entries(&self) -> impl Iterator<Item = GlyphEntry<'a>> {
        self.content.glyph_entries.iter()
            .map(|&(location, index)| {
                let kind = if self.book_dictionary.contains_key(&index) {
                    GlyphKind::BookShared
                } else if self.content.dictionary.contains_key(&index) {
                    GlyphKind::PageShared
                } else {
                    GlyphKind::Unique
                };
                GlyphEntry { location, kind, id: index, glyph: &self.glyphs[index] }
            })
    }
}

#[derive(Debug)]
pub struct Book {
    pages: Vec<PageContent>,
    /// Glyphs that appear on more than one page.
    /// Key is the index in `glyphs`, value is occurrence count.
    dictionary: HashMap<usize, usize>,
    /// List of all glyphs of the book.
    glyphs: Vec<Glyph>,
}

impl Book {
    pub fn new(contour_collections: impl Iterator<Item = ImageContourCollection>) -> Self {
        let mut glyph_indices = HashMap::<Glyph, usize>::new();
        let mut distribution = Vec::<GlypDistribution>::new();
        let mut pages = Vec::new();
        let mut dictionary = HashMap::new();
        
        for contour_collection in contour_collections {
            let page_index = pages.len();
            let size = Size2D::from(contour_collection.dimensions()).to_i32();
            let mut glyph_entries = Vec::new();
            
            let page_glyphs = contour_collection.outer_contours().map(Glyph::from_contour);
            for (glyph, location) in page_glyphs {
                let glyph_index = *glyph_indices.entry(glyph).or_insert_with(|| {
                    let new_index = distribution.len();
                    distribution.push(Default::default());
                    new_index
                });
                distribution[glyph_index].add(page_index);
                glyph_entries.push((location, glyph_index));
            }
            pages.push(PageContent { size, dictionary: HashMap::new(), glyph_entries });
        }
        
        for (glyph_index, glyph_distribution) in distribution.iter().enumerate() {
            if glyph_distribution.pages.len() > 1 {
                dictionary.insert(glyph_index, glyph_distribution.count);
            } else if glyph_distribution.count > 1 {
                let &page_index = glyph_distribution.pages.iter().next().unwrap();
                pages[page_index].dictionary.insert(glyph_index, glyph_distribution.count);
            }
        }
        
        let glyphs: Vec<_> = glyph_indices.into_iter()
            .sorted_unstable_by_key(|&(_, index)| index)
            .map(|(glyph, _)| glyph)
            .collect();
        
        Self { pages, dictionary, glyphs }
    }
    
    pub fn pages<'a>(&'a self) -> impl Iterator<Item = Page<'a>> {
        self.pages.iter().map(|content|
            Page { content: &content, glyphs: &self.glyphs, book_dictionary: &self.dictionary })
    }
    
    /// Glyphs that appear on more than one page.
    pub fn shared_glyphs<'a>(&'a self) -> impl Iterator<Item = SharedGlyph> {
        self.dictionary.iter().map(|(&index, &occurrence_count)|
            SharedGlyph { id: index, glyph: &self.glyphs[index], occurrence_count })
    }
}

#[derive(Debug)]
struct PageContent {
    size: Size2D<i32>,
    /// Glyphs that appear only on this page and more than once.
    /// Key is the index in `Book::glyphs`, value is occurrence count.
    dictionary: HashMap<usize, usize>,
    /// Locations and `Book::glyphs` indices of all glyphs on the page.
    glyph_entries: Vec<(Point2D<i32>, usize)>,
}

/// Counts glyph occurrences in the book
/// and presence of the glyph on pages.
#[derive(Debug, Default)]
struct GlypDistribution{
    count: usize,
    pages: HashSet<usize>,
}

impl GlypDistribution {
    pub fn add(&mut self, page_index: usize) {
        self.pages.insert(page_index);
        self.count += 1;
    }
}
