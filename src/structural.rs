//! Structural link policy and neighbor indexes.
//!
//! Structural settings define which metadata relations Sirno treats as graph links.
//! The link index derives `to`, `from`, and `clique` neighbors from parsed entries.

use std::collections::{BTreeMap, BTreeSet};
use std::fmt;
use std::str::FromStr;

use indexmap::IndexMap;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use thiserror::Error;

use crate::entry::{DESC_FIELD, Entry, FROZEN_FIELD, META_FIELD, NAME_FIELD};
use crate::identifier::EntryAddress;

fn is_false(value: &bool) -> bool {
    !*value
}

/// Configured ripple sources for one structural link direction.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
// sirno:witness:structural-edge-policy:begin
pub struct StructuralRippleSettings {
    /// Include waterline neighbors in tide workitems.
    #[serde(skip_serializing_if = "is_false")]
    pub lake: bool,
    /// Include Anchor-side neighbors in tide workitems.
    #[serde(skip_serializing_if = "is_false")]
    pub anchor: bool,
}
// sirno:witness:structural-edge-policy:end

impl StructuralRippleSettings {
    /// Construct ripple settings from explicit source flags.
    pub fn new(lake: bool, anchor: bool) -> Self {
        Self { lake, anchor }
    }

    /// Returns true when no ripple source is enabled.
    pub fn is_empty(&self) -> bool {
        !self.lake && !self.anchor
    }
}

/// Tooling settings for one structural link direction.
///
/// `render` includes the edge in generated footers.
/// `ripple` includes the edge in tide workitem generation.
// sirno:witness:structural-edge-policy:begin
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct StructuralEdgeSettings {
    /// Include this edge direction in generated footer rendering.
    pub render: bool,
    /// Include this edge direction in tide workitem generation.
    pub ripple: StructuralRippleSettings,
}
// sirno:witness:structural-edge-policy:end

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
struct StructuralEdgeConfig {
    #[serde(skip_serializing_if = "is_false")]
    render: bool,
}

impl Serialize for StructuralEdgeSettings {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        StructuralEdgeConfig { render: self.render }.serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for StructuralEdgeSettings {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let config = StructuralEdgeConfig::deserialize(deserializer)?;
        Ok(Self::render_only(config.render))
    }
}

// sirno:witness:structural-edge-policy:begin
impl StructuralEdgeSettings {
    /// Construct structural link settings from explicit render and ripple settings.
    pub fn new(render: bool, ripple: StructuralRippleSettings) -> Self {
        Self { render, ripple }
    }

    /// Construct an edge used only for generated footer rendering.
    pub fn render_only(enabled: bool) -> Self {
        Self::new(enabled, StructuralRippleSettings::default())
    }

    /// Construct an edge used for rendering and selected ripple sources.
    pub fn render_and_ripple(render: bool, lake: bool, anchor: bool) -> Self {
        Self::new(render, StructuralRippleSettings::new(lake, anchor))
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
        if self.ripple.anchor {
            parts.push("ripple.anchor=true");
        }
        if parts.is_empty() {
            write!(formatter, "none")
        } else {
            write!(formatter, "{}", parts.join(" "))
        }
    }
}

/// Direction of one structural link.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum StructuralEdgeDirection {
    /// Outgoing metadata targets from the current entry.
    To,
    /// Incoming metadata sources that point at the current entry.
    From,
    /// Entries connected through a shared target in the same relation.
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

/// Error raised when text does not name a structural link direction.
#[derive(Debug, Error, PartialEq, Eq)]
#[error("unknown structural link direction `{0}`; expected to, from, or clique")]
pub struct StructuralEdgeDirectionParseError(String);

