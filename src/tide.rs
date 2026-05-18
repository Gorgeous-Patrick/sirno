//! Dependency review worklists for Sirno Lake edits.
//!
//! Tide compares the current public lake against the latest Sirno Frost snapshot.
//! It derives review obligations from configured structural edge policies.

use std::collections::{BTreeMap, BTreeSet};
use std::fmt;
use std::str::FromStr;

use serde::{Deserialize, Serialize, de};
use thiserror::Error;

use crate::entry::{Entry, EntryRenderError};
use crate::id::{EntryId, EntryIdError};
use crate::render::{GeneratedLinkBody, GeneratedLinkError};
use crate::structural::{
    StructuralEdgeDirection, StructuralEdgeDirectionParseError, StructuralEdgeIndex,
    StructuralSettings,
};

/// One side that can produce a tide workitem.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum TideSource {
    /// The current public lake.
    Lake,
    /// The latest Sirno Frost snapshot.
    Frost,
}

/// One dependency review obligation.
///
/// Invariant: the tuple `(ripple, field, direction, neighbor)` fully identifies the workitem.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize)]
// sirno:witness:tide-workitem:begin
pub struct TideWorkitem {
    /// Changed entry that produced the obligation.
    pub ripple: EntryId,
    /// Structural field that produced the obligation.
    pub field: String,
    /// Structural edge direction that produced the obligation.
    pub direction: StructuralEdgeDirection,
    /// Entry that must be reviewed.
    pub neighbor: EntryId,
}

impl TideWorkitem {
    /// Construct a validated workitem tuple.
    pub fn new(
        ripple: EntryId, field: impl Into<String>, direction: StructuralEdgeDirection,
        neighbor: EntryId,
    ) -> Result<Self, TideWorkitemParseError> {
        let field = field.into();
        validate_field(&field)?;
        Ok(Self { ripple, field, direction, neighbor })
    }
}

impl fmt::Display for TideWorkitem {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{},{},{},{}", self.ripple, self.field, self.direction, self.neighbor)
    }
}
// sirno:witness:tide-workitem:end

impl<'de> Deserialize<'de> for TideWorkitem {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        #[derive(Deserialize)]
        struct RawWorkitem {
            ripple: EntryId,
            field: String,
            direction: StructuralEdgeDirection,
            neighbor: EntryId,
        }

        let raw = RawWorkitem::deserialize(deserializer)?;
        Self::new(raw.ripple, raw.field, raw.direction, raw.neighbor).map_err(de::Error::custom)
    }
}

// sirno:witness:tide-workitem:begin
impl FromStr for TideWorkitem {
    type Err = TideWorkitemParseError;

    fn from_str(raw: &str) -> Result<Self, Self::Err> {
        let parts = raw.split(',').collect::<Vec<_>>();
        if parts.len() != 4 {
            return Err(TideWorkitemParseError::TupleShape);
        }
        Self::new(
            EntryId::new(parts[0])?,
            parts[1].to_owned(),
            parts[2].parse()?,
            EntryId::new(parts[3])?,
        )
    }
}
// sirno:witness:tide-workitem:end

/// One persisted tide resolution.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize)]
pub struct TideResolution {
    /// Changed entry that produced the obligation.
    pub ripple: EntryId,
    /// Structural field that produced the obligation.
    pub field: String,
    /// Structural edge direction that produced the obligation.
    pub direction: StructuralEdgeDirection,
    /// Entry that was reviewed.
    pub neighbor: EntryId,
    /// Fingerprint of the ripple delta that was reviewed.
    pub fingerprint: String,
}

impl TideResolution {
    fn from_status(status: &TideStatus) -> Self {
        Self {
            ripple: status.workitem.ripple.clone(),
            field: status.workitem.field.clone(),
            direction: status.workitem.direction,
            neighbor: status.workitem.neighbor.clone(),
            fingerprint: status.fingerprint.clone(),
        }
    }
}

impl<'de> Deserialize<'de> for TideResolution {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        #[derive(Deserialize)]
        struct RawResolution {
            ripple: EntryId,
            field: String,
            direction: StructuralEdgeDirection,
            neighbor: EntryId,
            fingerprint: String,
        }

        let raw = RawResolution::deserialize(deserializer)?;
        validate_field(&raw.field).map_err(de::Error::custom)?;
        Ok(Self {
            ripple: raw.ripple,
            field: raw.field,
            direction: raw.direction,
            neighbor: raw.neighbor,
            fingerprint: raw.fingerprint,
        })
    }
}

/// Display status for one tide workitem.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct TideStatus {
    /// Full workitem tuple.
    pub workitem: TideWorkitem,
    /// Waterline or frostline sources that produced this workitem.
    pub sources: BTreeSet<TideSource>,
    /// Fingerprint of the ripple delta this status reviews.
    pub fingerprint: String,
    /// Whether a matching resolution exists in the lock.
    pub resolved: bool,
}

