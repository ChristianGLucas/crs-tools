// Shared CRS-resolution and WKT2-field-extraction helpers for crs-tools
// nodes.
//
// The generated service.rs only wires the node files listed in axiom.yaml as
// crate modules, so shared code cannot live at the crate root. Each node that
// needs these helpers includes this file as its own submodule with
// `#[path = "crsutil.rs"] mod crsutil;` (the `#[path]` is resolved relative
// to nodes/). Compiling it once per including node is harmless: every
// function is pure and returns only String/Vec/std types.
#![allow(dead_code)]

use proj4rs::proj::Proj;
use regex::Regex;

/// Max accepted length of a CRS identifier string (bytes) — an EPSG code,
/// PROJ4 string, or WKT. Checked on the RAW input before any parse/regex, so
/// a caller cannot force an expensive regex scan over an unbounded string.
pub const MAX_CRS_LEN: usize = 8_192;
/// Max points accepted per TransformCoordinates batch call.
pub const MAX_BATCH_POINTS: usize = 5_000;
/// Number of points sampled per edge when densifying a bounding box before
/// reprojecting it (matches the technique PROJ's own proj_trans_bounds uses:
/// a projection can bow an edge outward, so the reprojected corners alone do
/// not always bound the reprojected shape).
pub const BBOX_DENSIFY_POINTS: usize = 21;

/// True if x/y are finite. Coordinate range (vs. a specific CRS's domain) is
/// intentionally not checked here — a generic CRS's valid domain varies
/// per-projection, so an out-of-domain point is instead caught by
/// TRANSFORM_FAILED when proj4rs's own transform rejects it.
pub fn finite(x: f64, y: f64) -> bool {
    x.is_finite() && y.is_finite()
}

/// True if lon/lat are finite and within the WGS-84 geographic domain.
pub fn geo_in_range(lon: f64, lat: f64) -> Result<(), &'static str> {
    if !lon.is_finite() || !lat.is_finite() {
        return Err("NON_FINITE_COORD");
    }
    if lat.abs() > 90.0 || lon.abs() > 180.0 {
        return Err("OUT_OF_RANGE");
    }
    Ok(())
}

/// Extract a u16 EPSG code from a CRS identifier string: bare digits
/// ("4326"), an "EPSG:4326"-style prefix (case-insensitive), or a WKT string
/// carrying a trailing `ID["EPSG",4326]` (WKT2, what crs-definitions emits)
/// or `AUTHORITY["EPSG","4326"]` (older WKT1) tag — the LAST such tag in the
/// string is used, since a WKT for a projected CRS also carries its base
/// geographic CRS's own (different) EPSG code earlier in the same string.
pub fn parse_epsg_code(query: &str) -> Result<u16, &'static str> {
    let q = query.trim();
    if q.is_empty() {
        return Err("EMPTY_INPUT");
    }
    if q.len() > MAX_CRS_LEN {
        return Err("INPUT_TOO_LARGE");
    }
    if q.chars().all(|c| c.is_ascii_digit()) {
        return q.parse::<u16>().map_err(|_| "UNPARSEABLE");
    }
    let upper = q.to_ascii_uppercase();
    if let Some(rest) = upper.strip_prefix("EPSG:") {
        return rest.trim().parse::<u16>().map_err(|_| "UNPARSEABLE");
    }
    // WKT: take the last ID["EPSG",n] or AUTHORITY["EPSG","n"] tag.
    let re_id = Regex::new(r#"ID\["EPSG",\s*(\d+)\]"#).unwrap();
    if let Some(m) = re_id.captures_iter(q).last() {
        return m[1].parse::<u16>().map_err(|_| "UNPARSEABLE");
    }
    let re_auth = Regex::new(r#"AUTHORITY\["EPSG",\s*"(\d+)"\]"#).unwrap();
    if let Some(m) = re_auth.captures_iter(q).last() {
        return m[1].parse::<u16>().map_err(|_| "UNPARSEABLE");
    }
    Err("UNPARSEABLE")
}

/// Resolve a CRS query to its (epsg_code, registry Def). UNKNOWN_EPSG_CODE
/// means the query parsed to a well-formed code that is simply not in the
/// crs-definitions registry (not every EPSG code ever issued is included).
pub fn resolve_def(query: &str) -> Result<(u16, crs_definitions::Def), &'static str> {
    let code = parse_epsg_code(query)?;
    match crs_definitions::from_code(code) {
        Some(def) => Ok((code, def)),
        None => Err("UNKNOWN_EPSG_CODE"),
    }
}