/// Effective settings for one link relation.
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
    /// Construct link relation settings from explicit edge policies.
    pub fn new(
        to: StructuralEdgeSettings, from: StructuralEdgeSettings, clique: StructuralEdgeSettings,
    ) -> Self {
        Self { to, from, clique }
    }

    /// Construct link relation settings from render-only edge flags.
    pub fn render_only(to: bool, from: bool, clique: bool) -> Self {
        Self::new(
            StructuralEdgeSettings::render_only(to),
            StructuralEdgeSettings::render_only(from),
            StructuralEdgeSettings::render_only(clique),
        )
    }

    /// Return settings for one structural link direction.
    pub fn edge(&self, direction: StructuralEdgeDirection) -> &StructuralEdgeSettings {
        match direction {
            | StructuralEdgeDirection::To => &self.to,
            | StructuralEdgeDirection::From => &self.from,
            | StructuralEdgeDirection::Clique => &self.clique,
        }
    }

    /// Return these settings with entry-defined tide policy merged into each direction.
    pub fn with_tide_policy(mut self, policy: StructuralTideSettings) -> Self {
        self.to.ripple = policy.to;
        self.from.ripple = policy.from;
        self.clique.ripple = policy.clique;
        self
    }

    /// Return these settings with only generated-footer render policy retained.
    pub fn without_tide_policy(self) -> Self {
        Self::render_only(self.to.render, self.from.render, self.clique.render)
    }
}

fn is_default<T: Default + PartialEq>(value: &T) -> bool {
    value == &T::default()
}

/// Tide policy authored by the entry that defines one structural relation.
///
/// Invariant: each direction stores waterline and Anchor-side participation only.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
// sirno:witness:structural-edge-policy:begin
pub struct StructuralTideSettings {
    /// Outgoing metadata target tide policy.
    #[serde(skip_serializing_if = "StructuralRippleSettings::is_empty")]
    pub to: StructuralRippleSettings,
    /// Incoming metadata source tide policy.
    #[serde(skip_serializing_if = "StructuralRippleSettings::is_empty")]
    pub from: StructuralRippleSettings,
    /// Shared-target clique tide policy.
    #[serde(skip_serializing_if = "StructuralRippleSettings::is_empty")]
    pub clique: StructuralRippleSettings,
}
// sirno:witness:structural-edge-policy:end

impl StructuralTideSettings {
    /// Construct tide settings from explicit direction policies.
    pub fn new(
        to: StructuralRippleSettings, from: StructuralRippleSettings,
        clique: StructuralRippleSettings,
    ) -> Self {
        Self { to, from, clique }
    }

    /// Return true when no tide source is enabled in any direction.
    pub fn is_empty(&self) -> bool {
        self.to.is_empty() && self.from.is_empty() && self.clique.is_empty()
    }
}

/// Ordered effective link relation settings.
///
/// Config parsing fills relation order and relation entries from `Sirno.toml`.
/// Render callers merge generated-footer policy from `[render.structural]`.
/// Tide callers merge `meta.ripple.lake` and `meta.ripple.anchor` from relation entries.
pub type StructuralFieldMap = IndexMap<String, StructuralFieldSettings>;

/// Ordered structural relation entries.
pub type StructuralRelationMap = IndexMap<String, StructuralRelationSettings>;

/// Configured entry that defines one structural relation.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct StructuralRelationSettings {
    /// Entry that documents the relation and may define entry-side Tide policy.
    pub entry: EntryAddress,
}

/// Generated-footer render directions by structural relation.
pub type StructuralRenderMap = IndexMap<String, Vec<StructuralEdgeDirection>>;

/// Mist generated-footer render policy for structural links.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(transparent)]
pub struct StructuralRenderSettings {
    fields: StructuralRenderMap,
}

impl StructuralRenderSettings {
    /// Construct render settings from explicit relation directions.
    pub fn from_fields(
        fields: impl IntoIterator<
            Item = (impl Into<String>, impl IntoIterator<Item = StructuralEdgeDirection>),
        >,
    ) -> Self {
        Self {
            fields: fields
                .into_iter()
                .map(|(field, directions)| (field.into(), directions.into_iter().collect()))
                .collect(),
        }
    }