/// Derived tide state.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Tide {
    statuses: Vec<TideStatus>,
    ripple_ids: BTreeSet<EntryId>,
}

impl Tide {
    /// Derive tide state from frostline entries, waterline entries, and persisted resolutions.
    pub fn from_entries(
        frostline: &[Entry], waterline: &[Entry], settings: &StructuralSettings,
        resolutions: &[TideResolution],
    ) -> Result<Self, TideError> {
        let frostline = normalized_entries(frostline)?;
        let waterline = normalized_entries(waterline)?;
        let frost_by_id = entries_by_id(&frostline);
        let water_by_id = entries_by_id(&waterline);
        let mut ripple_ids = BTreeSet::new();

        for id in frost_by_id.keys().chain(water_by_id.keys()) {
            if frost_by_id.get(id) != water_by_id.get(id) {
                ripple_ids.insert((*id).clone());
            }
        }

        let frost_index = StructuralEdgeIndex::from_entries(&frostline);
        let water_index = StructuralEdgeIndex::from_entries(&waterline);
        let mut sources_by_workitem = BTreeMap::<TideWorkitem, BTreeSet<TideSource>>::new();
        let mut fingerprint_by_ripple = BTreeMap::<EntryId, String>::new();

        // sirno:witness:wave:begin
        for ripple in &ripple_ids {
            let fingerprint = ripple_fingerprint(frost_by_id.get(ripple), water_by_id.get(ripple))?;
            fingerprint_by_ripple.insert(ripple.clone(), fingerprint);
            for (field, field_settings) in settings.fields() {
                for direction in StructuralEdgeDirection::ORDER {
                    let edge = field_settings.edge(direction);
                    if edge.ripple.lake
                        && let Some(entry) = water_by_id.get(ripple)
                    {
                        insert_workitems(
                            &mut sources_by_workitem,
                            ripple,
                            field,
                            direction,
                            TideSource::Lake,
                            water_index.edge_targets(field, direction, entry),
                        )?;
                    }
                    if edge.ripple.frost
                        && let Some(entry) = frost_by_id.get(ripple)
                    {
                        insert_workitems(
                            &mut sources_by_workitem,
                            ripple,
                            field,
                            direction,
                            TideSource::Frost,
                            frost_index.edge_targets(field, direction, entry),
                        )?;
                    }
                }
            }
        }
        // sirno:witness:wave:end

        let mut statuses = sources_by_workitem
            .into_iter()
            .map(|(workitem, sources)| {
                let fingerprint = fingerprint_by_ripple
                    .get(&workitem.ripple)
                    .expect("workitem ripple has fingerprint")
                    .clone();
                let resolved = resolutions
                    .iter()
                    .any(|resolution| resolution.matches_status_parts(&workitem, &fingerprint));
                TideStatus { workitem, sources, fingerprint, resolved }
            })
            .collect::<Vec<_>>();
        statuses.sort_by(|left, right| left.workitem.cmp(&right.workitem));

        Ok(Self { statuses, ripple_ids })
    }

    /// All current workitem statuses.
    pub fn statuses(&self) -> &[TideStatus] {
        &self.statuses
    }

    /// Current ripple entry ids.
    pub fn ripple_ids(&self) -> &BTreeSet<EntryId> {
        &self.ripple_ids
    }

    /// Open workitem statuses.
    pub fn open_statuses(&self) -> impl Iterator<Item = &TideStatus> {
        self.statuses.iter().filter(|status| !status.resolved)
    }

    /// Entry ids that still need dependency review.
    pub fn review_entries(&self) -> Vec<EntryId> {
        self.open_statuses()
            .map(|status| status.workitem.neighbor.clone())
            .collect::<BTreeSet<_>>()
            .into_iter()
            .collect()
    }

    /// Returns true when no open workitem remains.
    pub fn is_clear(&self) -> bool {
        self.open_statuses().next().is_none()
    }

    /// Current matching resolutions.
    pub fn active_resolutions(&self) -> Vec<TideResolution> {
        self.statuses
            .iter()
            .filter(|status| status.resolved)
            .map(TideResolution::from_status)
            .collect()
    }

    /// Resolve open statuses that satisfy `predicate`.
    pub fn resolve_where(
        &self, predicate: impl Fn(&TideStatus) -> bool,
    ) -> (Vec<TideResolution>, usize) {
        let mut resolutions = self.active_resolutions();
        let mut added = 0;
        for status in self.open_statuses().filter(|status| predicate(status)) {
            resolutions.push(TideResolution::from_status(status));
            added += 1;
        }
        resolutions.sort();
        resolutions.dedup();
        (resolutions, added)
    }

