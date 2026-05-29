//! Dependency review worklists for lake edits.
//!
//! Tide compares the current lake against the accepted anchor baseline.
//! It derives review obligations from structural relation entries.

use std::collections::{BTreeMap, BTreeSet};
use std::fmt;
use std::str::FromStr;

use serde::{Deserialize, Serialize, de};
use sha2::Digest;
use thiserror::Error;

use crate::anchor::{AnchorEntry, AnchorError, entry_fingerprint};
use crate::entry::{Entry, EntryMetadata};
use crate::identifier::{EntryAddress, EntryAddressError};
use crate::render::{GeneratedLinkBody, GeneratedLinkError};
use crate::structural::{
    StructuralEdgeDirection, StructuralEdgeDirectionParseError, StructuralEdgeIndex,
    StructuralSettings,
};

/// One side that can produce a tide workitem.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum TideSource {
    /// The current lake.
    Lake,
    /// The accepted anchor baseline.
    Anchor,
}

/// One dependency review obligation.
///
/// Invariant: the tuple `(ripple, field, direction, neighbor)` fully identifies the workitem.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize)]
// sirno:witness:tide-workitem:begin
pub struct TideWorkitem {
    /// Changed entry that produced the obligation.
    pub ripple: EntryAddress,
    /// Link relation that produced the obligation.
    pub field: String,
    /// Structural link direction that produced the obligation.
    pub direction: StructuralEdgeDirection,
    /// Entry that must be reviewed.
    pub neighbor: EntryAddress,
}

impl TideWorkitem {
    /// Construct a validated workitem tuple.
    pub fn new(
        ripple: EntryAddress, field: impl Into<String>, direction: StructuralEdgeDirection,
        neighbor: EntryAddress,
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
            ripple: EntryAddress,
            field: String,
            direction: StructuralEdgeDirection,
            neighbor: EntryAddress,
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
            EntryAddress::new(parts[0])?,
            parts[1].to_owned(),
            parts[2].parse()?,
            EntryAddress::new(parts[3])?,
        )
    }
}
// sirno:witness:tide-workitem:end

/// One persisted tide resolution.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize)]
pub struct TideResolution {
    /// Changed entry that produced the obligation.
    pub ripple: EntryAddress,
    /// Link relation that produced the obligation.
    pub field: String,
    /// Structural link direction that produced the obligation.
    pub direction: StructuralEdgeDirection,
    /// Entry that was reviewed.
    pub neighbor: EntryAddress,
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
            ripple: EntryAddress,
            field: String,
            direction: StructuralEdgeDirection,
            neighbor: EntryAddress,
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
    /// Waterline or Anchor sources that produced this workitem.
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
    ripple_ids: BTreeSet<EntryAddress>,
}

/// Entry information needed to derive Tide without storing full baseline prose.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TideEntrySnapshot {
    /// Entry address.
    pub id: EntryAddress,
    /// Canonical entry fingerprint.
    pub fingerprint: String,
    /// Entry shape used for structural edge traversal.
    pub entry: Entry,
}

impl TideEntrySnapshot {
    /// Build a snapshot from a parsed entry.
    pub fn from_entry(entry: &Entry) -> Result<Self, TideError> {
        let entry = normalized_entry(entry)?;
        Ok(Self { id: entry.id.clone(), fingerprint: entry_fingerprint(&entry)?, entry })
    }

    /// Build a snapshot from one anchor entry record.
    pub fn from_anchor_entry(id: EntryAddress, record: &AnchorEntry) -> Result<Self, TideError> {
        let mut metadata = EntryMetadata::new(id.to_string(), "Anchor baseline entry.")?;
        for (field, targets) in &record.structural {
            metadata.set_structural_targets(field.clone(), targets.clone());
        }
        let entry = Entry::new(id.clone(), metadata, "");
        Ok(Self { id, fingerprint: record.fingerprint.clone(), entry })
    }
}

