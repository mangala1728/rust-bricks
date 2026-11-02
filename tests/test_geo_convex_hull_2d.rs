use rust_bricks::geo_convex_hull_2d::*;

#[test]
fn test_convex_hull_square_with_center() {
    // Golden vector set: a square with a centre point.
    let pts = vec![
        (0.0, 0.0),
        (4.0, 0.0),
        (4.0, 4.0),
        (0.0, 4.0),
        (2.0, 2.0), // interior point – must be excluded
    ];

    let hull_opt = convex_hull_2d(pts);
    assert!(hull_opt.is_some(), "Hull should be produced for non‑empty input");

    let hull = hull_opt.unwrap();
    // Expected hull vertices (counter‑clockwise, start at (0,0)).
    let expected = vec![
        (0.0, 0.0),
        (4.0, 0.0),
        (4.0, 4.0),
        (0.0, 4.0),
    ];

    assert_eq!(hull.len(), expected.len(), "Hull must contain exactly 4 vertices");
    for (got, exp) in hull.iter().zip(expected.iter()) {
        assert!((got.0 - exp.0).abs() < 1e-12 && (got.1 - exp.1).abs() < 1e-12,
                "Vertex mismatch: got {:?}, expected {:?}", got, exp);
    }
}

#[test]
fn test_convex_hull_domain_guards() {
    assert!(convex_hull_2d(vec![]).is_none());
    assert!(convex_hull_2d(vec![(1.0e10, 0.0), (0.0, 0.0), (1.0, 1.0)]).is_none());
    assert!(convex_hull_2d(vec![(-1.0e10, 0.0), (0.0, 0.0), (1.0, 1.0)]).is_none());

    // Single or two points
    let one_pt = vec![(1.0, 2.0)];
    let res = convex_hull_2d(one_pt.clone());
    assert_eq!(res, Some(one_pt));
}