/// Build a proj4rs `Proj` from a CRS query: a raw PROJ4 string (starts with
/// "+"), otherwise an EPSG code resolved through the registry (bare digits,
/// "EPSG:nnnn", or an EPSG-tagged WKT).
pub fn build_proj(query: &str) -> Result<Proj, &'static str> {
    let q = query.trim();
    if q.is_empty() {
        return Err("EMPTY_INPUT");
    }
    if q.len() > MAX_CRS_LEN {
        return Err("INPUT_TOO_LARGE");
    }
    if q.starts_with('+') {
        return Proj::from_proj_string(q).map_err(|_| "INVALID_CRS");
    }
    let code = parse_epsg_code(q)?;
    Proj::from_epsg_code(code).map_err(|_| "INVALID_CRS")
}

// ---------------------------------------------------------------------
// WKT2 field extraction. crs-definitions' `wkt` field is genuine ISO-19162
// WKT2 text (e.g. `GEOGCRS["WGS 84",...,AXIS[...],USAGE[...,BBOX[...]],
// ID["EPSG",4326]]`), so these are targeted regex extractions over a fixed,
// standardized text grammar -- not a re-derivation of any projection math.
// ---------------------------------------------------------------------

/// "geographic" | "projected" | "unknown", from the WKT's outermost keyword.
pub fn wkt_crs_type(wkt: &str) -> &'static str {
    let trimmed = wkt.trim_start();
    if trimmed.starts_with("GEOGCRS") || trimmed.starts_with("GEOGCS") {
        "geographic"
    } else if trimmed.starts_with("PROJCRS") || trimmed.starts_with("PROJCS") {
        "projected"
    } else {
        "unknown"
    }
}

