use std::num::NonZeroUsize;
use image::Luma;
use itertools::Itertools;
use test_case::test_case;
use crate::test_images::get_test_images;
use crate::geometry::{draw_orthopolygons, Orthopolygonlike, PointPosition, Polygonlike};
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
        /* 0 */ PointListItem { x: 0, y: 0, next: 2 },
        /* 1 */ PointListItem { x: 2, y: 1, next: 3 },
        /* 2 */ PointListItem { x: 3, y: 2, next: 1 },
        /* 3 */ PointListItem { x: 1, y: 3, next: 0 },
        /* 4 */ PointListItem { x: 2, y: 3, next: 5 },
        /* 5 */ PointListItem { x: 3, y: 4, next: 4 },
    ],
    vec![
        /* 0 */ root(NonZeroUsize::new(1)),
        /* 1 */ hier(0, 0, NonZeroUsize::new(2), None),
        /* 2 */ hier(4, 0, None, None),
    ]
)]
#[test_case(
    5, 4, vec![
        1, 1, 1, 1, 1,
        1, 0, 0, 0, 1,
        1, 0, 1, 0, 1,
        1, 0, 0, 0, 1,
    ],
    vec![
        /* 0 */ PointListItem { x: 0, y: 0, next: 5 },
        /* 1 */ PointListItem { x: 4, y: 1, next: 4 },
        /* 2 */ PointListItem { x: 2, y: 2, next: 3 },
        /* 3 */ PointListItem { x: 3, y: 3, next: 2 },
        /* 4 */ PointListItem { x: 1, y: 4, next: 0 },
        /* 5 */ PointListItem { x: 5, y: 4, next: 1 },
    ],
    vec![
        /* 0 */ root(NonZeroUsize::new(1)),
        /* 1 */ hier(0, 0, NonZeroUsize::new(2), None),
        /* 2 */ hier(2, 0, None, None),
    ]
)]
#[test_case(
    7, 5, vec![
        1, 1, 1, 1, 1, 1, 1,
        1, 0, 0, 0, 0, 0, 1,
        1, 0, 1, 1, 1, 0, 1,
        1, 0, 1, 0, 1, 0, 1,
        1, 0, 1, 1, 1, 0, 1,
    ],
    vec![
        /* 0 */ PointListItem { x: 0, y: 0, next: 7 },
        /* 1 */ PointListItem { x: 6, y: 1, next: 5 },
        /* 2 */ PointListItem { x: 2, y: 2, next: 6 },
        /* 3 */ PointListItem { x: 4, y: 3, next: 4 },
        /* 4 */ PointListItem { x: 3, y: 4, next: 3 },
        /* 5 */ PointListItem { x: 1, y: 5, next: 0 },
        /* 6 */ PointListItem { x: 5, y: 5, next: 2 },
        /* 7 */ PointListItem { x: 7, y: 5, next: 1 },
    ],
    vec![
        /* 0 */ root(NonZeroUsize::new(1)),
        /* 1 */ hier(0, 0, NonZeroUsize::new(2), None),
        /* 2 */ hier(2, 0, None, NonZeroUsize::new(3)),
        /* 3 */ hier(3, 2, None, None),
    ]
)]
fn small_test_images(
    width: u32, height: u32, image_pixels: Vec<u8>,
    expected_point_list: Vec<PointListItem>, expected_hierarchy: Vec<HierarchyItem>
) {
    let image = GrayImage::from_vec(width, height, image_pixels).unwrap();
    let actual = ImageContourCollection::new(&image, false);
    assert_eq!(actual.dimensions(), (width as i32, height as i32));
    assert_eq!(actual.point_list, expected_point_list);
    assert_eq!(actual.hierarchy, expected_hierarchy);
}

#[test]
fn hierarchy_consistency() {
    test_all_images(|testcase, _, _, contour_collection| {
        let h = contour_collection.hierarchy;
        let mut is_visited = vec![false; h.len()];
        
        let mut stack = vec![0];
        while let Some(index) = stack.pop() {
            is_visited[index] = true;
            let current = &h[index];
            
            if let Some(child) = current.first_child {
                assert!(!is_visited[child.get()],
                    "{testcase}: cycle in hierarchy tree (item {child})");
                assert_eq!(h[child.get()].parent, index,
                    "{testcase}: hierarchy item {child} has incorrect parent");
                stack.push(child.get());
            }
            if let Some(sibling) = current.next_sibling {
                assert!(!is_visited[sibling.get()],
                    "{testcase}: cycle in hierarchy tree (item {sibling})");
                assert_eq!(h[sibling.get()].parent, current.parent,
                    "{testcase}: hierarchy item {sibling} has incorrect parent");
                stack.push(sibling.get());
            }
        }
        
        assert!(is_visited.iter().all(|&v| v),
            "{testcase}: not all hierarchy items accessible");
    })
}

#[test]
fn contour_folding() {
    test_all_images(|testcase, _, _, contour_collection| {
        for contour in contour_collection.all_contours() {
            let Some(parent) = contour.parent() else { continue };
            for point in contour.vertices() {
                let is_ok = match parent.get_point_position(point) {
                    PointPosition::Inside => true,
                    PointPosition::Vertex => parent.is_outer(),
                    PointPosition::Outside | PointPosition::Edge => false,
                };
                assert!(is_ok, "{testcase}: a contour point is outside its parent contour");
            }
        }
    })
}

//#[test]
fn rasterization() {
    test_all_images(|testcase, image, inverted, contour_collection| {
        let (width, height) = contour_collection.dimensions();
        let contours: Vec<_> = contour_collection.all_contours().collect();
        let mut canvas = GrayImage::new(width as u32, height as u32);
        
        draw_orthopolygons(&mut canvas, |_| 255, contours.iter());
        canvas.save(format!("output/{testcase}.png")).unwrap();
        
        let mut pixels = canvas.pixels().zip_eq(image.pixels());
        let are_equal = if inverted {
            pixels.all(|(&Luma([actual]), &Luma([expected]))| 255 - actual == expected)
        } else {
            pixels.all(|(actual, expected)| actual == expected)
        };
        
        assert!(are_equal, "{testcase}: rasterized contours differ from the original image");
    })
}

fn test_all_images(test: impl Fn(String, &GrayImage, bool, ImageContourCollection)) {
    for (name, image) in get_test_images() {
        for &inverted in [false, true].iter() {
            let contour_collection = ImageContourCollection::new(&image, inverted);
            let testcase = if inverted {
                format!("Image: '{name}', inverted")
            } else {
                format!("Image: '{name}'")
            };
            test(testcase, &image, inverted, contour_collection);
        }
    }
}

const fn root(first_child: Option<NonZeroUsize>) -> HierarchyItem {
    HierarchyItem { head_point_index: 0, parent: 0, next_sibling: None, first_child }
}

const fn hier(head_point: usize, parent: usize, next_sibling: Option<NonZeroUsize>, first_child: Option<NonZeroUsize>) -> HierarchyItem {
    HierarchyItem { head_point_index: head_point, parent, next_sibling, first_child }
}