    /// Reopen matching active statuses.
    pub fn reopen_where(
        &self, predicate: impl Fn(&TideStatus) -> bool,
    ) -> (Vec<TideResolution>, usize) {
        let mut removed = 0;
        let mut resolutions = Vec::new();
        for status in self.statuses.iter().filter(|status| status.resolved) {
            if predicate(status) {
                removed += 1;
            } else {
                resolutions.push(TideResolution::from_status(status));
            }
        }
        (resolutions, removed)
    }
}

impl TideResolution {
    fn matches_status_parts(&self, workitem: &TideWorkitem, fingerprint: &str) -> bool {
        self.ripple == workitem.ripple
            && self.field == workitem.field
            && self.direction == workitem.direction
            && self.neighbor == workitem.neighbor
            && self.fingerprint == fingerprint
    }
}

fn normalized_entries(entries: &[Entry]) -> Result<Vec<Entry>, TideError> {
    entries.iter().map(normalized_entry).collect()
}

fn normalized_entry(entry: &Entry) -> Result<Entry, TideError> {
    let body = GeneratedLinkBody::new(&entry.body).delete()?;
    let body = strip_trailing_generated_link_divider(&body);
    Ok(Entry::new(entry.id.clone(), entry.metadata.clone(), body))
}

fn strip_trailing_generated_link_divider(body: &str) -> String {
    body.strip_suffix("\n\n---\n")
        .map(|before| format!("{before}\n"))
        .unwrap_or_else(|| body.to_owned())
}

fn entries_by_id(entries: &[Entry]) -> BTreeMap<EntryId, &Entry> {
    entries.iter().map(|entry| (entry.id.clone(), entry)).collect()
}

fn insert_workitems(
    sources_by_workitem: &mut BTreeMap<TideWorkitem, BTreeSet<TideSource>>, ripple: &EntryId,
    field: &str, direction: StructuralEdgeDirection, source: TideSource,
    neighbors: BTreeSet<EntryId>,
) -> Result<(), TideError> {
    for neighbor in neighbors {
        let workitem = TideWorkitem::new(ripple.clone(), field.to_owned(), direction, neighbor)?;
        sources_by_workitem.entry(workitem).or_default().insert(source);
    }
    Ok(())
}

fn ripple_fingerprint(
    frostline: Option<&&Entry>, waterline: Option<&&Entry>,
) -> Result<String, TideError> {
    let mut source = String::new();
    push_fingerprint_entry(&mut source, "frost", frostline.copied())?;
    push_fingerprint_entry(&mut source, "lake", waterline.copied())?;
    Ok(format!("{:016x}", fnv1a64(source.as_bytes())))
}

fn push_fingerprint_entry(
    out: &mut String, label: &str, entry: Option<&Entry>,
) -> Result<(), EntryRenderError> {
    out.push_str(label);
    out.push('\n');
    if let Some(entry) = entry {
        out.push_str(&entry.to_markdown()?);
    } else {
        out.push_str("(absent)\n");
    }
    out.push('\n');
    Ok(())
}

fn fnv1a64(bytes: &[u8]) -> u64 {
    const OFFSET: u64 = 0xcbf29ce484222325;
    const PRIME: u64 = 0x100000001b3;
    let mut hash = OFFSET;
    for byte in bytes {
        hash ^= u64::from(*byte);
        hash = hash.wrapping_mul(PRIME);
    }
    hash
}

fn validate_field(field: &str) -> Result<(), TideWorkitemParseError> {
    if field.is_empty() || field.contains('\n') || field.contains('\r') || field.contains(',') {
        return Err(TideWorkitemParseError::InvalidField(field.to_owned()));
    }
    Ok(())
}

