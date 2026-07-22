use crate::axiom_context::AxiomContext;
use crate::gen::messages::{ CrsQuery, CrsInfo };

#[path = "crsutil.rs"]
mod crsutil;

/// Structured metadata about a CRS: name, type (geographic/projected),
/// datum, ellipsoid, unit, and the EPSG registry's own documented area of
/// applicability -- read from crs-definitions' bundled WKT2 EPSG registry
/// text.
pub fn get_crs_info(
    ax: &dyn AxiomContext,
    input: CrsQuery,
) -> Result<CrsInfo, Box<dyn std::error::Error>> {
    let _ = ax;
    let (code, def) = match crsutil::resolve_def(&input.crs) {
        Ok(v) => v,
        Err(e) => return Ok(CrsInfo { error: e.into(), ..Default::default() }),
    };
    let wkt = def.wkt;
    let (ellipsoid_name, ellipsoid_semi_major_m, ellipsoid_inverse_flattening) =
        crsutil::wkt_ellipsoid(wkt).unwrap_or_default();
    let (unit, unit_to_meter) = crsutil::wkt_unit(wkt).unwrap_or_default();
    let (area_description, area_south_lat, area_west_lon, area_north_lat, area_east_lon) =
        crsutil::wkt_area_of_use(wkt).unwrap_or_default();
    Ok(CrsInfo {
        epsg_code: format!("EPSG:{}", code),
        name: crsutil::wkt_name(wkt),
        crs_type: crsutil::wkt_crs_type(wkt).to_string(),
        datum: crsutil::wkt_datum(wkt),
        ellipsoid_name,
        ellipsoid_semi_major_m,
        ellipsoid_inverse_flattening,
        unit,
        unit_to_meter,
        area_description,
        area_south_lat,
        area_west_lon,
        area_north_lat,
        area_east_lon,
        error: String::new(),
    })
}