impl Tide {
    /// Derive tide state from baseline entries, waterline entries, and persisted resolutions.
    pub fn from_entries(
        anchor: &[Entry], waterline: &[Entry], settings: &StructuralSettings,
        resolutions: &[TideResolution],
    ) -> Result<Self, TideError> {
        let anchor = snapshots_from_entries(anchor)?;
        let waterline = snapshots_from_entries(waterline)?;
        Self::from_snapshots(&anchor, &waterline, settings, resolutions)
    }

    /// Derive tide state from compact baseline and waterline snapshots.
    pub fn from_snapshots(
        anchor: &[TideEntrySnapshot], waterline: &[TideEntrySnapshot],
        settings: &StructuralSettings, resolutions: &[TideResolution],
    ) -> Result<Self, TideError> {
        let anchor_by_id = snapshots_by_id(anchor);
        let water_by_id = snapshots_by_id(waterline);
        let mut ripple_ids = BTreeSet::new();

        for id in anchor_by_id.keys().chain(water_by_id.keys()) {
            if anchor_by_id.get(id).map(|snapshot| &snapshot.fingerprint)
                != water_by_id.get(id).map(|snapshot| &snapshot.fingerprint)
            {
                ripple_ids.insert((*id).clone());
            }
        }

        let anchor_entries = snapshot_entries(anchor);
        let water_entries = snapshot_entries(waterline);
        let anchor_index = StructuralEdgeIndex::from_entries(&anchor_entries);
        let water_index = StructuralEdgeIndex::from_entries(&water_entries);
        let mut sources_by_workitem = BTreeMap::<TideWorkitem, BTreeSet<TideSource>>::new();
        let mut fingerprint_by_ripple = BTreeMap::<EntryAddress, String>::new();

        // sirno:witness:wave:begin
        for ripple in &ripple_ids {
            let fingerprint = ripple_fingerprint(anchor_by_id.get(ripple), water_by_id.get(ripple));
            fingerprint_by_ripple.insert(ripple.clone(), fingerprint);
            for (field, field_settings) in settings.fields() {
                for direction in StructuralEdgeDirection::ORDER {
                    let edge = field_settings.edge(direction);
                    if edge.ripple.lake
                        && let Some(snapshot) = water_by_id.get(ripple)
                    {
                        insert_workitems(
                            &mut sources_by_workitem,
                            ripple,
                            field,
                            direction,
                            TideSource::Lake,
                            water_index.edge_targets(field, direction, &snapshot.entry),
                        )?;
                    }
                    if edge.ripple.anchor
                        && let Some(snapshot) = anchor_by_id.get(ripple)
                    {
                        insert_workitems(
                            &mut sources_by_workitem,
                            ripple,
                            field,
                            direction,
                            TideSource::Anchor,
                            anchor_index.edge_targets(field, direction, &snapshot.entry),
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

    /// Current ripple entry addresses.
    pub fn ripple_ids(&self) -> &BTreeSet<EntryAddress> {
        &self.ripple_ids
    }

    /// Open workitem statuses.
    pub fn open_statuses(&self) -> impl Iterator<Item = &TideStatus> {
        self.statuses.iter().filter(|status| !status.resolved)
    }

    /// Entry addresses that still need dependency review.
    pub fn review_entries(&self) -> Vec<EntryAddress> {
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

fn snapshots_from_entries(entries: &[Entry]) -> Result<Vec<TideEntrySnapshot>, TideError> {
    entries.iter().map(TideEntrySnapshot::from_entry).collect()
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

fn snapshots_by_id(entries: &[TideEntrySnapshot]) -> BTreeMap<EntryAddress, &TideEntrySnapshot> {
    entries.iter().map(|entry| (entry.id.clone(), entry)).collect()
}

fn snapshot_entries(entries: &[TideEntrySnapshot]) -> Vec<Entry> {
    entries.iter().map(|snapshot| snapshot.entry.clone()).collect()
}

fn insert_workitems(
    sources_by_workitem: &mut BTreeMap<TideWorkitem, BTreeSet<TideSource>>, ripple: &EntryAddress,
    field: &str, direction: StructuralEdgeDirection, source: TideSource,
    neighbors: BTreeSet<EntryAddress>,
) -> Result<(), TideError> {
    for neighbor in neighbors {
        let workitem = TideWorkitem::new(ripple.clone(), field.to_owned(), direction, neighbor)?;
        sources_by_workitem.entry(workitem).or_default().insert(source);
    }
    Ok(())
}

fn ripple_fingerprint(
    anchor: Option<&&TideEntrySnapshot>, waterline: Option<&&TideEntrySnapshot>,
) -> String {
    let mut source = String::new();
    push_fingerprint_entry(&mut source, "anchor", anchor.copied());
    push_fingerprint_entry(&mut source, "lake", waterline.copied());
    format!("sha256:{:x}", sha2::Sha256::digest(source.as_bytes()))
}

fn push_fingerprint_entry(out: &mut String, label: &str, entry: Option<&TideEntrySnapshot>) {
    out.push_str(label);
    out.push('\n');
    if let Some(snapshot) = entry {
        out.push_str(&snapshot.fingerprint);
        out.push('\n');
    } else {
        out.push_str("(absent)\n");
    }
    out.push('\n');
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
    /// A link relation cannot be used in a workitem tuple.
    #[error("link relation must be non-empty and cannot contain comma or line breaks: {0}")]
    InvalidField(String),
    /// Entry address parsing failed.
    #[error(transparent)]
    EntryAddress(#[from] EntryAddressError),
    /// Direction parsing failed.
    #[error(transparent)]
    Direction(#[from] StructuralEdgeDirectionParseError),
}

/// Error raised while deriving tide state.
#[derive(Debug, Error)]
pub enum TideError {
    /// Anchor canonicalization failed.
    #[error(transparent)]
    Anchor(#[from] AnchorError),
    /// Generated footer boundaries were malformed during normalization.
    #[error(transparent)]
    GeneratedLink(#[from] GeneratedLinkError),
    /// Entry metadata construction failed during snapshot reconstruction.
    #[error(transparent)]
    EntryParse(#[from] crate::EntryParseError),
    /// Workitem construction failed.
    #[error(transparent)]
    Workitem(#[from] TideWorkitemParseError),
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        EntryMetaType, EntryMetadata, StructuralEdgeSettings, StructuralFieldSettings,
        StructuralRippleSettings, StructuralTideSettings,
    };

    fn id(raw: &str) -> EntryAddress {
        EntryAddress::new(raw).unwrap()
    }

    fn entry(raw_id: &str) -> Entry {
        Entry::new(
            id(raw_id),
            EntryMetadata::new(raw_id, "desc").unwrap(),
            format!("{raw_id} body.\n"),
        )
    }

    fn belongs_settings(lake: bool, anchor: bool) -> StructuralSettings {
        StructuralSettings::from_fields([(
            "belongs",
            StructuralFieldSettings::new(
                StructuralEdgeSettings::new(false, StructuralRippleSettings::new(lake, anchor)),
                StructuralEdgeSettings::new(false, StructuralRippleSettings::new(lake, anchor)),
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
    fn derives_workitems_from_entry_defined_tide_policy() {
        let mut old = entry("ripple");
        old.metadata.push_structural_target("belongs", id("old-neighbor"));
        let mut new = entry("ripple");
        new.metadata.push_structural_target("belongs", id("new-neighbor"));
        let mut relation = entry("belongs");
        relation.metadata.meta.entry_type = Some(EntryMetaType::Structural);
        relation.metadata.meta.tide = Some(StructuralTideSettings::new(
            StructuralRippleSettings::new(true, false),
            StructuralRippleSettings::default(),
            StructuralRippleSettings::default(),
        ));
        let settings = StructuralSettings::from_fields([(
            "belongs",
            StructuralFieldSettings::render_only(true, false, false),
        )])
        .with_tide_policies_from_entries(&[relation]);

        let tide = Tide::from_entries(&[old], &[new], &settings, &[]).unwrap();

        let workitems =
            tide.statuses().iter().map(|status| status.workitem.to_string()).collect::<Vec<_>>();
        assert_eq!(workitems, ["ripple,belongs,to,new-neighbor"]);
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
