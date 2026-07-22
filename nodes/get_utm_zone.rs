use crate::axiom_context::AxiomContext;
use crate::gen::messages::{ GeoCoordinate, UtmZoneInfo };

#[path = "crsutil.rs"]
mod crsutil;

/// Determine the UTM zone/hemisphere/EPSG code covering a WGS-84 lon/lat
/// point, per the standard UTM zoning formula and the standard WGS-84/UTM
/// EPSG numbering scheme (326xx north, 327xx south) -- then cross-checked
/// by confirming that EPSG code actually resolves in the CRS registry. This
/// identifies a suitable projected CRS for accurate local measurement near
/// a point; it does not convert a coordinate INTO UTM eastings/northings
/// (see christiangeorgelucas/geo-encoding-tools' LatLonToUTM for that).
pub fn get_utm_zone(
    ax: &dyn AxiomContext,
    input: GeoCoordinate,
) -> Result<UtmZoneInfo, Box<dyn std::error::Error>> {
    let _ = ax;
    if let Err(e) = crsutil::geo_in_range(input.lon, input.lat) {
        return Ok(UtmZoneInfo {
            zone_number: 0,
            hemisphere: String::new(),
            epsg_code: String::new(),
            crs_name: String::new(),
            error: e.into(),
        });
    }
    let mut zone = ((input.lon + 180.0) / 6.0).floor() as i32 + 1;
    zone = zone.clamp(1, 60);
    let north = input.lat >= 0.0;
    let hemisphere = if north { "N" } else { "S" };
    let code_num: u16 = if north { 32600 + zone as u16 } else { 32700 + zone as u16 };

    // Cross-check the formula-derived code against the registry: if it
    // isn't found, that's a real inconsistency, not something to paper over.
    let def = match crs_definitions::from_code(code_num) {
        Some(d) => d,
        None => {
            return Ok(UtmZoneInfo {
                zone_number: zone,
                hemisphere: hemisphere.to_string(),
                epsg_code: String::new(),
                crs_name: String::new(),
                error: "UNKNOWN_EPSG_CODE".into(),
            });
        }
    };
    Ok(UtmZoneInfo {
        zone_number: zone,
        hemisphere: hemisphere.to_string(),
        epsg_code: format!("EPSG:{}", code_num),
        crs_name: crsutil::wkt_name(def.wkt),
        error: String::new(),
    })
}