    /// Iterate render policies in user-authored order.
    pub fn fields(&self) -> impl Iterator<Item = (&str, &[StructuralEdgeDirection])> {
        self.fields.iter().map(|(field, directions)| (field.as_str(), directions.as_slice()))
    }

    /// Return render directions for one relation.
    pub fn directions_for(&self, field: &str) -> Option<&[StructuralEdgeDirection]> {
        self.fields.get(field).map(Vec::as_slice)
    }

    /// Return true when no structural render policy is configured.
    pub fn is_empty(&self) -> bool {
        self.fields.is_empty()
    }
}

/// Structural link relations and effective edge policy.
///
/// Each key names a metadata relation that Sirno should treat as structural.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
// sirno:witness:structural-edge-policy:begin
pub struct StructuralSettings {
    fields: StructuralFieldMap,
    entries: IndexMap<String, EntryAddress>,
}
// sirno:witness:structural-edge-policy:end

impl Serialize for StructuralSettings {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        self.relation_settings().serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for StructuralSettings {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let relations = StructuralRelationMap::deserialize(deserializer)?;
        Ok(Self::from_relations(
            relations.into_iter().map(|(field, settings)| (field, settings.entry)),
        ))
    }
}

impl StructuralSettings {
    /// Discover structural relations from parsed relation entries.
    ///
    /// Relation order is entry-address order.
    pub fn from_entries(entries: &[Entry]) -> Self {
        let mut relation_entries = entries
            .iter()
            .filter(|entry| {
                entry.metadata.meta.is_structural_relation()
                    && validate_structural_field_name(entry.id.as_str()).is_ok()
            })
            .map(|entry| entry.id.clone())
            .collect::<Vec<_>>();
        relation_entries.sort();

        Self::from_relations(
            relation_entries.into_iter().map(|entry| (entry.as_str().to_owned(), entry)),
        )
    }

    /// Construct structural settings from explicit relation settings.
    pub fn from_fields(
        fields: impl IntoIterator<Item = (impl Into<String>, StructuralFieldSettings)>,
    ) -> Self {
        let mut settings = Self::default();
        for (field, field_settings) in fields {
            let field = field.into();
            let entry = EntryAddress::new(&field)
                .expect("structural field can default to its matching entry address");
            settings.fields.insert(field.clone(), field_settings);
            settings.entries.insert(field, entry);
        }
        settings
    }

    /// Construct structural settings from relation entries.
    pub fn from_relations(
        relations: impl IntoIterator<Item = (impl Into<String>, EntryAddress)>,
    ) -> Self {
        let mut settings = Self::default();
        for (field, entry) in relations {
            settings.set_relation_entry(field, entry);
        }
        settings
    }

    /// Return relation entries in structural order.
    pub fn relations(&self) -> impl Iterator<Item = (&str, &EntryAddress)> {
        self.entries.iter().map(|(field, entry)| (field.as_str(), entry))
    }

    /// Iterate relations in structural order.
    pub fn fields(&self) -> impl Iterator<Item = (&str, &StructuralFieldSettings)> {
        self.fields.iter().map(|(field, settings)| (field.as_str(), settings))
    }

    /// Return true when a metadata relation is defined as structural.
    pub fn contains_field(&self, field: &str) -> bool {
        self.fields.contains_key(field)
    }

    /// Return the entry that defines one structural relation.
    pub fn entry_for_field(&self, field: &str) -> Option<&EntryAddress> {
        self.entries.get(field)
    }

    /// Return true when the entry defines a discovered structural relation.
    pub fn contains_entry(&self, entry: &EntryAddress) -> bool {
        self.entries.values().any(|defined| defined == entry)
    }

    /// Add or update one structural relation entry.
    ///
    /// Existing relations keep their original order position.
    pub fn set_relation_entry(&mut self, field: impl Into<String>, entry: EntryAddress) -> bool {
        let field = field.into();
        let changed = self.entries.get(&field) != Some(&entry);
        self.fields.entry(field.clone()).or_default();
        self.entries.insert(field, entry);
        changed
    }

