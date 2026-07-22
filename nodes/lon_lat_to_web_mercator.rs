use crate::axiom_context::AxiomContext;
use crate::gen::messages::{ GeoCoordinate, MercatorPoint };
use proj4rs::proj::Proj;

#[path = "crsutil.rs"]
mod crsutil;

/// Convert a WGS-84 lon/lat point (EPSG:4326) to Web Mercator (EPSG:3857)
/// meters, the projection used by most web slippy-tile map libraries.
pub fn lon_lat_to_web_mercator(
    ax: &dyn AxiomContext,
    input: GeoCoordinate,
) -> Result<MercatorPoint, Box<dyn std::error::Error>> {
    let _ = ax;
    if let Err(e) = crsutil::geo_in_range(input.lon, input.lat) {
        return Ok(MercatorPoint { x: 0.0, y: 0.0, error: e.into() });
    }
    // Web Mercator's own valid domain: beyond +/-85.06 degrees latitude the
    // projection's y coordinate diverges toward infinity.
    if input.lat.abs() > 85.06 {
        return Ok(MercatorPoint { x: 0.0, y: 0.0, error: "OUT_OF_RANGE".into() });
    }
    // EPSG:4326 / EPSG:3857 are always resolvable — unwrap is safe.
    let from = Proj::from_epsg_code(4326).unwrap();
    let to = Proj::from_epsg_code(3857).unwrap();
    match crsutil::transform_xy(&from, &to, input.lon, input.lat) {
        Ok((x, y)) => Ok(MercatorPoint { x, y, error: String::new() }),
        Err(e) => Ok(MercatorPoint { x: 0.0, y: 0.0, error: e.into() }),
    }
}