/// The CRS's own name: the first quoted string in the WKT, which is always
/// the outermost GEOGCRS/PROJCRS's own name (never a nested BASEGEOGCRS's).
pub fn wkt_name(wkt: &str) -> String {
    let re = Regex::new(r#"^\w+\["([^"]*)""#).unwrap();
    re.captures(wkt.trim_start())
        .map(|c| c[1].to_string())
        .unwrap_or_default()
}

/// (name, semi_major_axis_m, inverse_flattening) from the WKT's ELLIPSOID
/// clause. A PROJCRS reuses its BASEGEOGCRS's single ellipsoid, so the first
/// (and only) ELLIPSOID match is correct for both geographic and projected CRS.
pub fn wkt_ellipsoid(wkt: &str) -> Option<(String, f64, f64)> {
    let re = Regex::new(r#"ELLIPSOID\["([^"]*)",\s*([0-9.eE+-]+),\s*([0-9.eE+-]+)"#).unwrap();
    re.captures(wkt).map(|c| {
        (
            c[1].to_string(),
            c[2].parse::<f64>().unwrap_or(0.0),
            c[3].parse::<f64>().unwrap_or(0.0),
        )
    })
}

/// The datum name: an ENSEMBLE (multi-realization datums like WGS 84) or a
/// classic single DATUM, whichever the WKT carries.
pub fn wkt_datum(wkt: &str) -> String {
    let re_ens = Regex::new(r#"ENSEMBLE\["([^"]*)""#).unwrap();
    if let Some(c) = re_ens.captures(wkt) {
        return c[1].to_string();
    }
    let re_datum = Regex::new(r#"DATUM\["([^"]*)""#).unwrap();
    re_datum
        .captures(wkt)
        .map(|c| c[1].to_string())
        .unwrap_or_default()
}

/// (unit_name, unit_to_base_conversion_factor) from the CRS's coordinate
/// system unit, read off its first AXIS clause's trailing ANGLEUNIT/LENGTHUNIT
/// (degrees->radians for a geographic CRS, or meters/etc for a projected one).
pub fn wkt_unit(wkt: &str) -> Option<(String, f64)> {
    let re = Regex::new(r#"(?:ANGLEUNIT|LENGTHUNIT)\["([^"]*)",\s*([0-9.eE+-]+)"#).unwrap();
    re.captures_iter(wkt)
        .last()
        .map(|c| (c[1].to_string(), c[2].parse::<f64>().unwrap_or(0.0)))
}

/// All (name, direction) AXIS entries the CRS's own coordinate system
/// declares, in ISO-19111 authority order (index 0 is the authority's first
/// axis — e.g. latitude for EPSG:4326, easting for most projected CRS).
pub fn wkt_axes(wkt: &str) -> Vec<(String, String)> {
    let re = Regex::new(r#"AXIS\["([^"]*)",\s*(\w+)"#).unwrap();
    re.captures_iter(wkt)
        .map(|c| (c[1].to_string(), c[2].to_string()))
        .collect()
}

/// (description, south_lat, west_lon, north_lat, east_lon) from the WKT's
/// USAGE[...AREA[...],BBOX[s,w,n,e]] clause -- the EPSG registry's own
/// documented area of applicability for this CRS.
pub fn wkt_area_of_use(wkt: &str) -> Option<(String, f64, f64, f64, f64)> {
    let re_area = Regex::new(r#"AREA\["([^"]*)"\]"#).unwrap();
    let desc = re_area.captures(wkt).map(|c| c[1].to_string())?;
    let re_bbox = Regex::new(r#"BBOX\[([^\]]*)\]"#).unwrap();
    let bbox = re_bbox.captures(wkt)?;
    let nums: Vec<f64> = bbox[1]
        .split(',')
        .filter_map(|s| s.trim().parse::<f64>().ok())
        .collect();
    if nums.len() != 4 {
        return None;
    }
    Some((desc, nums[0], nums[1], nums[2], nums[3]))
}

/// Transform one (x, y) point from `from` to `to`, in each CRS's own native
/// GIS-convention units: degrees for a geographic CRS (x=lon, y=lat), linear
/// units (meters) for a projected one. proj4rs itself works in radians for
/// geographic axes (matching PROJ's internal convention) -- this wrapper is
/// the one place that degree<->radian conversion happens, via `Proj::is_latlong`,
/// so every transform node in this package shares one correct implementation.
pub fn transform_xy(from: &Proj, to: &Proj, x: f64, y: f64) -> Result<(f64, f64), &'static str> {
    if !finite(x, y) {
        return Err("NON_FINITE_COORD");
    }
    let (px, py) = if from.is_latlong() {
        (x.to_radians(), y.to_radians())
    } else {
        (x, y)
    };
    let mut pt = (px, py, 0.0);
    proj4rs::transform::transform(from, to, &mut pt).map_err(|_| "TRANSFORM_FAILED")?;
    if !finite(pt.0, pt.1) {
        return Err("TRANSFORM_FAILED");
    }
    let (ox, oy) = if to.is_latlong() {
        (pt.0.to_degrees(), pt.1.to_degrees())
    } else {
        (pt.0, pt.1)
    };
    Ok((ox, oy))
}

/// Densify a bounding box's 4 edges into `BBOX_DENSIFY_POINTS` points each
/// (including corners), transform every sample through `proj`, and return
/// the reprojected min/max envelope. Mirrors PROJ's own proj_trans_bounds
/// technique: corner-only reprojection under-bounds a box whose edge bows
/// outward under the target projection.
pub fn densify_and_transform_bounds(
    from: &Proj,
    to: &Proj,
    min_x: f64,
    min_y: f64,
    max_x: f64,
    max_y: f64,
) -> Result<(f64, f64, f64, f64), &'static str> {
    let n = BBOX_DENSIFY_POINTS;
    let mut xs: Vec<f64> = Vec::with_capacity(n * 4);
    let mut ys: Vec<f64> = Vec::with_capacity(n * 4);
    for i in 0..n {
        let t = i as f64 / (n - 1) as f64;
        // bottom edge, top edge
        xs.push(min_x + t * (max_x - min_x));
        ys.push(min_y);
        xs.push(min_x + t * (max_x - min_x));
        ys.push(max_y);
        // left edge, right edge
        xs.push(min_x);
        ys.push(min_y + t * (max_y - min_y));
        xs.push(max_x);
        ys.push(min_y + t * (max_y - min_y));
    }
    let mut out_min_x = f64::INFINITY;
    let mut out_min_y = f64::INFINITY;
    let mut out_max_x = f64::NEG_INFINITY;
    let mut out_max_y = f64::NEG_INFINITY;
    let mut any_ok = false;
    for i in 0..xs.len() {
        if let Ok((ox, oy)) = transform_xy(from, to, xs[i], ys[i]) {
            any_ok = true;
            out_min_x = out_min_x.min(ox);
            out_min_y = out_min_y.min(oy);
            out_max_x = out_max_x.max(ox);
            out_max_y = out_max_y.max(oy);
        }
    }
    if !any_ok {
        return Err("TRANSFORM_FAILED");
    }
    Ok((out_min_x, out_min_y, out_max_x, out_max_y))
}
