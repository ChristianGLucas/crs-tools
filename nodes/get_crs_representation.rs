use crate::axiom_context::AxiomContext;
use crate::gen::messages::{ CrsQuery, CrsRepresentation };

#[path = "crsutil.rs"]
mod crsutil;

/// The WKT2 (ISO 19162) and PROJ4-string representations of a CRS, from the
/// crs-definitions EPSG registry.
pub fn get_crs_representation(
    ax: &dyn AxiomContext,
    input: CrsQuery,
) -> Result<CrsRepresentation, Box<dyn std::error::Error>> {
    let _ = ax;
    let (_, def) = match crsutil::resolve_def(&input.crs) {
        Ok(v) => v,
        Err(e) => return Ok(CrsRepresentation { wkt: String::new(), proj4: String::new(), error: e.into() }),
    };
    Ok(CrsRepresentation {
        wkt: def.wkt.to_string(),
        proj4: def.proj4.to_string(),
        error: String::new(),
    })
}
