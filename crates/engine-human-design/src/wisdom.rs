//! Human Design wisdom data structures and loader

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GateWisdom {
    pub number: u8,
    pub name: String,
    pub keynote: String,
    pub description: String,
    pub center: String,
    pub channel_partner: Option<u8>,
    pub gift: Option<String>,
    pub shadow: Option<String>,
    pub siddhi: Option<String>,
    #[serde(default)]
    pub codon: String,
    #[serde(default)]
    pub amino_acid: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CenterWisdom {
    pub name: String,
    #[serde(rename = "type")]
    pub center_type: String,
    pub function: String,
    pub gates: Vec<u8>,
    pub when_defined: String,
    pub when_undefined: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChannelWisdom {
    pub name: String,
    pub gates: Vec<u8>,
    pub centers: Vec<String>,
    pub description: String,
    pub circuitry: String,
    #[serde(default)]
    pub keynote: String,
    #[serde(default)]
    pub theme: String,
    #[serde(rename = "type", default)]
    pub channel_type: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TypeWisdom {
    pub name: String,
    pub percentage: u8,
    pub aura: String,
    pub strategy: String,
    pub authority_types: Vec<String>,
    pub signature: String,
    pub not_self_theme: String,
    pub definition_requirement: String,
    pub description: String,
    pub characteristics: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthorityWisdom {
    pub name: String,
    pub types: Vec<String>,
    pub percentage: String,
    pub center: String,
    pub requirement: String,
    pub description: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProfileWisdom {
    pub name: String,
    pub theme: String,
    pub description: String,
    pub conscious: String,
    pub unconscious: String,
    pub life_purpose: String,
    pub characteristics: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LineWisdom {
    pub name: String,
    pub theme: String,
    pub description: String,
    pub characteristics: Vec<String>,
    pub keywords: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DefinitionWisdom {
    pub name: String,
    pub percentage: String,
    pub description: String,
    pub characteristics: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CircuitryWisdom {
    pub name: String,
    pub theme: String,
    pub description: String,
    #[serde(default)]
    pub energy_type: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IncarnationCrossWisdom {
    pub name: String,
    #[serde(rename = "type")]
    pub cross_type: String,
    #[serde(default)]
    pub gates: CrossGates,
    pub description: String,
    #[serde(default)]
    pub theme: String,
    #[serde(default)]
    pub purpose: String,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct CrossGates {
    #[serde(default)]
    pub conscious_sun: u8,
    #[serde(default)]
    pub conscious_earth: u8,
    #[serde(default)]
    pub unconscious_sun: u8,
    #[serde(default)]
    pub unconscious_earth: u8,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VariableWisdom {
    pub name: String,
    pub description: String,
    #[serde(default)]
    pub characteristics: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlanetaryActivationWisdom {
    pub name: String,
    #[serde(default)]
    pub symbol: String,
    #[serde(default)]
    pub importance: String,
    #[serde(default)]
    pub personality_meaning: String,
    #[serde(default)]
    pub design_meaning: String,
    #[serde(default)]
    pub keynotes: Vec<String>,
    #[serde(default)]
    pub description: String,
    #[serde(default)]
    pub theme: String,
}

#[derive(Debug, Deserialize)]
struct GatesFile {
    gates: HashMap<String, GateWisdom>,
}

#[derive(Debug, Deserialize)]
struct CentersFile {
    centers: HashMap<String, CenterWisdom>,
}

#[derive(Debug, Deserialize)]
struct ChannelsFile {
    channels: HashMap<String, ChannelWisdom>,
}

#[derive(Debug, Deserialize)]
struct TypesFile {
    types: HashMap<String, TypeWisdom>,
}

#[derive(Debug, Deserialize)]
struct AuthoritiesFile {
    authorities: HashMap<String, AuthorityWisdom>,
}

#[derive(Debug, Deserialize)]
struct ProfilesFile {
    profiles: HashMap<String, ProfileWisdom>,
}

#[derive(Debug, Deserialize)]
struct LinesFile {
    lines: HashMap<String, LineWisdom>,
}

#[derive(Debug, Deserialize)]
struct DefinitionsFile {
    definition_types: HashMap<String, DefinitionWisdom>,
}

#[derive(Debug, Deserialize)]
struct CircuitryFile {
    circuits: HashMap<String, CircuitryWisdom>,
}

#[derive(Debug, Deserialize)]
struct IncarnationCrossesFile {
    crosses: HashMap<String, IncarnationCrossWisdom>,
}

#[derive(Debug, Deserialize)]
struct VariablesFile {
    variables: HashMap<String, VariableWisdom>,
}

#[derive(Debug, Deserialize)]
struct PlanetaryActivationsFile {
    planets: HashMap<String, PlanetaryActivationWisdom>,
}

fn load_wisdom_file<T: for<'de> Deserialize<'de>>(path: &str) -> Result<T, String> {
    let content = std::fs::read_to_string(path)
        .map_err(|e| format!("Failed to read {}: {}", path, e))?;
    serde_json::from_str(&content)
        .map_err(|e| format!("Failed to parse {}: {}", path, e))
}

pub fn load_gates() -> HashMap<String, GateWisdom> {
    let path = concat!(env!("CARGO_MANIFEST_DIR"), "/../../data/human-design/gates.json");
    load_wisdom_file::<GatesFile>(path)
        .map(|f| f.gates)
        .unwrap_or_else(|e| {
            eprintln!("Error loading gates: {}", e);
            HashMap::new()
        })
}

pub fn load_centers() -> HashMap<String, CenterWisdom> {
    let path = concat!(env!("CARGO_MANIFEST_DIR"), "/../../data/human-design/centers.json");
    load_wisdom_file::<CentersFile>(path)
        .map(|f| f.centers)
        .unwrap_or_else(|e| {
            eprintln!("Error loading centers: {}", e);
            HashMap::new()
        })
}

pub fn load_channels() -> HashMap<String, ChannelWisdom> {
    let path = concat!(env!("CARGO_MANIFEST_DIR"), "/../../data/human-design/channels.json");
    load_wisdom_file::<ChannelsFile>(path)
        .map(|f| f.channels)
        .unwrap_or_else(|e| {
            eprintln!("Error loading channels: {}", e);
            HashMap::new()
        })
}

pub fn load_types() -> HashMap<String, TypeWisdom> {
    let path = concat!(env!("CARGO_MANIFEST_DIR"), "/../../data/human-design/types.json");
    load_wisdom_file::<TypesFile>(path)
        .map(|f| f.types)
        .unwrap_or_else(|e| {
            eprintln!("Error loading types: {}", e);
            HashMap::new()
        })
}

pub fn load_authorities() -> HashMap<String, AuthorityWisdom> {
    let path = concat!(env!("CARGO_MANIFEST_DIR"), "/../../data/human-design/authorities.json");
    load_wisdom_file::<AuthoritiesFile>(path)
        .map(|f| f.authorities)
        .unwrap_or_else(|e| {
            eprintln!("Error loading authorities: {}", e);
            HashMap::new()
        })
}

pub fn load_profiles() -> HashMap<String, ProfileWisdom> {
    let path = concat!(env!("CARGO_MANIFEST_DIR"), "/../../data/human-design/profiles.json");
    load_wisdom_file::<ProfilesFile>(path)
        .map(|f| f.profiles)
        .unwrap_or_else(|e| {
            eprintln!("Error loading profiles: {}", e);
            HashMap::new()
        })
}

pub fn load_lines() -> HashMap<String, LineWisdom> {
    let path = concat!(env!("CARGO_MANIFEST_DIR"), "/../../data/human-design/lines.json");
    load_wisdom_file::<LinesFile>(path)
        .map(|f| f.lines)
        .unwrap_or_else(|e| {
            eprintln!("Error loading lines: {}", e);
            HashMap::new()
        })
}

pub fn load_definitions() -> HashMap<String, DefinitionWisdom> {
    let path = concat!(env!("CARGO_MANIFEST_DIR"), "/../../data/human-design/definitions.json");
    load_wisdom_file::<DefinitionsFile>(path)
        .map(|f| f.definition_types)
        .unwrap_or_else(|e| {
            eprintln!("Error loading definitions: {}", e);
            HashMap::new()
        })
}

pub fn load_circuitry() -> HashMap<String, CircuitryWisdom> {
    let path = concat!(env!("CARGO_MANIFEST_DIR"), "/../../data/human-design/circuitry.json");
    load_wisdom_file::<CircuitryFile>(path)
        .map(|f| f.circuits)
        .unwrap_or_else(|e| {
            eprintln!("Error loading circuitry: {}", e);
            HashMap::new()
        })
}

pub fn load_incarnation_crosses() -> HashMap<String, IncarnationCrossWisdom> {
    let path = concat!(env!("CARGO_MANIFEST_DIR"), "/../../data/human-design/incarnation_crosses.json");
    load_wisdom_file::<IncarnationCrossesFile>(path)
        .map(|f| f.crosses)
        .unwrap_or_else(|e| {
            eprintln!("Error loading incarnation crosses: {}", e);
            HashMap::new()
        })
}

pub fn load_variables() -> HashMap<String, VariableWisdom> {
    let path = concat!(env!("CARGO_MANIFEST_DIR"), "/../../data/human-design/variables.json");
    load_wisdom_file::<VariablesFile>(path)
        .map(|f| f.variables)
        .unwrap_or_else(|e| {
            eprintln!("Error loading variables: {}", e);
            HashMap::new()
        })
}

pub fn load_planetary_activations() -> HashMap<String, PlanetaryActivationWisdom> {
    let path = concat!(env!("CARGO_MANIFEST_DIR"), "/../../data/human-design/planetary_activations.json");
    load_wisdom_file::<PlanetaryActivationsFile>(path)
        .map(|f| f.planets)
        .unwrap_or_else(|e| {
            eprintln!("Error loading planetary activations: {}", e);
            HashMap::new()
        })
}