/// Error raised while parsing a workitem tuple.
#[derive(Debug, Error)]
pub enum TideWorkitemParseError {
    /// The text does not contain exactly four comma-separated tuple fields.
    #[error("expected RIPPLE,FIELD,DIRECTION,NEIGHBOR")]
    TupleShape,
    /// A structural field cannot be used in a workitem tuple.
    #[error("structural field must be non-empty and cannot contain comma or line breaks: {0}")]
    InvalidField(String),
    /// Entry id parsing failed.
    #[error(transparent)]
    EntryId(#[from] EntryIdError),
    /// Direction parsing failed.
    #[error(transparent)]
    Direction(#[from] StructuralEdgeDirectionParseError),
}

/// Error raised while deriving tide state.
#[derive(Debug, Error)]
pub enum TideError {
    /// Generated footer boundaries were malformed during normalization.
    #[error(transparent)]
    GeneratedLink(#[from] GeneratedLinkError),
    /// Entry rendering failed during fingerprinting.
    #[error(transparent)]
    EntryRender(#[from] EntryRenderError),
    /// Workitem construction failed.
    #[error(transparent)]
    Workitem(#[from] TideWorkitemParseError),
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        EntryMetadata, StructuralEdgeSettings, StructuralFieldSettings, StructuralRippleSettings,
    };

    fn id(raw: &str) -> EntryId {
        EntryId::new(raw).unwrap()
    }

    fn entry(raw_id: &str) -> Entry {
        Entry::new(
            id(raw_id),
            EntryMetadata::new(raw_id, "desc").unwrap(),
            format!("{raw_id} body.\n"),
        )
    }

    fn belongs_settings(lake: bool, frost: bool) -> StructuralSettings {
        StructuralSettings::from_fields([(
            "belongs",
            StructuralFieldSettings::new(
                StructuralEdgeSettings::new(false, StructuralRippleSettings::new(lake, frost)),
                StructuralEdgeSettings::new(false, StructuralRippleSettings::new(lake, frost)),
                StructuralEdgeSettings::default(),
            ),
        )])
    }

    #[test]
    fn derives_workitems_from_configured_sources() {
        let mut old = entry("ripple");
        old.metadata.push_structural_target("belongs", id("old-neighbor"));
        let mut new = entry("ripple");
        new.body = "changed body.\n".to_owned();
        new.metadata.push_structural_target("belongs", id("new-neighbor"));

        let tide = Tide::from_entries(&[old], &[new], &belongs_settings(true, true), &[]).unwrap();
        let workitems =
            tide.statuses().iter().map(|status| status.workitem.to_string()).collect::<Vec<_>>();

        assert_eq!(workitems, ["ripple,belongs,to,new-neighbor", "ripple,belongs,to,old-neighbor"]);
    }

    #[test]
    fn review_entries_are_deduplicated_open_neighbors() {
        let mut old = entry("ripple");
        old.metadata.push_structural_target("belongs", id("neighbor"));
        let mut new = old.clone();
        new.body = "changed body.\n".to_owned();

        let tide = Tide::from_entries(&[old], &[new], &belongs_settings(true, true), &[]).unwrap();
        let entries =
            tide.review_entries().into_iter().map(|id| id.to_string()).collect::<Vec<_>>();

        assert_eq!(entries, ["neighbor"]);
    }

    #[test]
    fn matching_resolution_marks_workitem_resolved() {
        let mut old = entry("ripple");
        old.metadata.push_structural_target("belongs", id("neighbor"));
        let mut new = old.clone();
        new.body = "changed body.\n".to_owned();
        let open = Tide::from_entries(
            std::slice::from_ref(&old),
            std::slice::from_ref(&new),
            &belongs_settings(true, false),
            &[],
        )
        .unwrap();
        let resolution = TideResolution::from_status(&open.statuses()[0]);

        let resolved =
            Tide::from_entries(&[old], &[new], &belongs_settings(true, false), &[resolution])
                .unwrap();

        assert!(resolved.statuses()[0].resolved);
    }

    #[test]
    fn changed_ripple_fingerprint_reopens_resolution() {
        let mut old = entry("ripple");
        old.metadata.push_structural_target("belongs", id("neighbor"));
        let mut new = old.clone();
        new.body = "changed body.\n".to_owned();
        let open = Tide::from_entries(
            std::slice::from_ref(&old),
            std::slice::from_ref(&new),
            &belongs_settings(true, false),
            &[],
        )
        .unwrap();
        let resolution = TideResolution::from_status(&open.statuses()[0]);
        new.body = "changed again.\n".to_owned();

        let reopened =
            Tide::from_entries(&[old], &[new], &belongs_settings(true, false), &[resolution])
                .unwrap();

        assert!(!reopened.statuses()[0].resolved);
    }

    #[test]
    fn infer_can_resolve_deleted_ripple_neighbors() {
        let mut old = entry("ripple");
        old.metadata.push_structural_target("belongs", id("deleted-neighbor"));
        let deleted_neighbor = entry("deleted-neighbor");
        let mut new = old.clone();
        new.body = "changed body.\n".to_owned();

        let settings = StructuralSettings::from_fields([(
            "belongs",
            StructuralFieldSettings::new(
                StructuralEdgeSettings::new(false, StructuralRippleSettings::new(false, true)),
                StructuralEdgeSettings::default(),
                StructuralEdgeSettings::default(),
            ),
        )]);
        let tide = Tide::from_entries(&[old, deleted_neighbor], &[new], &settings, &[]).unwrap();
        let (_, count) =
            tide.resolve_where(|status| tide.ripple_ids().contains(&status.workitem.neighbor));

        assert_eq!(count, 1);
    }

    #[test]
    fn json_workitem_rejects_comma_field() {
        let error = serde_json::from_str::<TideWorkitem>(
            r#"{"ripple":"ripple","field":"bad,field","direction":"to","neighbor":"neighbor"}"#,
        )
        .unwrap_err();

        assert!(error.to_string().contains("comma"));
    }
}
