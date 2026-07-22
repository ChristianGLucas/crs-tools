// Separate test file: nodes/reproject_bounding_box_test.rs. The generated service wires
// it into the crate via `#[cfg(test)] #[path="nodes/reproject_bounding_box_test.rs"] mod
// reproject_bounding_box_test;`. It reaches the node + SDK through `crate::` paths (this is
// a sibling module of the node, not a child — so `super::*` would not resolve).
#[path = "crsutil.rs"]
mod crsutil;

#[cfg(test)]
mod tests {
    use crate::axiom_context::*;
    use crate::gen::messages::BBoxTransformInput;
    use crate::reproject_bounding_box::reproject_bounding_box;
    use std::collections::HashMap;
    use super::crsutil;

    // TESTS — delete this block when done ─────────────────────────────────────
    // Tests are required to publish this package. The publish pipeline runs your
    // tests as a quality gate — a package will not be published if tests fail or
    // do not meet the minimum requirements.
    //
    // Requirements checked before publishing:
    //   - At least one test per node
    //   - All tests must pass
    //   - Output fields must be meaningfully asserted — not just Ok-checked
    //
    // The generated test below is a starting point. Replace the TODO with real
    // assertions: given a specific input, what should the output fields contain?
    //
    // Run your tests locally at any time:  axiom test

    struct TestLogger;
    impl AxiomLogger for TestLogger {
        fn debug(&self, _m: &str, _a: &HashMap<&str, String>) {}
        fn info(&self, _m: &str, _a: &HashMap<&str, String>) {}
        fn warn(&self, _m: &str, _a: &HashMap<&str, String>) {}
        fn error(&self, _m: &str, _a: &HashMap<&str, String>) {}
    }
    struct TestSecrets;
    impl AxiomSecrets for TestSecrets {
        fn get(&self, _n: &str) -> (String, bool) { (String::new(), false) }
        fn status(&self, _n: &str) -> SecretStatus { SecretStatus::Unset }
    }
    struct EmptyFlow { pos: FlowPosition }
    impl FlowReflection for EmptyFlow {
        fn nodes(&self) -> &[ReflectionNode] { &[] }
        fn edges(&self) -> &[ReflectionEdge] { &[] }
        fn loop_edges(&self) -> &[ReflectionEdge] { &[] }
        fn position(&self) -> &FlowPosition { &self.pos }
        fn graph_id(&self) -> &str { "" }
    }
    struct TestReflection { flow: EmptyFlow }
    impl Reflection for TestReflection { fn flow(&self) -> &dyn FlowReflection { &self.flow } }
    struct TestFlowMut;
    impl FlowMutation for TestFlowMut {
        fn add_node(&self, _p: &str, _v: &str, _c: Option<CanvasPosition>) -> u32 { 0 }
        fn add_edge(&self, _s: u32, _d: u32, _c: Option<EdgeCondition>) {}
    }
    struct TestMutation { flow: TestFlowMut }
    impl Mutation for TestMutation { fn flow(&self) -> &dyn FlowMutation { &self.flow } }

    // Mock AxiomContext a node author edits to drive a specific test scenario.
    struct TestContext {
        log: TestLogger, secrets: TestSecrets,
        reflection: TestReflection, mutation: TestMutation,
    }
    impl AxiomContext for TestContext {
        fn log(&self) -> &dyn AxiomLogger { &self.log }
        fn secrets(&self) -> &dyn AxiomSecrets { &self.secrets }
        fn execution_id(&self) -> &str { "test-execution-id" }
        fn flow_id(&self) -> &str { "test-flow-id" }
        fn tenant_id(&self) -> &str { "test-tenant-id" }
        fn reflection(&self) -> &dyn Reflection { &self.reflection }
        fn mutation(&self) -> &dyn Mutation { &self.mutation }
    }
    fn test_context() -> TestContext {
        TestContext {
            log: TestLogger, secrets: TestSecrets,
            reflection: TestReflection { flow: EmptyFlow { pos: FlowPosition::default() } },
            mutation: TestMutation { flow: TestFlowMut },
        }
    }

