use crate::axiom_context::AxiomContext;
use crate::gen::messages::{ CrsQuery, CrsCanonical };

#[path = "crsutil.rs"]
mod crsutil;

/// Normalize a CRS identifier -- an EPSG code ("EPSG:4326"/"4326") or an
/// EPSG-tagged WKT string -- into its canonical registry form: authority,
/// code, name, and full WKT2/PROJ4 definitions. (A raw, non-EPSG PROJ4
/// string has no registry entry to normalize against and is rejected with
/// UNPARSEABLE; use ValidateCRS to merely check that proj4rs can build a
/// projection from one.)
pub fn parse_crs(
    ax: &dyn AxiomContext,
    input: CrsQuery,
) -> Result<CrsCanonical, Box<dyn std::error::Error>> {
    let _ = ax;
    let (code, def) = match crsutil::resolve_def(&input.crs) {
        Ok(v) => v,
        Err(e) => return Ok(CrsCanonical { error: e.into(), ..Default::default() }),
    };
    Ok(CrsCanonical {
        epsg_code: format!("EPSG:{}", code),
        authority: "EPSG".to_string(),
        code: code.to_string(),
        proj4: def.proj4.to_string(),
        wkt: def.wkt.to_string(),
        name: crsutil::wkt_name(def.wkt),
        error: String::new(),
    })
}
