//! Verified public Gene Keys labels and their factual source boundaries.
//! Full author meanings are unavailable; historical archetypes are not a fallback.

use crate::models::{GeneKey, GeneKeysData};
use serde_json::{json, Value};
use std::collections::HashMap;
use std::sync::OnceLock;

const PUBLIC_LABELS_JSON: &str =
    include_str!("../../../data/gene-keys/verified-public-labels.json");

struct PublicLabels {
    keys: HashMap<u8, GeneKey>,
    sources: HashMap<u8, Value>,
    metadata: Value,
}

static PUBLIC_LABELS: OnceLock<PublicLabels> = OnceLock::new();

fn public_labels() -> &'static PublicLabels {
    PUBLIC_LABELS.get_or_init(|| {
        load_public_labels(PUBLIC_LABELS_JSON).expect("Invalid Gene Keys public-label source data")
    })
}

pub fn gene_keys() -> &'static HashMap<u8, GeneKey> {
    &public_labels().keys
}

pub fn get_gene_key(number: u8) -> Option<&'static GeneKey> {
    gene_keys().get(&number)
}

pub fn get_gene_key_provenance(number: u8) -> Option<&'static Value> {
    public_labels().sources.get(&number)
}

pub fn meaning_source_quality() -> Value {
    let metadata = &public_labels().metadata;
    json!({
        "status": "verified_public_labels_only",
        "source": "official_per_key_public_headings",
        "data_version": metadata["data_version"],
        "labels": metadata["label_status"],
        "descriptions": metadata["description_status"],
        "name": metadata["name_status"],
        "programming_partners": metadata["programming_partner_source"],
        "reflection_prompts": metadata["reflection_prompts_status"],
        "full_wisdom": "unavailable",
        "rights": metadata["rights"],
        "note": "Short public labels are verified. Author meanings, biological correspondences and chapter titles are unavailable. Reflection questions are Selemene's own, not author text or a measured user state."
    })
}

/// Exact half-wheel opposition, using the same gate mapping as HD calculations.
fn opposition_partner(number: u8) -> Option<u8> {
    (0..64).find_map(|position| {
        let longitude = f64::from(position) * 360.0 / 64.0;
        (engine_human_design::longitude_to_gate(longitude) == number)
            .then(|| engine_human_design::longitude_to_gate(longitude + 180.0))
    })
}

fn load_public_labels(raw: &str) -> Result<PublicLabels, Box<dyn std::error::Error>> {
    let document: Value = serde_json::from_str(raw)?;
    let data: GeneKeysData = serde_json::from_value(document.clone())?;
    if data.gene_keys_info.total_keys != 64 || data.gene_keys.len() != 64 {
        return Err("Expected exactly 64 Gene Keys public-label records".into());
    }
    let metadata = document["metadata"].clone();
    if metadata["data_version"].as_str().is_none_or(str::is_empty)
        || metadata["label_status"] != "verified_official_public_heading"
        || metadata["description_status"] != "unavailable_authorized_meanings_not_supplied"
    {
        return Err("Missing or unsupported public-label source metadata".into());
    }
    let mut keys = HashMap::new();
    let mut sources = HashMap::new();
    for (id, key) in data.gene_keys {
        let number = id.parse::<u8>()?;
        if !(1..=64).contains(&number) || id != number.to_string() || key.number != number {
            return Err(format!("Gene Key {id}: noncanonical ID or mismatched number").into());
        }
        if [&key.shadow, &key.gift, &key.siddhi]
            .iter()
            .any(|label| label.trim().is_empty())
        {
            return Err(format!("Gene Key {id}: blank public label").into());
        }
        if key.name != format!("Gene Key {number}")
            || !key.shadow_description.is_empty()
            || !key.gift_description.is_empty()
            || !key.siddhi_description.is_empty()
            || key.codon.is_some()
            || key.amino_acid.is_some()
            || key.physiology.is_some()
            || key.life_theme.is_some()
        {
            return Err(
                format!("Gene Key {id}: unsupported author meaning or biological field").into(),
            );
        }
        if key.programming_partner != opposition_partner(number) {
            return Err(
                format!("Gene Key {id}: partner is not exact Rave Mandala opposition").into(),
            );
        }
        let source = document["gene_keys"][&id]["provenance"].clone();
        let expected_url = format!("https://genekeys.com/gene-key-{number}/");
        let hash = source["source_response_sha256"]
            .as_str()
            .unwrap_or_default();
        if source["url"] != expected_url
            || source["status"] != 200
            || hash.len() != 64
            || !hash
                .bytes()
                .all(|b| b.is_ascii_hexdigit() && !b.is_ascii_uppercase())
            || source["retrieved_at"].as_str().is_none_or(str::is_empty)
        {
            return Err(format!("Gene Key {id}: missing or mismatched primary provenance").into());
        }
        keys.insert(number, key);
        sources.insert(number, source);
    }
    for number in 1..=64 {
        let key = keys
            .get(&number)
            .ok_or_else(|| format!("Missing Gene Key {number}"))?;
        let partner = key.programming_partner.unwrap();
        if partner == number
            || keys.get(&partner).and_then(|p| p.programming_partner) != Some(number)
        {
            return Err(format!("Gene Key {number}: nonreciprocal partner").into());
        }
    }
    Ok(PublicLabels {
        keys,
        sources,
        metadata,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn all_labels_present_with_unavailable_author_prose() {
        assert_eq!(gene_keys().len(), 64);
        for number in 1..=64 {
            let key = get_gene_key(number).unwrap();
            assert_eq!(key.number, number);
            assert!(!key.shadow.is_empty() && !key.gift.is_empty() && !key.siddhi.is_empty());
            assert!(
                key.shadow_description.is_empty()
                    && key.gift_description.is_empty()
                    && key.siddhi_description.is_empty()
            );
            assert!(key.physiology.is_none() && key.codon.is_none() && key.amino_acid.is_none());
        }
        assert_eq!(meaning_source_quality()["full_wisdom"], "unavailable");
    }

    #[test]
    fn rejects_missing_ids_numbers_sources_and_unverified_fields() {
        let original: Value = serde_json::from_str(PUBLIC_LABELS_JSON).unwrap();
        for mutation in 0..7 {
            let mut bad = original.clone();
            match mutation {
                0 => {
                    bad["gene_keys"].as_object_mut().unwrap().remove("64");
                }
                1 => bad["gene_keys"]["1"]["number"] = json!(2),
                2 => bad["gene_keys"]["1"]["shadow"] = json!(" "),
                3 => bad["gene_keys"]["1"]["programming_partner"] = json!(33),
                4 => {
                    bad["gene_keys"]["1"]["provenance"]["url"] =
                        json!("https://genekeys.com/gene-key-2/")
                }
                5 => bad["gene_keys"]["1"]["physiology"] = json!("Physiology 1"),
                _ => bad["gene_keys"]["1"]["shadow_description"] = json!("Unverified meaning"),
            }
            assert!(
                load_public_labels(&bad.to_string()).is_err(),
                "mutation {mutation}"
            );
        }
    }
}
