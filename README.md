# crs-tools

Deterministic, offline coordinate-reference-system (CRS) and map-projection
transforms plus CRS metadata introspection, for the [Axiom](https://axiom.dev)
marketplace (handle `christiangeorgelucas`).

Converts coordinates between coordinate reference systems (EPSG codes),
reprojects bounding boxes, converts between WGS-84 lon/lat and Web Mercator,
identifies the UTM zone for a point, and looks up structured metadata about a
CRS — name, type, datum, ellipsoid, unit, authority-defined axis order, area
of applicability, and WKT2/PROJ4 representations.

## Why this exists

GIS work regularly needs to move a coordinate between coordinate systems
(e.g. WGS-84 lon/lat to a projected CRS for accurate local measurement, or to
Web Mercator for web map tiles) and to know a CRS's own properties (is
`EPSG:4326`'s axis order lat-then-lon or lon-then-lat? what's its area of
applicability?). This package wraps that as a set of pure, stateless nodes.

It is distinct from three other packages in this marketplace that touch
geospatial data: `geo-tools` (WGS-84 geodesic distance/bearing/area — no CRS
or projection machinery), `geo-encoding-tools` (geohash and dedicated
lat/lon<->UTM/MGRS coordinate *conversion*, not generic EPSG reprojection),
and `geometry-tools` (planar 2D computational geometry, no CRS awareness).

## Nodes

| Node | Does |
|---|---|
| `TransformCoordinate` | Transform a single (x, y) coordinate between two CRS |
| `TransformCoordinates` | Transform a batch of up to 5,000 coordinates between two CRS |
| `LonLatToWebMercator` | WGS-84 lon/lat (EPSG:4326) → Web Mercator meters (EPSG:3857) |
| `WebMercatorToLonLat` | Web Mercator meters → WGS-84 lon/lat |
| `GetUTMZone` | UTM zone/hemisphere/EPSG code covering a lon/lat point |
| `GetCRSInfo` | Name, type, datum, ellipsoid, unit, area of use for a CRS |
| `GetCRSRepresentation` | WKT2 (ISO 19162) and PROJ4-string representations of a CRS |
| `ValidateCRS` | Whether an EPSG code / PROJ4 string / EPSG-tagged WKT is recognized |
| `GetCRSAxisInfo` | A CRS's authority-defined axis order (the lat/lon-vs-lon/lat footgun) |
| `ParseCRS` | Normalize a CRS identifier into its canonical registry form |
| `ReprojectBoundingBox` | Reproject a bounding box between two CRS, edge-densified |

## Implementation

Wraps [`proj4rs`](https://github.com/3liz/proj4rs) (pure Rust, MIT OR
Apache-2.0) and [`crs-definitions`](https://github.com/frewsxcv/crs-definitions)
(CC0-1.0), a zero-dependency embedded EPSG registry of PROJ4/WKT2
definitions — a fully permissive-licensed, pure-Rust path to PROJ's
projection math without pulling in PROJ's own C library or pyproj's Python
binding (whose runtime dependency closure carries an MPL-2.0-licensed
component, `certifi`).

Offline and deterministic: no network grid downloads are ever performed —
this package's dependencies have no such capability to begin with.

Built for the Axiom marketplace.

## License

MIT — see [LICENSE](./LICENSE).
