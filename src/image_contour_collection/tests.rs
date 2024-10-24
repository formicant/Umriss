use std::num::NonZeroUsize;
use test_case::test_case;
use crate::test_images::get_test_images;
use super::*;

#[test_case(
    1, 1, vec![0],
    vec![],
    vec![root(None)]
)]
#[test_case(
    1, 1, vec![1],
    vec![PointListItem { x: 0, y: 0, next: 1 }, PointListItem { x: 1, y: 1, next: 0 }],
    vec![root(NonZeroUsize::new(1)), hier(0, 0, None, None)]
)]
#[test_case(
    3, 4, vec![
        1, 1, 1,
        1, 0, 1,
        1, 0, 0,
        0, 0, 1,
    ],
    vec![
        PointListItem { x: 0, y: 0, next: 2 },
        PointListItem { x: 2, y: 1, next: 3 },
        PointListItem { x: 3, y: 2, next: 1 },
        PointListItem { x: 1, y: 3, next: 0 },
        PointListItem { x: 2, y: 3, next: 5 },
        PointListItem { x: 3, y: 4, next: 4 },
    ],
    vec![
        root(NonZeroUsize::new(2)),
        hier(0, 0, None, None),
        hier(4, 0, NonZeroUsize::new(1), None),
    ]
)]
fn small_test_images(
    width: u32, height: u32, image_pixels: Vec<u8>,
    expected_point_list: Vec<PointListItem>, expected_hierarchy: Vec<HierarchyItem>
) {
    let image = GrayImage::from_vec(width, height, image_pixels).unwrap();
    let actual = ImageContourCollection::new(&image, false);
    assert_eq!(actual.dimensions(), (width, height));
    assert_eq!(actual.point_list, expected_point_list);
    assert_eq!(actual.hierarchy, expected_hierarchy);
}

#[test]
fn point_list_consistency() {
    test_all_images(|name, contour_collection| {
        let list = contour_collection.point_list;
        assert!(list.len() >= 2, "Image: '{}'", name);
    })
}

fn test_all_images(test: impl Fn(&str, ImageContourCollection)) {
    for (name, image) in get_test_images() {
        for &inverted in [false, true].iter() {
            let contour_collection = ImageContourCollection::new(&image, inverted);
            test(&name, contour_collection);
        }
    }
}

const fn root(first_child: Option<NonZeroUsize>) -> HierarchyItem {
    HierarchyItem { head_point: hierarchy_builder::NONE, parent: hierarchy_builder::NONE, next_sibling: None, first_child }
}

const fn hier(head_point: usize, parent: usize, next_sibling: Option<NonZeroUsize>, first_child: Option<NonZeroUsize>) -> HierarchyItem {
    HierarchyItem { head_point, parent, next_sibling, first_child }
}
