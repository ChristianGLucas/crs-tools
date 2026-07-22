// Separate test file: nodes/get_crs_info_test.rs. The generated service wires
// it into the crate via `#[cfg(test)] #[path="nodes/get_crs_info_test.rs"] mod
// get_crs_info_test;`. It reaches the node + SDK through `crate::` paths (this is
// a sibling module of the node, not a child — so `super::*` would not resolve).
#[cfg(test)]
mod tests {
    use crate::axiom_context::*;
    use crate::gen::messages::CrsQuery;
    use crate::get_crs_info::get_crs_info;
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

    fn q(crs: &str) -> CrsQuery {
        CrsQuery { crs: crs.to_string() }
    }

    // Independent oracle: WGS-84's defining ellipsoid constants
    // (semi-major axis 6378137 m, inverse flattening 298.257223563) are
    // published NIMA/NGA/EPSG physical constants, not derived from this
    // package's own code.
    #[test]
    fn test_wgs84_matches_published_ellipsoid_constants() {
        let ax = test_context();
        let out = get_crs_info(&ax, q("EPSG:4326")).unwrap();
        assert_eq!(out.error, "");
        assert_eq!(out.epsg_code, "EPSG:4326");
        assert_eq!(out.name, "WGS 84");
        assert_eq!(out.crs_type, "geographic");
        assert_eq!(out.ellipsoid_name, "WGS 84");
        assert!((out.ellipsoid_semi_major_m - 6_378_137.0).abs() < 1e-6);
        assert!((out.ellipsoid_inverse_flattening - 298.257223563).abs() < 1e-6);
        assert_eq!(out.unit, "degree");
        // EPSG:4326's documented area of use is the whole world.
        assert_eq!(out.area_south_lat, -90.0);
        assert_eq!(out.area_west_lon, -180.0);
        assert_eq!(out.area_north_lat, 90.0);
        assert_eq!(out.area_east_lon, 180.0);
    }

    #[test]
    fn test_projected_crs_is_classified_projected() {
        let ax = test_context();
        let out = get_crs_info(&ax, q("EPSG:32633")).unwrap();
        assert_eq!(out.error, "");
        assert_eq!(out.crs_type, "projected");
        assert_eq!(out.unit, "metre");
        // UTM 33N's documented area of use does not cover the whole world.
        assert!(out.area_north_lat < 90.0);
    }

    #[test]
    fn test_unknown_epsg_code_is_structured_error() {
        let ax = test_context();
        let out = get_crs_info(&ax, q("EPSG:1")).unwrap();
        assert_eq!(out.error, "UNKNOWN_EPSG_CODE");
    }

    #[test]
    fn test_empty_input_is_structured_error() {
        let ax = test_context();
        let out = get_crs_info(&ax, q("")).unwrap();
        assert_eq!(out.error, "EMPTY_INPUT");
    }
}
