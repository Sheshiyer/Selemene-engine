//! Human Design data structures

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HDChart {
    pub personality_activations: Vec<Activation>,
    pub design_activations: Vec<Activation>,
    pub centers: HashMap<Center, CenterState>,
    pub channels: Vec<Channel>,
    pub hd_type: HDType,
    pub authority: Authority,
    pub profile: Profile,
    pub definition: Definition,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Activation {
    pub planet: Planet,
    pub gate: u8,
    pub line: u8,
    pub longitude: f64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Planet {
    Sun,
    Earth,
    Moon,
    NorthNode,
    SouthNode,
    Mercury,
    Venus,
    Mars,
    Jupiter,
    Saturn,
    Uranus,
    Neptune,
    Pluto,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CenterState {
    pub defined: bool,
    pub gates: Vec<u8>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Center {
    Head,
    Ajna,
    Throat,
    G,
    Heart,
    Spleen,
    SolarPlexus,
    Sacral,
    Root,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Channel {
    pub gate1: u8,
    pub gate2: u8,
    pub name: String,
    pub circuitry: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum HDType {
    Generator,
    ManifestingGenerator,
    Projector,
    Manifestor,
    Reflector,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Authority {
    Sacral,
    Emotional,
    Splenic,
    Heart,
    GCenter,
    Mental,
    Lunar,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Profile {
    pub conscious_line: u8,
    pub unconscious_line: u8,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Definition {
    Single,
    Split,
    TripleSplit,
    QuadrupleSplit,
    NoDefinition,
}
