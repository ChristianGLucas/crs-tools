// Separate test file: nodes/transform_coordinates_test.rs. The generated service wires
// it into the crate via `#[cfg(test)] #[path="nodes/transform_coordinates_test.rs"] mod
// transform_coordinates_test;`. It reaches the node + SDK through `crate::` paths (this is
// a sibling module of the node, not a child — so `super::*` would not resolve).
#[cfg(test)]
mod tests {
    use crate::axiom_context::*;
    use crate::gen::messages::{ BatchCoordinateTransformInput, XyPoint };
    use crate::transform_coordinates::transform_coordinates;
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

    fn pt(x: f64, y: f64) -> XyPoint {
        XyPoint { x, y }
    }

    // Independent oracle: same Zurich EPSG:4326->EPSG:3857 value documented
    // in go-proj's README (see transform_coordinate_test.rs), applied to the
    // first element of a 3-point batch.
    #[test]
    fn test_batch_agrees_with_independent_oracle_and_transforms_every_point() {
        let ax = test_context();
        let input = BatchCoordinateTransformInput {
            points: vec![pt(8.541111, 47.374444), pt(0.0, 0.0), pt(-73.9857, 40.7484)],
            source_crs: "EPSG:4326".to_string(),
            target_crs: "EPSG:3857".to_string(),
        };
        let out = transform_coordinates(&ax, input).unwrap();
        assert_eq!(out.error, "");
        assert_eq!(out.points.len(), 3);
        assert!((out.points[0].x - 950792.127329).abs() < 0.01);
        assert!((out.points[0].y - 6003408.475803).abs() < 0.01);
        assert_eq!(out.points[1].error, "");
        assert!(out.points[1].x.abs() < 1e-6 && out.points[1].y.abs() < 1e-6);
        assert_eq!(out.points[2].error, "");
    }

    #[test]
    fn test_one_bad_point_does_not_abort_the_batch() {
        let ax = test_context();
        let input = BatchCoordinateTransformInput {
            points: vec![pt(f64::NAN, 0.0), pt(2.0, 45.0)],
            source_crs: "EPSG:4326".to_string(),
            target_crs: "EPSG:3857".to_string(),
        };
        let out = transform_coordinates(&ax, input).unwrap();
        assert_eq!(out.error, "");
        assert_eq!(out.points.len(), 2);
        assert_eq!(out.points[0].error, "NON_FINITE_COORD");
        assert_eq!(out.points[1].error, "");
    }

    #[test]
    fn test_empty_batch_is_structured_error() {
        let ax = test_context();
        let input = BatchCoordinateTransformInput { points: vec![], source_crs: "EPSG:4326".to_string(), target_crs: "EPSG:3857".to_string() };
        let out = transform_coordinates(&ax, input).unwrap();
        assert_eq!(out.error, "EMPTY_INPUT");
        assert!(out.points.is_empty());
    }

    #[test]
    fn test_bad_crs_is_a_call_level_error() {
        let ax = test_context();
        let input = BatchCoordinateTransformInput { points: vec![pt(1.0, 1.0)], source_crs: "nope".to_string(), target_crs: "EPSG:3857".to_string() };
        let out = transform_coordinates(&ax, input).unwrap();
        assert_eq!(out.error, "UNPARSEABLE");
        assert!(out.points.is_empty());
    }
}
