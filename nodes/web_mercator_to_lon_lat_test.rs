// Separate test file: nodes/web_mercator_to_lon_lat_test.rs. The generated service wires
// it into the crate via `#[cfg(test)] #[path="nodes/web_mercator_to_lon_lat_test.rs"] mod
// web_mercator_to_lon_lat_test;`. It reaches the node + SDK through `crate::` paths (this is
// a sibling module of the node, not a child — so `super::*` would not resolve).
#[cfg(test)]
mod tests {
    use crate::axiom_context::*;
    use crate::gen::messages::MercatorPoint;
    use crate::web_mercator_to_lon_lat::web_mercator_to_lon_lat;
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

    fn merc(x: f64, y: f64) -> MercatorPoint {
        MercatorPoint { x, y, error: String::new() }
    }

    // Independent oracle: the go-proj README's Zurich example, inverse
    // direction -- feeding its published EPSG:3857 output back through
    // WebMercatorToLonLat should recover its published EPSG:4326 input.
    #[test]
    fn test_agrees_with_independent_proj_oracle_zurich_inverse() {
        let ax = test_context();
        let out = web_mercator_to_lon_lat(&ax, merc(950792.127329, 6003408.475803)).unwrap();
        assert_eq!(out.error, "");
        assert!((out.lon - 8.541111).abs() < 1e-6, "lon={}", out.lon);
        assert!((out.lat - 47.374444).abs() < 1e-6, "lat={}", out.lat);
    }

    #[test]
    fn test_origin_is_origin() {
        let ax = test_context();
        let out = web_mercator_to_lon_lat(&ax, merc(0.0, 0.0)).unwrap();
        assert_eq!(out.error, "");
        assert!(out.lon.abs() < 1e-6 && out.lat.abs() < 1e-6);
    }

    #[test]
    fn test_round_trip_with_lon_lat_to_web_mercator() {
        let ax = test_context();
        let forward = crate::lon_lat_to_web_mercator::lon_lat_to_web_mercator(
            &ax,
            crate::gen::messages::GeoCoordinate { lon: -122.4194, lat: 37.7749, error: String::new() },
        )
        .unwrap();
        let back = web_mercator_to_lon_lat(&ax, merc(forward.x, forward.y)).unwrap();
        assert!((back.lon - (-122.4194)).abs() < 1e-6);
        assert!((back.lat - 37.7749).abs() < 1e-6);
    }

    #[test]
    fn test_non_finite_is_structured_error() {
        let ax = test_context();
        let out = web_mercator_to_lon_lat(&ax, merc(f64::NAN, 0.0)).unwrap();
        assert_eq!(out.error, "NON_FINITE_COORD");
    }
}
