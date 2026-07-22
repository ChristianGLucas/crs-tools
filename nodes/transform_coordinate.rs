use crate::axiom_context::AxiomContext;
use crate::gen::messages::{ CoordinateTransformInput, TransformedCoordinate };

#[path = "crsutil.rs"]
mod crsutil;

/// Transform a single (x, y) coordinate from `source_crs` to `target_crs`
/// (each an EPSG code like "EPSG:4326"/"4326", or a raw PROJ4 definition
/// string), via proj4rs. x/y follow GIS convention: longitude/easting then
/// latitude/northing, regardless of either CRS's own authority-defined axis
/// order (see GetCRSAxisInfo).
pub fn transform_coordinate(
    ax: &dyn AxiomContext,
    input: CoordinateTransformInput,
) -> Result<TransformedCoordinate, Box<dyn std::error::Error>> {
    let _ = ax;
    let from = match crsutil::build_proj(&input.source_crs) {
        Ok(p) => p,
        Err(e) => return Ok(TransformedCoordinate { x: 0.0, y: 0.0, error: e.into() }),
    };
    let to = match crsutil::build_proj(&input.target_crs) {
        Ok(p) => p,
        Err(e) => return Ok(TransformedCoordinate { x: 0.0, y: 0.0, error: e.into() }),
    };
    match crsutil::transform_xy(&from, &to, input.x, input.y) {
        Ok((x, y)) => Ok(TransformedCoordinate { x, y, error: String::new() }),
        Err(e) => Ok(TransformedCoordinate { x: 0.0, y: 0.0, error: e.into() }),
    }
}
