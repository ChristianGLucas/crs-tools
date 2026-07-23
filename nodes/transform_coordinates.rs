use crate::axiom_context::AxiomContext;
use crate::gen::messages::{ BatchCoordinateTransformInput, BatchTransformedCoordinates, TransformedCoordinate };

#[path = "crsutil.rs"]
mod crsutil;

/// Transform a list of (x, y) coordinates from `source_crs` to `target_crs`
/// in one call. A per-point failure (e.g. one non-finite coordinate) is
/// reported on that element alone; the rest of the batch still transforms.
/// A call-level failure (bad CRS, an empty batch) reports on the top-level
/// `error` with an empty `points` list.
pub fn transform_coordinates(
    ax: &dyn AxiomContext,
    input: BatchCoordinateTransformInput,
) -> Result<BatchTransformedCoordinates, Box<dyn std::error::Error>> {
    let _ = ax;
    if input.points.is_empty() {
        return Ok(BatchTransformedCoordinates { points: vec![], error: "EMPTY_INPUT".into() });
    }
    let from = match crsutil::build_proj(&input.source_crs) {
        Ok(p) => p,
        Err(e) => return Ok(BatchTransformedCoordinates { points: vec![], error: e.into() }),
    };
    let to = match crsutil::build_proj(&input.target_crs) {
        Ok(p) => p,
        Err(e) => return Ok(BatchTransformedCoordinates { points: vec![], error: e.into() }),
    };
    let points = input
        .points
        .iter()
        .map(|p| match crsutil::transform_xy(&from, &to, p.x, p.y) {
            Ok((x, y)) => TransformedCoordinate { x, y, error: String::new() },
            Err(e) => TransformedCoordinate { x: 0.0, y: 0.0, error: e.into() },
        })
        .collect();
    Ok(BatchTransformedCoordinates { points, error: String::new() })
}