    /// Rename a structural relation entry address.
    pub fn rename_entry_reference(&mut self, old_id: &EntryAddress, new_id: &EntryAddress) -> bool {
        let mut changed = false;
        for entry in self.entries.values_mut() {
            if entry == old_id {
                *entry = new_id.clone();
                changed = true;
            }
        }
        changed
    }

    /// Rename one structural link relation.
    ///
    /// The field stays in its original order position.
    pub fn rename_field(&mut self, old_id: &EntryAddress, new_id: &EntryAddress) -> bool {
        let old_field = old_id.as_str();
        if !self.fields.contains_key(old_field) {
            return false;
        }

        let mut renamed_fields = StructuralFieldMap::with_capacity(self.fields.len());
        let mut renamed_entries = IndexMap::with_capacity(self.entries.len());
        for (field, settings) in std::mem::take(&mut self.fields) {
            if field == old_field {
                renamed_fields.insert(new_id.as_str().to_owned(), settings);
            } else {
                renamed_fields.insert(field, settings);
            }
        }
        for (field, entry) in std::mem::take(&mut self.entries) {
            if field == old_field {
                renamed_entries.insert(new_id.as_str().to_owned(), entry);
            } else {
                renamed_entries.insert(field, entry);
            }
        }
        self.fields = renamed_fields;
        self.entries = renamed_entries;
        true
    }

    /// Return effective settings with generated-footer render policy applied.
    pub fn with_render_settings(&self, render: &StructuralRenderSettings) -> Self {
        let mut fields = StructuralFieldMap::new();
        let mut entries = IndexMap::new();
        for (field, directions) in render.fields() {
            let Some(entry) = self.entries.get(field) else {
                continue;
            };
            fields.insert(
                field.to_owned(),
                StructuralFieldSettings::render_only(
                    directions.contains(&StructuralEdgeDirection::To),
                    directions.contains(&StructuralEdgeDirection::From),
                    directions.contains(&StructuralEdgeDirection::Clique),
                ),
            );
            entries.insert(field.to_owned(), entry.clone());
        }
        for (field, entry) in &self.entries {
            if fields.contains_key(field) {
                continue;
            }
            fields.insert(field.clone(), StructuralFieldSettings::default());
            entries.insert(field.clone(), entry.clone());
        }
        Self { fields, entries }
    }

    /// Return effective settings with entry-side tide policies merged into relations.
    // sirno:witness:structural-edge-policy:begin
    pub fn with_tide_policies_from_entries(&self, entries: &[Entry]) -> Self {
        let policies = entries
            .iter()
            .filter(|entry| entry.metadata.meta.is_structural_relation())
            .map(|entry| {
                (entry.id.as_str().to_owned(), entry.metadata.meta.tide.unwrap_or_default())
            })
            .collect::<BTreeMap<_, _>>();
        let fields = self
            .fields
            .iter()
            .map(|(field, settings)| {
                let settings = settings.without_tide_policy();
                let settings = policies
                    .get(
                        self.entries
                            .get(field)
                            .expect("structural relation has matching entry")
                            .as_str(),
                    )
                    .map(|policy| settings.with_tide_policy(*policy))
                    .unwrap_or(settings);
                (field.clone(), settings)
            })
            .collect();
        Self { fields, entries: self.entries.clone() }
    }
    // sirno:witness:structural-edge-policy:end

    fn relation_settings(&self) -> StructuralRelationMap {
        self.entries
            .iter()
            .map(|(field, entry)| {
                (field.clone(), StructuralRelationSettings { entry: entry.clone() })
            })
            .collect()
    }
}

