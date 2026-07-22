use crate::axiom_context::AxiomContext;
use crate::gen::messages::{ CrsQuery, CrsValidation };

#[path = "crsutil.rs"]
mod crsutil;

/// Validate that a CRS identifier (EPSG code, PROJ4 string, or EPSG-tagged
/// WKT) is recognized -- an unknown or malformed identifier returns
/// `valid: false` with a `reason`, never a crash.
pub fn validate_crs(
    ax: &dyn AxiomContext,
    input: CrsQuery,
) -> Result<CrsValidation, Box<dyn std::error::Error>> {
    let _ = ax;
    let q = input.crs.trim();
    if q.starts_with('+') {
        // A raw PROJ4 string has no EPSG code/registry name of its own; it
        // is valid iff proj4rs can actually build a projection from it.
        return match crsutil::build_proj(q) {
            Ok(_) => Ok(CrsValidation { valid: true, epsg_code: String::new(), name: String::new(), reason: String::new() }),
            Err(e) => Ok(CrsValidation { valid: false, epsg_code: String::new(), name: String::new(), reason: e.into() }),
        };
    }
    match crsutil::resolve_def(q) {
        Ok((code, def)) => Ok(CrsValidation {
            valid: true,
            epsg_code: format!("EPSG:{}", code),
            name: crsutil::wkt_name(def.wkt),
            reason: String::new(),
        }),
        Err(e) => Ok(CrsValidation { valid: false, epsg_code: String::new(), name: String::new(), reason: e.into() }),
    }
}
