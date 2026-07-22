use crate::axiom_context::AxiomContext;
use crate::gen::messages::{ CrsQuery, CrsAxisInfo };

#[path = "crsutil.rs"]
mod crsutil;

/// The axis names/directions a CRS's own registry entry declares (its
/// ISO-19111 authority-defined order -- e.g. EPSG:4326 officially orders
/// latitude before longitude, a well-known GIS footgun), alongside this
/// package's own fixed x,y transform I/O convention.
pub fn get_crs_axis_info(
    ax: &dyn AxiomContext,
    input: CrsQuery,
) -> Result<CrsAxisInfo, Box<dyn std::error::Error>> {
    let _ = ax;
    let (_, def) = match crsutil::resolve_def(&input.crs) {
        Ok(v) => v,
        Err(e) => return Ok(CrsAxisInfo { error: e.into(), ..Default::default() }),
    };
    let axes = crsutil::wkt_axes(def.wkt);
    let axis_names = axes.iter().map(|(n, _)| n.clone()).collect();
    let axis_directions = axes.iter().map(|(_, d)| d.clone()).collect();
    let note = if axes.is_empty() {
        "This CRS's registry entry states no explicit AXIS clause.".to_string()
    } else {
        "axis_names/axis_directions are this CRS's own authority-defined order; every crs-tools transform node always takes/returns transform_io_order regardless.".to_string()
    };
    Ok(CrsAxisInfo {
        axis_names,
        axis_directions,
        transform_io_order: "x,y = longitude/easting, latitude/northing".to_string(),
        note,
        error: String::new(),
    })
}
