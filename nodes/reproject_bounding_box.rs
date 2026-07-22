use crate::axiom_context::AxiomContext;
use crate::gen::messages::{ BBoxTransformInput, BBoxResult };

#[path = "crsutil.rs"]
mod crsutil;

/// Reproject a bounding box (min/max corners) from `source_crs` to
/// `target_crs`. Densifies each edge before transforming (21 sample points
/// per edge) rather than transforming the two corners alone, since a
/// projection can bow an edge outward -- the same technique PROJ's own
/// proj_trans_bounds uses.
pub fn reproject_bounding_box(
    ax: &dyn AxiomContext,
    input: BBoxTransformInput,
) -> Result<BBoxResult, Box<dyn std::error::Error>> {
    let _ = ax;
    if !crsutil::finite(input.min_x, input.min_y) || !crsutil::finite(input.max_x, input.max_y) {
        return Ok(BBoxResult { error: "NON_FINITE_COORD".into(), ..Default::default() });
    }
    if input.min_x > input.max_x || input.min_y > input.max_y {
        return Ok(BBoxResult { error: "INVALID_ARGUMENT".into(), ..Default::default() });
    }
    let from = match crsutil::build_proj(&input.source_crs) {
        Ok(p) => p,
        Err(e) => return Ok(BBoxResult { error: e.into(), ..Default::default() }),
    };
    let to = match crsutil::build_proj(&input.target_crs) {
        Ok(p) => p,
        Err(e) => return Ok(BBoxResult { error: e.into(), ..Default::default() }),
    };
    match crsutil::densify_and_transform_bounds(&from, &to, input.min_x, input.min_y, input.max_x, input.max_y) {
        Ok((min_x, min_y, max_x, max_y)) => Ok(BBoxResult { min_x, min_y, max_x, max_y, error: String::new() }),
        Err(e) => Ok(BBoxResult { error: e.into(), ..Default::default() }),
    }
}
