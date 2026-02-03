//! Nakshatra syllables database

use super::types::NakshatraSyllables;

/// Get syllables for all nakshatras
pub fn get_all_nakshatra_syllables() -> Vec<NakshatraSyllables> {
    vec![
        NakshatraSyllables {
            nakshatra: "Ashwini".to_string(),
            pada1: vec!["Chu".to_string()],
            pada2: vec!["Che".to_string()],
            pada3: vec!["Cho".to_string()],
            pada4: vec!["La".to_string()],
        },
        NakshatraSyllables {
            nakshatra: "Bharani".to_string(),
            pada1: vec!["Li".to_string()],
            pada2: vec!["Lu".to_string()],
            pada3: vec!["Le".to_string()],
            pada4: vec!["Lo".to_string()],
        },
        NakshatraSyllables {
            nakshatra: "Krittika".to_string(),
            pada1: vec!["A".to_string()],
            pada2: vec!["I".to_string(), "Ee".to_string()],
            pada3: vec!["U".to_string(), "Oo".to_string()],
            pada4: vec!["E".to_string(), "Ay".to_string()],
        },
        NakshatraSyllables {
            nakshatra: "Rohini".to_string(),
            pada1: vec!["O".to_string()],
            pada2: vec!["Va".to_string(), "Wa".to_string()],
            pada3: vec!["Vi".to_string(), "Wi".to_string()],
            pada4: vec!["Vu".to_string(), "Wu".to_string()],
        },
        NakshatraSyllables {
            nakshatra: "Mrigashira".to_string(),
            pada1: vec!["Ve".to_string(), "We".to_string()],
            pada2: vec!["Vo".to_string(), "Wo".to_string()],
            pada3: vec!["Ka".to_string()],
            pada4: vec!["Ki".to_string()],
        },
        NakshatraSyllables {
            nakshatra: "Ardra".to_string(),
            pada1: vec!["Ku".to_string()],
            pada2: vec!["Gha".to_string()],
            pada3: vec!["Ng".to_string(), "Da".to_string()],
            pada4: vec!["Chha".to_string()],
        },
        NakshatraSyllables {
            nakshatra: "Punarvasu".to_string(),
            pada1: vec!["Ke".to_string()],
            pada2: vec!["Ko".to_string()],
            pada3: vec!["Ha".to_string()],
            pada4: vec!["Hi".to_string()],
        },
        NakshatraSyllables {
            nakshatra: "Pushya".to_string(),
            pada1: vec!["Hu".to_string()],
            pada2: vec!["He".to_string()],
            pada3: vec!["Ho".to_string()],
            pada4: vec!["Da".to_string()],
        },
        NakshatraSyllables {
            nakshatra: "Ashlesha".to_string(),
            pada1: vec!["Di".to_string()],
            pada2: vec!["Du".to_string()],
            pada3: vec!["De".to_string()],
            pada4: vec!["Do".to_string()],
        },
        NakshatraSyllables {
            nakshatra: "Magha".to_string(),
            pada1: vec!["Ma".to_string()],
            pada2: vec!["Mi".to_string()],
            pada3: vec!["Mu".to_string()],
            pada4: vec!["Me".to_string()],
        },
        NakshatraSyllables {
            nakshatra: "Purva Phalguni".to_string(),
            pada1: vec!["Mo".to_string()],
            pada2: vec!["Ta".to_string()],
            pada3: vec!["Ti".to_string()],
            pada4: vec!["Tu".to_string()],
        },
        NakshatraSyllables {
            nakshatra: "Uttara Phalguni".to_string(),
            pada1: vec!["Te".to_string()],
            pada2: vec!["To".to_string()],
            pada3: vec!["Pa".to_string()],
            pada4: vec!["Pi".to_string()],
        },
        NakshatraSyllables {
            nakshatra: "Hasta".to_string(),
            pada1: vec!["Pu".to_string()],
            pada2: vec!["Sha".to_string()],
            pada3: vec!["Na".to_string()],
            pada4: vec!["Tha".to_string()],
        },
        NakshatraSyllables {
            nakshatra: "Chitra".to_string(),
            pada1: vec!["Pe".to_string()],
            pada2: vec!["Po".to_string()],
            pada3: vec!["Ra".to_string()],
            pada4: vec!["Ri".to_string()],
        },
        NakshatraSyllables {
            nakshatra: "Swati".to_string(),
            pada1: vec!["Ru".to_string()],
            pada2: vec!["Re".to_string()],
            pada3: vec!["Ro".to_string()],
            pada4: vec!["Ta".to_string()],
        },
        NakshatraSyllables {
            nakshatra: "Vishakha".to_string(),
            pada1: vec!["Ti".to_string()],
            pada2: vec!["Tu".to_string()],
            pada3: vec!["Te".to_string()],
            pada4: vec!["To".to_string()],
        },
        NakshatraSyllables {
            nakshatra: "Anuradha".to_string(),
            pada1: vec!["Na".to_string()],
            pada2: vec!["Ni".to_string()],
            pada3: vec!["Nu".to_string()],
            pada4: vec!["Ne".to_string()],
        },
        NakshatraSyllables {
            nakshatra: "Jyeshtha".to_string(),
            pada1: vec!["No".to_string()],
            pada2: vec!["Ya".to_string()],
            pada3: vec!["Yi".to_string()],
            pada4: vec!["Yu".to_string()],
        },
        NakshatraSyllables {
            nakshatra: "Mula".to_string(),
            pada1: vec!["Ye".to_string()],
            pada2: vec!["Yo".to_string()],
            pada3: vec!["Bha".to_string()],
            pada4: vec!["Bhi".to_string()],
        },
        NakshatraSyllables {
            nakshatra: "Purva Ashadha".to_string(),
            pada1: vec!["Bhu".to_string()],
            pada2: vec!["Dha".to_string()],
            pada3: vec!["Pha".to_string()],
            pada4: vec!["Dha".to_string()],
        },
        NakshatraSyllables {
            nakshatra: "Uttara Ashadha".to_string(),
            pada1: vec!["Bhe".to_string()],
            pada2: vec!["Bho".to_string()],
            pada3: vec!["Ja".to_string()],
            pada4: vec!["Ji".to_string()],
        },
        NakshatraSyllables {
            nakshatra: "Shravana".to_string(),
            pada1: vec!["Ju".to_string(), "Khi".to_string()],
            pada2: vec!["Je".to_string(), "Khu".to_string()],
            pada3: vec!["Jo".to_string(), "Khe".to_string()],
            pada4: vec!["Gha".to_string(), "Kho".to_string()],
        },
        NakshatraSyllables {
            nakshatra: "Dhanishta".to_string(),
            pada1: vec!["Ga".to_string()],
            pada2: vec!["Gi".to_string()],
            pada3: vec!["Gu".to_string()],
            pada4: vec!["Ge".to_string()],
        },
        NakshatraSyllables {
            nakshatra: "Shatabhisha".to_string(),
            pada1: vec!["Go".to_string()],
            pada2: vec!["Sa".to_string()],
            pada3: vec!["Si".to_string()],
            pada4: vec!["Su".to_string()],
        },
        NakshatraSyllables {
            nakshatra: "Purva Bhadrapada".to_string(),
            pada1: vec!["Se".to_string()],
            pada2: vec!["So".to_string()],
            pada3: vec!["Da".to_string()],
            pada4: vec!["Di".to_string()],
        },
        NakshatraSyllables {
            nakshatra: "Uttara Bhadrapada".to_string(),
            pada1: vec!["Du".to_string()],
            pada2: vec!["Tha".to_string()],
            pada3: vec!["Jha".to_string()],
            pada4: vec!["Da".to_string()],
        },
        NakshatraSyllables {
            nakshatra: "Revati".to_string(),
            pada1: vec!["De".to_string()],
            pada2: vec!["Do".to_string()],
            pada3: vec!["Cha".to_string()],
            pada4: vec!["Chi".to_string()],
        },
    ]
}

/// Get syllables for a specific nakshatra
pub fn get_syllables_for_nakshatra(nakshatra: &str, pada: u8) -> Vec<String> {
    let all = get_all_nakshatra_syllables();
    
    all.iter()
        .find(|n| n.nakshatra.eq_ignore_ascii_case(nakshatra))
        .map(|n| n.get_syllables_for_pada(pada).to_vec())
        .unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_all_nakshatras() {
        let all = get_all_nakshatra_syllables();
        assert_eq!(all.len(), 27);
    }

    #[test]
    fn test_specific_nakshatra() {
        let syllables = get_syllables_for_nakshatra("Rohini", 1);
        assert!(syllables.contains(&"O".to_string()));
    }
}