    fn bbox(min_x: f64, min_y: f64, max_x: f64, max_y: f64, source_crs: &str, target_crs: &str) -> BBoxTransformInput {
        BBoxTransformInput { min_x, min_y, max_x, max_y, source_crs: source_crs.to_string(), target_crs: target_crs.to_string() }
    }

    // Independent oracle: the Web Mercator world extent, +/-20037508.3428 m
    // on x, is a widely documented constant (half the WGS-84
    // semi-major-axis circumference) -- reprojecting the full WGS-84
    // longitude range must hit it exactly on both corners.
    #[test]
    fn test_world_bbox_matches_known_web_mercator_extent() {
        let ax = test_context();
        let out = reproject_bounding_box(&ax, bbox(-180.0, 0.0, 180.0, 0.0, "EPSG:4326", "EPSG:3857")).unwrap();
        assert_eq!(out.error, "");
        assert!((out.min_x - (-20_037_508.3428)).abs() < 0.1, "min_x={}", out.min_x);
        assert!((out.max_x - 20_037_508.3428).abs() < 0.1, "max_x={}", out.max_x);
    }

    #[test]
    fn test_identity_crs_is_a_noop() {
        let ax = test_context();
        let out = reproject_bounding_box(&ax, bbox(2.0, 40.0, 10.0, 50.0, "EPSG:4326", "EPSG:4326")).unwrap();
        assert_eq!(out.error, "");
        assert!((out.min_x - 2.0).abs() < 1e-6);
        assert!((out.min_y - 40.0).abs() < 1e-6);
        assert!((out.max_x - 10.0).abs() < 1e-6);
        assert!((out.max_y - 50.0).abs() < 1e-6);
    }

    // A box straddling a UTM zone's central meridian: densifying the top
    // and bottom edges must produce a WIDER reprojected x-range than
    // transforming the two corners alone would (the projection bows the
    // edge outward off the central meridian) -- this is the concrete,
    // checkable consequence of edge densification, independent of any
    // specific numeric oracle.
    #[test]
    fn test_densification_widens_bbox_beyond_corner_only_reprojection() {
        let ax = test_context();
        let out = reproject_bounding_box(&ax, bbox(10.0, 40.0, 20.0, 60.0, "EPSG:4326", "EPSG:32633")).unwrap();
        assert_eq!(out.error, "");
        // Corner-only reprojection of these 4 corners:
        let from = crsutil::build_proj("EPSG:4326").unwrap();
        let to = crsutil::build_proj("EPSG:32633").unwrap();
        let mut corner_min_x = f64::INFINITY;
        let mut corner_max_x = f64::NEG_INFINITY;
        for (x, y) in [(10.0, 40.0), (20.0, 40.0), (10.0, 60.0), (20.0, 60.0)] {
            let (ox, _) = crsutil::transform_xy(&from, &to, x, y).unwrap();
            corner_min_x = corner_min_x.min(ox);
            corner_max_x = corner_max_x.max(ox);
        }
        assert!(out.min_x <= corner_min_x + 1e-6, "densified min_x={} corner min_x={}", out.min_x, corner_min_x);
        assert!(out.max_x >= corner_max_x - 1e-6, "densified max_x={} corner max_x={}", out.max_x, corner_max_x);
    }

    #[test]
    fn test_inverted_bbox_is_structured_error() {
        let ax = test_context();
        let out = reproject_bounding_box(&ax, bbox(10.0, 10.0, 0.0, 0.0, "EPSG:4326", "EPSG:3857")).unwrap();
        assert_eq!(out.error, "INVALID_ARGUMENT");
    }

    #[test]
    fn test_bad_crs_is_structured_error() {
        let ax = test_context();
        let out = reproject_bounding_box(&ax, bbox(0.0, 0.0, 1.0, 1.0, "nope", "EPSG:3857")).unwrap();
        assert_eq!(out.error, "UNPARSEABLE");
    }
}
