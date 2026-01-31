//! Static wisdom data loaded at startup

use crate::wisdom::*;
use lazy_static::lazy_static;
use std::collections::HashMap;

lazy_static! {
    /// 64 Human Design gates with complete wisdom data
    pub static ref GATES: HashMap<String, GateWisdom> = load_gates();

    /// 9 Human Design centers with functions and characteristics
    pub static ref CENTERS: HashMap<String, CenterWisdom> = load_centers();

    /// 36 Human Design channels connecting gates
    pub static ref CHANNELS: HashMap<String, ChannelWisdom> = load_channels();

    /// 5 Human Design types with strategies
    pub static ref TYPES: HashMap<String, TypeWisdom> = load_types();

    /// 7 Human Design authorities for decision-making
    pub static ref AUTHORITIES: HashMap<String, AuthorityWisdom> = load_authorities();

    /// 12 Human Design profiles (life themes)
    pub static ref PROFILES: HashMap<String, ProfileWisdom> = load_profiles();

    /// 6 Human Design lines (expressions within gates)
    pub static ref LINES: HashMap<String, LineWisdom> = load_lines();

    /// 5 Human Design definition types
    pub static ref DEFINITIONS: HashMap<String, DefinitionWisdom> = load_definitions();

    /// 3 main circuitry types (Individual, Tribal, Collective)
    pub static ref CIRCUITRY: HashMap<String, CircuitryWisdom> = load_circuitry();

    /// Incarnation crosses (life purpose combinations)
    pub static ref INCARNATION_CROSSES: HashMap<String, IncarnationCrossWisdom> = load_incarnation_crosses();

    /// Variable wisdom (tone/color data)
    pub static ref VARIABLES: HashMap<String, VariableWisdom> = load_variables();

    /// Planetary activation wisdom
    pub static ref PLANETARY_ACTIVATIONS: HashMap<String, PlanetaryActivationWisdom> = load_planetary_activations();
}

/// Initialize all wisdom data (forces lazy_static evaluation)
pub fn init_wisdom() {
    lazy_static::initialize(&GATES);
    lazy_static::initialize(&CENTERS);
    lazy_static::initialize(&CHANNELS);
    lazy_static::initialize(&TYPES);
    lazy_static::initialize(&AUTHORITIES);
    lazy_static::initialize(&PROFILES);
    lazy_static::initialize(&LINES);
    lazy_static::initialize(&DEFINITIONS);
    lazy_static::initialize(&CIRCUITRY);
    lazy_static::initialize(&INCARNATION_CROSSES);
    lazy_static::initialize(&VARIABLES);
    lazy_static::initialize(&PLANETARY_ACTIVATIONS);
}
