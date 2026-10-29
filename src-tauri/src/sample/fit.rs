//! Fit notes: on a Sample of a Tech Pack Style, the measured value of each
//! Point of Measure in each size, shown beside the Tech Pack value and
//! tolerance (GLOSSARY: Fit Note).
//!
//! A Style on a Blank has no Point of Measure table, so it offers no fit
//! notes: [`sample_fit`] returns `None` and [`set_fit_note`] refuses. Fit
//! notes recorded while the base was a Tech Pack stay stored and come back
//! if the base returns to it.
//!
//! A Point of Measure or size that a fit note refers to cannot be removed
//! from the Tech Pack (see `tech_pack::remove_point_of_measure`); the
//! founder clears those fit notes first, so no measurement goes silently.

use rusqlite::{params, Connection};

use super::{get_sample, SampleId};
use crate::error::{CoreError, CoreResult};
use crate::library::Library;
use crate::tech_pack::{self, PointOfMeasureId, SizeId, TechPack, TechPackSize};

/// The fit notes of a Sample: the Tech Pack table with a measured value
/// beside each Tech Pack value.
#[derive(Debug, Clone, PartialEq, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SampleFit {
    pub sample_id: SampleId,
    /// The unit of every value and tolerance, from the Tech Pack.
    pub unit: String,
    /// The columns, in Tech Pack order.
    pub sizes: Vec<TechPackSize>,
    /// One row per Point of Measure, in Tech Pack order.
    pub rows: Vec<FitRow>,
}

/// One Point of Measure on a Sample.
#[derive(Debug, Clone, PartialEq, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct FitRow {
    pub point_of_measure_id: PointOfMeasureId,
    pub name: String,
    pub tolerance: f64,
    /// One entry per size, in the order of [`SampleFit::sizes`].
    pub cells: Vec<FitCell>,
}

/// One Point of Measure in one size on a Sample.
#[derive(Debug, Clone, PartialEq, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct FitCell {
    pub size_id: SizeId,
    /// The Tech Pack value, if the founder entered one.
    pub spec: Option<f64>,
    /// The value measured on the Sample, if recorded.
    pub measured: Option<f64>,
    /// `measured - spec`, when both are known.
    pub deviation: Option<f64>,
    /// True when the deviation is within the tolerance either way.
    pub within_tolerance: Option<bool>,
}

/// The fit notes of a Sample, or `None` when its Style's base is not a Tech
/// Pack.
pub fn sample_fit(lib: &Library, sample_id: SampleId) -> CoreResult<Option<SampleFit>> {
    let sample = get_sample(lib, sample_id)?;
    let Some(tp) = tech_pack::get_tech_pack(lib, sample.style_id)? else { return Ok(None) };
    let measured = read_fit_notes(lib.db(), sample_id)?;
    Ok(Some(build(sample_id, tp, &measured)))
}

/// Records (`Some`) or clears (`None`) the measured value of a Point of
/// Measure in one size on a Sample. The Point of Measure and the size must
/// be in the Tech Pack of the Sample's Style. Refuses on a Style whose base
/// is not a Tech Pack.
pub fn set_fit_note(
    lib: &Library,
    sample_id: SampleId,
    point_of_measure_id: PointOfMeasureId,
    size_id: SizeId,
    measured: Option<f64>,
) -> CoreResult<SampleFit> {
    let sample = get_sample(lib, sample_id)?;
    let tp = tech_pack::get_tech_pack(lib, sample.style_id)?.ok_or_else(|| CoreError::InvalidInput {
        reason: "fit notes are only for Styles built on a Tech Pack".into(),
    })?;
    if !tp.points_of_measure.iter().any(|p| p.id == point_of_measure_id) {
        return Err(CoreError::InvalidInput { reason: "that Point of Measure is not in this Style's Tech Pack".into() });
    }
    if !tp.sizes.iter().any(|z| z.id == size_id) {
        return Err(CoreError::InvalidInput { reason: "that size is not in this Style's Tech Pack".into() });
    }
    if measured.is_some_and(|v| !v.is_finite()) {
        return Err(CoreError::InvalidInput { reason: "a measured value must be a number".into() });
    }
    let conn = lib.writer()?;
    match measured {
        Some(v) => conn.execute(
            "INSERT INTO sample_fit (sample_id, pom_id, size_id, measured) VALUES (?1, ?2, ?3, ?4)
             ON CONFLICT (sample_id, pom_id, size_id) DO UPDATE SET measured = excluded.measured",
            params![sample_id, point_of_measure_id, size_id, v],
        )?,
        None => conn.execute(
            "DELETE FROM sample_fit WHERE sample_id = ?1 AND pom_id = ?2 AND size_id = ?3",
            params![sample_id, point_of_measure_id, size_id],
        )?,
    };
    let measured = read_fit_notes(lib.db(), sample_id)?;
    Ok(build(sample_id, tp, &measured))
}

/// The Sample rounds (oldest first) that hold fit notes for a Point of
/// Measure. The Tech Pack uses this to refuse removing it.
pub(crate) fn rounds_measuring_point_of_measure(conn: &Connection, id: PointOfMeasureId) -> CoreResult<Vec<u32>> {
    rounds_where(conn, "f.pom_id = ?1", id)
}

/// The Sample rounds (oldest first) that hold fit notes in a size. The Tech
/// Pack uses this to refuse removing it.
pub(crate) fn rounds_measuring_size(conn: &Connection, id: SizeId) -> CoreResult<Vec<u32>> {
    rounds_where(conn, "f.size_id = ?1", id)
}

fn rounds_where(conn: &Connection, filter: &str, id: i64) -> CoreResult<Vec<u32>> {
    let mut stmt = conn.prepare(&format!(
        "SELECT DISTINCT s.round FROM sample_fit f JOIN sample s ON s.id = f.sample_id WHERE {filter} ORDER BY s.round"
    ))?;
    let rounds = stmt.query_map([id], |r| r.get(0))?.collect::<Result<_, _>>()?;
    Ok(rounds)
}

fn read_fit_notes(conn: &Connection, sample_id: SampleId) -> CoreResult<Vec<(PointOfMeasureId, SizeId, f64)>> {
    let mut stmt = conn.prepare("SELECT pom_id, size_id, measured FROM sample_fit WHERE sample_id = ?1")?;
    let rows = stmt.query_map([sample_id], |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?)))?.collect::<Result<_, _>>()?;
    Ok(rows)
}

fn build(sample_id: SampleId, tp: TechPack, measured: &[(PointOfMeasureId, SizeId, f64)]) -> SampleFit {
    let rows = tp
        .points_of_measure
        .iter()
        .map(|point_of_measure| FitRow {
            point_of_measure_id: point_of_measure.id,
            name: point_of_measure.name.clone(),
            tolerance: point_of_measure.tolerance,
            cells: tp
                .sizes
                .iter()
                .zip(&point_of_measure.values)
                .map(|(size, &spec)| {
                    let measured = measured
                        .iter()
                        .find(|(p, z, _)| *p == point_of_measure.id && *z == size.id)
                        .map(|&(_, _, v)| v);
                    let deviation = spec.zip(measured).map(|(s, m)| m - s);
                    // A small epsilon so that, for example, 57.0 against
                    // 56.0 with tolerance 1.0 counts as within.
                    let within_tolerance = deviation.map(|d| d.abs() <= point_of_measure.tolerance + 1e-9);
                    FitCell { size_id: size.id, spec, measured, deviation, within_tolerance }
                })
                .collect(),
        })
        .collect();
    SampleFit { sample_id, unit: tp.details.unit, sizes: tp.sizes, rows }
}
