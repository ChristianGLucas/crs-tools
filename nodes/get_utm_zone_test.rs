// Separate test file: nodes/get_utm_zone_test.rs. The generated service wires
// it into the crate via `#[cfg(test)] #[path="nodes/get_utm_zone_test.rs"] mod
// get_utm_zone_test;`. It reaches the node + SDK through `crate::` paths (this is
// a sibling module of the node, not a child — so `super::*` would not resolve).
#[cfg(test)]
mod tests {
    use crate::axiom_context::*;
    use crate::gen::messages::GeoCoordinate;
    use crate::get_utm_zone::get_utm_zone;
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

    // Independent oracle: Zurich, Switzerland is widely documented as lying
    // in UTM zone 32N (EPSG:32632) -- a well-known geographic fact,
    // independent of this package's own zoning formula.
    #[test]
    fn test_zurich_is_zone_32n() {
        let ax = test_context();
        let out = get_utm_zone(&ax, geo(8.541111, 47.374444)).unwrap();
        assert_eq!(out.error, "");
        assert_eq!(out.zone_number, 32);
        assert_eq!(out.hemisphere, "N");
        assert_eq!(out.epsg_code, "EPSG:32632");
        assert!(out.crs_name.contains("32N"), "name={}", out.crs_name);
    }

    // Independent oracle: New York City is widely documented as lying in
    // UTM zone 18N (EPSG:32618).
    #[test]
    fn test_nyc_is_zone_18n() {
        let ax = test_context();
        let out = get_utm_zone(&ax, geo(-73.9857, 40.7484)).unwrap();
        assert_eq!(out.zone_number, 18);
        assert_eq!(out.hemisphere, "N");
        assert_eq!(out.epsg_code, "EPSG:32618");
    }

    // Independent oracle: Sydney, Australia is widely documented as lying
    // in UTM zone 56S (EPSG:32756) -- exercises the southern-hemisphere path.
    #[test]
    fn test_sydney_is_zone_56s() {
        let ax = test_context();
        let out = get_utm_zone(&ax, geo(151.2093, -33.8688)).unwrap();
        assert_eq!(out.zone_number, 56);
        assert_eq!(out.hemisphere, "S");
        assert_eq!(out.epsg_code, "EPSG:32756");
        assert!(out.crs_name.contains("56S"), "name={}", out.crs_name);
    }

    #[test]
    fn test_out_of_range_is_structured_error() {
        let ax = test_context();
        let out = get_utm_zone(&ax, geo(0.0, 91.0)).unwrap();
        assert_eq!(out.error, "OUT_OF_RANGE");
    }
}