/// Validate a metadata key used as a structural relation name.
pub fn validate_structural_field_name(field: &str) -> Result<(), StructuralFieldNameError> {
    if field.is_empty() || field.contains('\n') || field.contains('\r') || field.contains(',') {
        return Err(StructuralFieldNameError::Invalid(field.to_owned()));
    }
    if matches!(field, NAME_FIELD | DESC_FIELD | META_FIELD | FROZEN_FIELD) {
        return Err(StructuralFieldNameError::Reserved(field.to_owned()));
    }
    Ok(())
}

/// Error raised when a structural relation entry cannot be used as a metadata key.
#[derive(Debug, Error, PartialEq, Eq)]
pub enum StructuralFieldNameError {
    /// The field is empty, multiline, or contains a forbidden separator.
    #[error("structural relation name must be a non-empty single-line metadata key: {0}")]
    Invalid(String),
    /// The field belongs to required or managed metadata.
    #[error("structural relation name is reserved for Sirno metadata: {0}")]
    Reserved(String),
}

/// Lake-wide structural link context.
///
/// Invariant: each clique target maps to itself and every parsed entry that names it.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
// sirno:witness:generated-footer:begin
pub struct StructuralEdgeIndex {
    sources_by_field_target: BTreeMap<String, BTreeMap<EntryAddress, BTreeSet<EntryAddress>>>,
    cliques_by_field_target: BTreeMap<String, BTreeMap<EntryAddress, BTreeSet<EntryAddress>>>,
}
// sirno:witness:generated-footer:end

impl StructuralEdgeIndex {
    /// Construct a structural link index from parsed entries.
    // sirno:witness:generated-footer:begin
    pub fn from_entries(entries: &[Entry]) -> Self {
        let mut sources_by_field_target =
            BTreeMap::<String, BTreeMap<EntryAddress, BTreeSet<EntryAddress>>>::new();
        let mut cliques_by_field_target =
            BTreeMap::<String, BTreeMap<EntryAddress, BTreeSet<EntryAddress>>>::new();
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
        sources_by_target: &mut BTreeMap<EntryAddress, BTreeSet<EntryAddress>>,
        source: &EntryAddress, targets: &[EntryAddress],
    ) {
        for target in targets {
            sources_by_target.entry(target.clone()).or_default().insert(source.clone());
        }
    }

    fn insert_cliques(
        cliques_by_target: &mut BTreeMap<EntryAddress, BTreeSet<EntryAddress>>,
        source: &EntryAddress, targets: &[EntryAddress],
    ) {
        for target in targets {
            let clique = cliques_by_target.entry(target.clone()).or_default();
            clique.insert(target.clone());
            clique.insert(source.clone());
        }
    }

    /// Return target entries for one structural link direction.
    pub fn edge_targets(
        &self, field: &str, direction: StructuralEdgeDirection, entry: &Entry,
    ) -> BTreeSet<EntryAddress> {
        match direction {
            | StructuralEdgeDirection::To => {
                entry.metadata.structural_targets_for(field).iter().cloned().collect()
            }
            | StructuralEdgeDirection::From => self.incoming_targets(field, entry),
            | StructuralEdgeDirection::Clique => self.clique_targets(field, entry),
        }
    }

    fn incoming_targets(&self, field: &str, entry: &Entry) -> BTreeSet<EntryAddress> {
        self.sources_by_field_target
            .get(field)
            .and_then(|sources_by_target| sources_by_target.get(&entry.id))
            .cloned()
            .unwrap_or_default()
    }

