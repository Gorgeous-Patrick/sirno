//! Structural edge policy and neighbor indexes.
//!
//! Structural settings define which metadata fields Sirno treats as graph edges.
//! The edge index derives `to`, `from`, and `clique` neighbors from parsed entries.

use std::collections::{BTreeMap, BTreeSet};
use std::fmt;
use std::str::FromStr;

use indexmap::IndexMap;
use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::entry::Entry;
use crate::id::EntryId;

fn is_false(value: &bool) -> bool {
    !*value
}

/// Configured ripple sources for one structural edge direction.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
// sirno:witness:structural-edge-policy:begin
pub struct StructuralRippleSettings {
    /// Include waterline neighbors in tide workitems.
    #[serde(skip_serializing_if = "is_false")]
    pub lake: bool,
    /// Include frostline neighbors in tide workitems.
    #[serde(skip_serializing_if = "is_false")]
    pub frost: bool,
}
// sirno:witness:structural-edge-policy:end

impl StructuralRippleSettings {
    /// Construct ripple settings from explicit source flags.
    pub fn new(lake: bool, frost: bool) -> Self {
        Self { lake, frost }
    }

    /// Returns true when no ripple source is enabled.
    pub fn is_empty(&self) -> bool {
        !self.lake && !self.frost
    }
}

/// Tooling settings for one structural edge direction.
///
/// `render` includes the edge in generated footers.
/// `ripple` includes the edge in tide workitem generation.
// sirno:witness:structural-edge-policy:begin
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct StructuralEdgeSettings {
    /// Include this edge direction in generated footer rendering.
    #[serde(skip_serializing_if = "is_false")]
    pub render: bool,
    /// Include this edge direction in tide workitem generation.
    #[serde(skip_serializing_if = "StructuralRippleSettings::is_empty")]
    pub ripple: StructuralRippleSettings,
}
// sirno:witness:structural-edge-policy:end

// sirno:witness:structural-edge-policy:begin
impl StructuralEdgeSettings {
    /// Construct structural edge settings from explicit render and ripple settings.
    pub fn new(render: bool, ripple: StructuralRippleSettings) -> Self {
        Self { render, ripple }
    }

    /// Construct an edge used only for generated footer rendering.
    pub fn render_only(enabled: bool) -> Self {
        Self::new(enabled, StructuralRippleSettings::default())
    }

    /// Construct an edge used for rendering and selected ripple sources.
    pub fn render_and_ripple(render: bool, lake: bool, frost: bool) -> Self {
        Self::new(render, StructuralRippleSettings::new(lake, frost))
    }
}
// sirno:witness:structural-edge-policy:end

impl fmt::Display for StructuralEdgeSettings {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut parts = Vec::new();
        if self.render {
            parts.push("render=true");
        }
        if self.ripple.lake {
            parts.push("ripple.lake=true");
        }
        if self.ripple.frost {
            parts.push("ripple.frost=true");
        }
        if parts.is_empty() {
            write!(formatter, "none")
        } else {
            write!(formatter, "{}", parts.join(" "))
        }
    }
}

/// Direction of one configured structural edge.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum StructuralEdgeDirection {
    /// Outgoing metadata targets from the current entry.
    To,
    /// Incoming metadata sources that point at the current entry.
    From,
    /// Entries connected through a shared target in the same field.
    Clique,
}

impl StructuralEdgeDirection {
    /// Directions in deterministic generated-footer and tide order.
    pub const ORDER: [Self; 3] = [Self::To, Self::From, Self::Clique];

    /// Lowercase direction label.
    pub fn label(self) -> &'static str {
        match self {
            | Self::To => "to",
            | Self::From => "from",
            | Self::Clique => "clique",
        }
    }
}

impl fmt::Display for StructuralEdgeDirection {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.label())
    }
}

impl FromStr for StructuralEdgeDirection {
    type Err = StructuralEdgeDirectionParseError;

    fn from_str(raw: &str) -> Result<Self, Self::Err> {
        match raw {
            | "to" => Ok(Self::To),
            | "from" => Ok(Self::From),
            | "clique" => Ok(Self::Clique),
            | direction => Err(StructuralEdgeDirectionParseError(direction.to_owned())),
        }
    }
}

/// Error raised when text does not name a structural edge direction.
#[derive(Debug, Error, PartialEq, Eq)]
#[error("unknown structural edge direction `{0}`; expected to, from, or clique")]
pub struct StructuralEdgeDirectionParseError(String);

/// Settings for one configured structural field.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
// sirno:witness:structural-edge-policy:begin
pub struct StructuralFieldSettings {
    /// Outgoing metadata target edge policy.
    #[serde(skip_serializing_if = "is_default")]
    pub to: StructuralEdgeSettings,
    /// Incoming metadata source edge policy.
    #[serde(skip_serializing_if = "is_default")]
    pub from: StructuralEdgeSettings,
    /// Shared-target clique edge policy.
    #[serde(skip_serializing_if = "is_default")]
    pub clique: StructuralEdgeSettings,
}
// sirno:witness:structural-edge-policy:end

impl StructuralFieldSettings {
    /// Construct structural field settings from explicit edge policies.
    pub fn new(
        to: StructuralEdgeSettings, from: StructuralEdgeSettings, clique: StructuralEdgeSettings,
    ) -> Self {
        Self { to, from, clique }
    }

    /// Construct structural field settings from render-only edge flags.
    pub fn render_only(to: bool, from: bool, clique: bool) -> Self {
        Self::new(
            StructuralEdgeSettings::render_only(to),
            StructuralEdgeSettings::render_only(from),
            StructuralEdgeSettings::render_only(clique),
        )
    }

