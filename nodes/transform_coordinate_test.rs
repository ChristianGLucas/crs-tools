// Separate test file: nodes/transform_coordinate_test.rs. The generated service wires
// it into the crate via `#[cfg(test)] #[path="nodes/transform_coordinate_test.rs"] mod
// transform_coordinate_test;`. It reaches the node + SDK through `crate::` paths (this is
// a sibling module of the node, not a child — so `super::*` would not resolve).
#[cfg(test)]
mod tests {
    use crate::axiom_context::*;
    use crate::gen::messages::CoordinateTransformInput;
    use crate::transform_coordinate::transform_coordinate;
    use std::collections::HashMap;

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

    fn input(x: f64, y: f64, source_crs: &str, target_crs: &str) -> CoordinateTransformInput {
        CoordinateTransformInput { x, y, source_crs: source_crs.to_string(), target_crs: target_crs.to_string() }
    }

    // Independent oracle: go-proj's (github.com/twpayne/go-proj) own README
    // documents this exact EPSG:4326 -> EPSG:3857 conversion for Zurich,
    // computed via the real PROJ C library -- a wholly separate
    // implementation from proj4rs.
    //   proj.NewCRSToCRS("EPSG:4326","EPSG:3857").Forward(47.374444, 8.541111)
    //   == x=950792.127329 y=6003408.475803
    #[test]
    fn test_agrees_with_independent_proj_oracle_zurich() {
        let ax = test_context();
        let out = transform_coordinate(&ax, input(8.541111, 47.374444, "EPSG:4326", "EPSG:3857")).unwrap();
        assert_eq!(out.error, "");
        assert!((out.x - 950792.127329).abs() < 0.01, "x={}", out.x);
        assert!((out.y - 6003408.475803).abs() < 0.01, "y={}", out.y);
    }

    // Independent oracle: UTM's own definition guarantees a point exactly ON
    // a zone's central meridian gets easting == the 500,000 m false easting,
    // regardless of which library computes it. UTM zone 33N's central
    // meridian is 15E (EPSG:32633).
    #[test]
    fn test_utm_central_meridian_is_exactly_false_easting() {
        let ax = test_context();
        let out = transform_coordinate(&ax, input(15.0, 50.0, "EPSG:4326", "EPSG:32633")).unwrap();
        assert_eq!(out.error, "");
        assert!((out.x - 500_000.0).abs() < 0.01, "x={}", out.x);
        assert!(out.y > 0.0);
    }

    // Bare-digit and "EPSG:"-prefixed source_crs must resolve identically.
    #[test]
    fn test_bare_epsg_code_matches_prefixed() {
        let ax = test_context();
        let a = transform_coordinate(&ax, input(2.0, 45.0, "4326", "3857")).unwrap();
        let b = transform_coordinate(&ax, input(2.0, 45.0, "EPSG:4326", "EPSG:3857")).unwrap();
        assert_eq!(a.error, "");
        assert_eq!(a.x, b.x);
        assert_eq!(a.y, b.y);
    }

    #[test]
    fn test_identity_transform_is_a_noop() {
        let ax = test_context();
        let out = transform_coordinate(&ax, input(12.34, 56.78, "EPSG:4326", "EPSG:4326")).unwrap();
        assert_eq!(out.error, "");
        assert!((out.x - 12.34).abs() < 1e-9);
        assert!((out.y - 56.78).abs() < 1e-9);
    }

    #[test]
    fn test_unknown_crs_is_structured_error_not_crash() {
        let ax = test_context();
        let out = transform_coordinate(&ax, input(1.0, 1.0, "EPSG:99999999", "EPSG:3857")).unwrap();
        assert_eq!(out.error, "UNPARSEABLE");
        let out2 = transform_coordinate(&ax, input(1.0, 1.0, "not a crs", "EPSG:3857")).unwrap();
        assert_eq!(out2.error, "UNPARSEABLE");
    }

    #[test]
    fn test_non_finite_coord_is_structured_error() {
        let ax = test_context();
        let out = transform_coordinate(&ax, input(f64::NAN, 1.0, "EPSG:4326", "EPSG:3857")).unwrap();
        assert_eq!(out.error, "NON_FINITE_COORD");
    }

    #[test]
    fn test_raw_proj4_string_works_as_crs() {
        let ax = test_context();
        let out = transform_coordinate(
            &ax,
            input(8.541111, 47.374444, "+proj=longlat +datum=WGS84 +no_defs", "EPSG:3857"),
        )
        .unwrap();
        assert_eq!(out.error, "");
        assert!((out.x - 950792.127329).abs() < 0.01, "x={}", out.x);
    }
}
