// Separate test file: nodes/parse_crs_test.rs. The generated service wires
// it into the crate via `#[cfg(test)] #[path="nodes/parse_crs_test.rs"] mod
// parse_crs_test;`. It reaches the node + SDK through `crate::` paths (this is
// a sibling module of the node, not a child — so `super::*` would not resolve).
#[cfg(test)]
mod tests {
    use crate::axiom_context::*;
    use crate::gen::messages::CrsQuery;
    use crate::parse_crs::parse_crs;
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

    #[test]
    fn test_bare_epsg_code_normalizes() {
        let ax = test_context();
        let out = parse_crs(&ax, q("4326")).unwrap();
        assert_eq!(out.error, "");
        assert_eq!(out.epsg_code, "EPSG:4326");
        assert_eq!(out.authority, "EPSG");
        assert_eq!(out.code, "4326");
        assert_eq!(out.name, "WGS 84");
        assert!(out.proj4.contains("+proj=longlat"));
        assert!(out.wkt.starts_with("GEOGCRS["));
    }

    // Round trip through the WKT-tag-extraction path: feeding EPSG:3857's
    // own WKT (as produced by GetCRSRepresentation) back into ParseCRS must
    // resolve to the same EPSG code it came from.
    #[test]
    fn test_wkt_input_round_trips_through_its_own_authority_tag() {
        let ax = test_context();
        let first = parse_crs(&ax, q("EPSG:3857")).unwrap();
        assert_eq!(first.error, "");
        let second = parse_crs(&ax, q(&first.wkt)).unwrap();
        assert_eq!(second.error, "");
        assert_eq!(second.epsg_code, "EPSG:3857");
        assert_eq!(second.name, first.name);
    }

    #[test]
    fn test_unknown_epsg_code_is_structured_error() {
        let ax = test_context();
        let out = parse_crs(&ax, q("EPSG:1")).unwrap();
        assert_eq!(out.error, "UNKNOWN_EPSG_CODE");
    }

    #[test]
    fn test_non_epsg_proj4_string_is_rejected_not_a_crash() {
        let ax = test_context();
        let out = parse_crs(&ax, q("+proj=longlat +datum=WGS84 +no_defs")).unwrap();
        assert_eq!(out.error, "UNPARSEABLE");
    }
}