    /// Return settings for one structural edge direction.
    pub fn edge(&self, direction: StructuralEdgeDirection) -> &StructuralEdgeSettings {
        match direction {
            | StructuralEdgeDirection::To => &self.to,
            | StructuralEdgeDirection::From => &self.from,
            | StructuralEdgeDirection::Clique => &self.clique,
        }
    }
}

fn is_default<T: Default + PartialEq>(value: &T) -> bool {
    value == &T::default()
}

/// Ordered structural field settings from `Sirno.toml`.
pub type StructuralFieldMap = IndexMap<String, StructuralFieldSettings>;

/// Configured structural fields.
///
/// Each key names a metadata field that Sirno should treat as structural.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(transparent)]
// sirno:witness:structural-edge-policy:begin
pub struct StructuralSettings {
    fields: StructuralFieldMap,
}
// sirno:witness:structural-edge-policy:end

impl StructuralSettings {
    /// Construct structural settings from explicit field settings.
    pub fn from_fields(
        fields: impl IntoIterator<Item = (impl Into<String>, StructuralFieldSettings)>,
    ) -> Self {
        Self {
            fields: fields.into_iter().map(|(field, settings)| (field.into(), settings)).collect(),
        }
    }

    /// Iterate configured fields in user-authored order.
    pub fn fields(&self) -> impl Iterator<Item = (&str, &StructuralFieldSettings)> {
        self.fields.iter().map(|(field, settings)| (field.as_str(), settings))
    }

    /// Return true when a metadata field is configured as structural.
    pub fn contains_field(&self, field: &str) -> bool {
        self.fields.contains_key(field)
    }
}

/// Lake-wide structural edge context.
///
/// Invariant: each clique target maps to itself and every parsed entry that names it.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
// sirno:witness:generated-footer:begin
pub struct StructuralEdgeIndex {
    sources_by_field_target: BTreeMap<String, BTreeMap<EntryId, BTreeSet<EntryId>>>,
    cliques_by_field_target: BTreeMap<String, BTreeMap<EntryId, BTreeSet<EntryId>>>,
}
// sirno:witness:generated-footer:end

impl StructuralEdgeIndex {
    /// Construct a structural edge index from parsed entries.
    // sirno:witness:generated-footer:begin
    pub fn from_entries(entries: &[Entry]) -> Self {
        let mut sources_by_field_target =
            BTreeMap::<String, BTreeMap<EntryId, BTreeSet<EntryId>>>::new();
        let mut cliques_by_field_target =
            BTreeMap::<String, BTreeMap<EntryId, BTreeSet<EntryId>>>::new();
        for entry in entries {
            for (field, targets) in entry.metadata.structural_fields() {
                Self::insert_sources(
                    sources_by_field_target.entry(field.to_owned()).or_default(),
                    &entry.id,
                    targets,
                );
                Self::insert_cliques(
                    cliques_by_field_target.entry(field.to_owned()).or_default(),
                    &entry.id,
                    targets,
                );
            }
            // sirno:witness:generated-footer:end
            // sirno:witness:generated-footer:begin
        }
        Self { sources_by_field_target, cliques_by_field_target }
    }
    // sirno:witness:generated-footer:end

    fn insert_sources(
        sources_by_target: &mut BTreeMap<EntryId, BTreeSet<EntryId>>, source: &EntryId,
        targets: &[EntryId],
    ) {
        for target in targets {
            sources_by_target.entry(target.clone()).or_default().insert(source.clone());
        }
    }

    fn insert_cliques(
        cliques_by_target: &mut BTreeMap<EntryId, BTreeSet<EntryId>>, source: &EntryId,
        targets: &[EntryId],
    ) {
        for target in targets {
            let clique = cliques_by_target.entry(target.clone()).or_default();
            clique.insert(target.clone());
            clique.insert(source.clone());
        }
    }

    /// Return target entries for one structural edge direction.
    pub fn edge_targets(
        &self, field: &str, direction: StructuralEdgeDirection, entry: &Entry,
    ) -> BTreeSet<EntryId> {
        match direction {
            | StructuralEdgeDirection::To => {
                entry.metadata.structural_targets_for(field).iter().cloned().collect()
            }
            | StructuralEdgeDirection::From => self.incoming_targets(field, entry),
            | StructuralEdgeDirection::Clique => self.clique_targets(field, entry),
        }
    }

    fn incoming_targets(&self, field: &str, entry: &Entry) -> BTreeSet<EntryId> {
        self.sources_by_field_target
            .get(field)
            .and_then(|sources_by_target| sources_by_target.get(&entry.id))
            .cloned()
            .unwrap_or_default()
    }

    // sirno:witness:structural-edge-policy:begin
    fn clique_targets(&self, field: &str, entry: &Entry) -> BTreeSet<EntryId> {
        let mut targets = BTreeSet::new();
        let Some(cliques_by_target) = self.cliques_by_field_target.get(field) else {
            return targets;
        };
        for target in entry.metadata.structural_targets_for(field) {
            if let Some(clique) = cliques_by_target.get(target) {
                targets.extend(clique.iter().filter(|id| *id != &entry.id).cloned());
            }
        }
        if let Some(clique) = cliques_by_target.get(&entry.id) {
            targets.extend(clique.iter().filter(|id| *id != &entry.id).cloned());
        }
        targets
    }
    // sirno:witness:structural-edge-policy:end
}
