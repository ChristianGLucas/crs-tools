use crate::axiom_context::AxiomContext;
use crate::gen::messages::{ MercatorPoint, GeoCoordinate };
use proj4rs::proj::Proj;

#[path = "crsutil.rs"]
mod crsutil;

/// Convert a Web Mercator (EPSG:3857) point in meters back to WGS-84
/// lon/lat (EPSG:4326).
pub fn web_mercator_to_lon_lat(
    ax: &dyn AxiomContext,
    input: MercatorPoint,
) -> Result<GeoCoordinate, Box<dyn std::error::Error>> {
    let _ = ax;
    if !crsutil::finite(input.x, input.y) {
        return Ok(GeoCoordinate { lon: 0.0, lat: 0.0, error: "NON_FINITE_COORD".into() });
    }
    let from = Proj::from_epsg_code(3857).unwrap();
    let to = Proj::from_epsg_code(4326).unwrap();
    match crsutil::transform_xy(&from, &to, input.x, input.y) {
        Ok((lon, lat)) => Ok(GeoCoordinate { lon, lat, error: String::new() }),
        Err(e) => Ok(GeoCoordinate { lon: 0.0, lat: 0.0, error: e.into() }),
    }
}