    // sirno:witness:structural-edge-policy:begin
    fn clique_targets(&self, field: &str, entry: &Entry) -> BTreeSet<EntryAddress> {
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::entry::{EntryMetaType, EntryMetadata};

    #[test]
    fn renames_structural_setting_field_in_place() {
        let mut settings = StructuralSettings::from_fields([
            ("category", StructuralFieldSettings::default()),
            ("refines", StructuralFieldSettings::render_only(true, false, false)),
            ("belongs", StructuralFieldSettings::default()),
        ]);

        assert!(settings.rename_field(
            &EntryAddress::new("refines").unwrap(),
            &EntryAddress::new("prerequisite").unwrap()
        ));

        let fields = settings.fields().map(|(field, _)| field).collect::<Vec<_>>();
        assert_eq!(fields, ["category", "prerequisite", "belongs"]);
        assert_eq!(settings.fields().nth(1).map(|(_, settings)| settings.to.render), Some(true));
        assert!(!settings.contains_field("refines"));
        assert_eq!(
            settings.entry_for_field("prerequisite"),
            Some(&EntryAddress::new("refines").unwrap())
        );
    }

    #[test]
    fn relations_track_entry_addresses() {
        let mut settings = StructuralSettings::from_relations([(
            "kind",
            EntryAddress::new("metadata.kind").unwrap(),
        )]);

        assert!(settings.contains_field("kind"));
        assert!(settings.contains_entry(&EntryAddress::new("metadata.kind").unwrap()));
        assert_eq!(
            settings.entry_for_field("kind"),
            Some(&EntryAddress::new("metadata.kind").unwrap())
        );
        assert!(settings.rename_entry_reference(
            &EntryAddress::new("metadata.kind").unwrap(),
            &EntryAddress::new("metadata.type").unwrap()
        ));
        assert_eq!(
            settings.entry_for_field("kind"),
            Some(&EntryAddress::new("metadata.type").unwrap())
        );
    }

    #[test]
    fn render_settings_apply_to_relations() {
        let settings = StructuralSettings::from_relations([
            ("kind", EntryAddress::new("metadata-kind").unwrap()),
            ("area", EntryAddress::new("area").unwrap()),
        ]);
        let render = StructuralRenderSettings::from_fields([(
            "kind",
            [StructuralEdgeDirection::To, StructuralEdgeDirection::From],
        )]);

        let effective = settings.with_render_settings(&render);
        let fields = effective.fields().collect::<Vec<_>>();

        assert!(fields[0].1.to.render);
        assert!(fields[0].1.from.render);
        assert!(!fields[0].1.clique.render);
        assert!(!fields[1].1.to.render);
    }

    #[test]
    fn merges_entry_tide_policy_into_render_settings() {
        let settings = StructuralSettings::from_fields([(
            "belongs",
            StructuralFieldSettings::render_only(true, true, false),
        )]);
        let mut metadata = EntryMetadata::new("Belongs", "A relation.").unwrap();
        metadata.meta.entry_type = Some(EntryMetaType::Structural);
        metadata.meta.tide = Some(StructuralTideSettings::new(
            StructuralRippleSettings::new(true, false),
            StructuralRippleSettings::new(true, true),
            StructuralRippleSettings::default(),
        ));
        let entry = Entry::new(EntryAddress::new("belongs").unwrap(), metadata, "Body.\n");

        let effective = settings.with_tide_policies_from_entries(&[entry]);
        let (_, field_settings) = effective.fields().next().unwrap();

        assert!(field_settings.to.render);
        assert!(field_settings.from.render);
        assert!(field_settings.to.ripple.lake);
        assert!(!field_settings.to.ripple.anchor);
        assert!(field_settings.from.ripple.lake);
        assert!(field_settings.from.ripple.anchor);
        assert!(!field_settings.clique.ripple.lake);
    }

    #[test]
    fn ignores_render_settings_without_entry_policy() {
        let settings = StructuralSettings::from_fields([(
            "belongs",
            StructuralFieldSettings::new(
                StructuralEdgeSettings::render_and_ripple(true, true, true),
                StructuralEdgeSettings::render_and_ripple(false, true, true),
                StructuralEdgeSettings::render_and_ripple(false, true, true),
            ),
        )]);

        let effective = settings.with_tide_policies_from_entries(&[]);
        let (_, field_settings) = effective.fields().next().unwrap();

        assert!(field_settings.to.render);
        assert!(field_settings.to.ripple.is_empty());
        assert!(field_settings.from.ripple.is_empty());
        assert!(field_settings.clique.ripple.is_empty());
    }
}
