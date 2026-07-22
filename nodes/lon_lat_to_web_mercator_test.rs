// Separate test file: nodes/lon_lat_to_web_mercator_test.rs. The generated service wires
// it into the crate via `#[cfg(test)] #[path="nodes/lon_lat_to_web_mercator_test.rs"] mod
// lon_lat_to_web_mercator_test;`. It reaches the node + SDK through `crate::` paths (this is
// a sibling module of the node, not a child — so `super::*` would not resolve).
#[cfg(test)]
mod tests {
    use crate::axiom_context::*;
    use crate::gen::messages::GeoCoordinate;
    use crate::lon_lat_to_web_mercator::lon_lat_to_web_mercator;
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

    fn geo(lon: f64, lat: f64) -> GeoCoordinate {
        GeoCoordinate { lon, lat, error: String::new() }
    }

    // Independent oracle: go-proj's README documents this exact conversion,
    // computed by the real PROJ C library (a wholly separate implementation
    // from proj4rs).
    #[test]
    fn test_agrees_with_independent_proj_oracle_zurich() {
        let ax = test_context();
        let out = lon_lat_to_web_mercator(&ax, geo(8.541111, 47.374444)).unwrap();
        assert_eq!(out.error, "");
        assert!((out.x - 950792.127329).abs() < 0.01, "x={}", out.x);
        assert!((out.y - 6003408.475803).abs() < 0.01, "y={}", out.y);
    }

    // Independent oracle: the Web Mercator world extent, +/-20037508.3428 m,
    // is a widely documented constant (2*pi*6378137 / 2, the WGS-84
    // semi-major axis's half-circumference) -- (lon=180, lat=0).
    #[test]
    fn test_matches_known_world_extent_constant() {
        let ax = test_context();
        let out = lon_lat_to_web_mercator(&ax, geo(180.0, 0.0)).unwrap();
        assert_eq!(out.error, "");
        assert!((out.x - 20_037_508.3428).abs() < 0.01, "x={}", out.x);
        assert!(out.y.abs() < 1e-6);
    }

    #[test]
    fn test_origin_is_origin() {
        let ax = test_context();
        let out = lon_lat_to_web_mercator(&ax, geo(0.0, 0.0)).unwrap();
        assert_eq!(out.error, "");
        assert!(out.x.abs() < 1e-6 && out.y.abs() < 1e-6);
    }

    #[test]
    fn test_beyond_mercator_domain_is_structured_error() {
        let ax = test_context();
        let out = lon_lat_to_web_mercator(&ax, geo(0.0, 86.0)).unwrap();
        assert_eq!(out.error, "OUT_OF_RANGE");
    }

    #[test]
    fn test_out_of_range_lat_is_structured_error() {
        let ax = test_context();
        let out = lon_lat_to_web_mercator(&ax, geo(0.0, 91.0)).unwrap();
        assert_eq!(out.error, "OUT_OF_RANGE");
    }

    #[test]
    fn test_non_finite_is_structured_error() {
        let ax = test_context();
        let out = lon_lat_to_web_mercator(&ax, geo(f64::NAN, 0.0)).unwrap();
        assert_eq!(out.error, "NON_FINITE_COORD");
    }
}
