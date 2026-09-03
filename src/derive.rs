//! Derivation of database columns that are redundant with other columns.
//!
//! The BTyperDB dump ships a number of columns that carry no information of
//! their own: they are reformattings of a neighbouring column, or a lookup, or
//! a threshold rule. The ingest ignores those columns in the input file and
//! recomputes them with the functions below, so that the rules live in one
//! place instead of in whatever produced the TSV.
//!
//! Every rule here was checked against the full 14854-row dump of 2026-08-26
//! and reproduces the shipped column exactly; see the unit tests at the bottom
//! for the cases that pin down the awkward corners.
//!
//! The geography lookup needs the ISO 3166 tables, which are vendored in and
//! loaded by the `ingest` crate -- `GeoTable` is built there and passed in, so
//! that the tables do not end up in the wasm bundle.

use crate::{ColumnType, DatabaseColumn};
use std::collections::HashMap;

////////////////////////////////////////////////////////////
/// A column whose value the ingest computes rather than reads.
pub struct DerivedColumn {
    pub name: &'static str,
    pub column_type: ColumnType,
    /// For the matchcol_* family, the display column this is the searchable
    /// form of. None for columns that are user-facing in their own right.
    pub matchcol_for: Option<&'static str>,
}

////////////////////////////////////////////////////////////
/// Every column the ingest derives, and therefore ignores in the input file.
///
/// The matchcol_* entries also drive their own metadata -- see
/// `matchcol_metadata` -- so adding one here is enough to make it searchable;
/// it does not have to be repeated in btyperdb_include.json.
pub const DERIVED_COLUMNS: &[DerivedColumn] = &[
    m("matchcol_BTyper3_species", ColumnType::Text, "BTyper3_species(ANI)"),
    m("matchcol_BTyper3_subspecies", ColumnType::Text, "BTyper3_subspecies(ANI)"),
    m(
        "matchcol_BTyper3_Pseudo_Gene_Flow_Unit",
        ColumnType::Text,
        "BTyper3_Pseudo_Gene_Flow_Unit(ANI)",
    ),
    m(
        "matchcol_BTyper3_Closest_Type_Strain",
        ColumnType::Text,
        "BTyper3_Closest_Type_Strain(ANI)",
    ),
    m(
        "matchcol_BTyper3_anthrax_toxin",
        ColumnType::Integer,
        "BTyper3_anthrax_toxin(genes)",
    ),
    m(
        "matchcol_BTyper3_emetic_toxin_cereulide",
        ColumnType::Integer,
        "BTyper3_emetic_toxin_cereulide(genes)",
    ),
    m(
        "matchcol_BTyper3_diarrheal_toxin_Nhe",
        ColumnType::Integer,
        "BTyper3_diarrheal_toxin_Nhe(genes)",
    ),
    m(
        "matchcol_BTyper3_diarrheal_toxin_Hbl",
        ColumnType::Integer,
        "BTyper3_diarrheal_toxin_Hbl(genes)",
    ),
    m(
        "matchcol_BTyper3_diarrheal_toxin_CytK",
        ColumnType::Text,
        "BTyper3_diarrheal_toxin_CytK(top_hit)",
    ),
    m(
        "matchcol_BTyper3_sphingomyelinase_Sph",
        ColumnType::Text,
        "BTyper3_sphingomyelinase_Sph(gene)",
    ),
    m(
        "matchcol_BTyper3_capsule_Cap",
        ColumnType::Integer,
        "BTyper3_capsule_Cap(genes)",
    ),
    m(
        "matchcol_BTyper3_capsule_Has",
        ColumnType::Integer,
        "BTyper3_capsule_Has(genes)",
    ),
    m(
        "matchcol_BTyper3_capsule_Bps",
        ColumnType::Integer,
        "BTyper3_capsule_Bps(genes)",
    ),
    m("matchcol_BTyper3_Bt", ColumnType::Text, "BTyper3_Bt(genes)"),
    m(
        "matchcol_BTyper3_PubMLST_ST",
        ColumnType::Integer,
        "BTyper3_PubMLST_ST[clonal_complex](perfect_matches)",
    ),
    m(
        "matchcol_BTyper3_Adjusted_panC_Group",
        ColumnType::Text,
        "BTyper3_Adjusted_panC_Group(predicted_species)",
    ),
    // Computed, but ordinary columns as far as the user is concerned: whether
    // they are shown, printed or offered as a dropdown is a curation choice
    // that nothing about the derivation decides, so it stays in the metadata
    // file with every other user-facing column.
    d("Country(Code)", ColumnType::Text),
    d("Region(Code)", ColumnType::Text),
    d("Continent", ColumnType::Text),
    d("Genome_Quality", ColumnType::Text),
];

const fn m(name: &'static str, t: ColumnType, source: &'static str) -> DerivedColumn {
    DerivedColumn { name, column_type: t, matchcol_for: Some(source) }
}

const fn d(name: &'static str, t: ColumnType) -> DerivedColumn {
    DerivedColumn { name, column_type: t, matchcol_for: None }
}

////////////////////////////////////////////////////////////
/// Metadata for the matchcol_* columns, which does not appear in
/// btyperdb_include.json because none of it is a curation decision.
///
/// A matchcol exists for exactly one reason: to make a display column
/// filterable. So it is searchable, it is never a table column or part of an
/// export, and its type is whatever the derivation produces. Text ones get an
/// autocomplete list; numeric ones are searched as a range, where the
/// frontend never renders a list.
pub fn matchcol_metadata() -> Vec<DatabaseColumn> {
    DERIVED_COLUMNS
        .iter()
        .filter(|c| c.matchcol_for.is_some())
        .map(|c| DatabaseColumn {
            column_id: c.name.to_string(),
            column_type: c.column_type,
            default_from: String::new(),
            default_to: String::new(),
            show_by_default: false,
            dropdown: c.column_type == ColumnType::Text,
            display: false,
            search: true,
            print: false,
        })
        .collect()
}

/// Value used throughout BTyperDB for "we do not know"
pub const UNKNOWN: &str = "Unknown";

////////////////////////////////////////////////////////////
/// Strip one trailing parenthesised group, e.g. "cereus(98.8)" => "cereus"
fn strip_trailing_paren(v: &str) -> &str {
    let v = v.trim_end();
    if !v.ends_with(')') {
        return v;
    }
    // Only a group that opens at the same nesting depth counts, so that
    // "Group_VI(mycoides/paramycoides)" loses exactly one group.
    let mut depth = 0i32;
    for (i, c) in v.char_indices().rev() {
        match c {
            ')' => depth += 1,
            '(' => {
                depth -= 1;
                if depth == 0 {
                    return v[..i].trim_end();
                }
            }
            _ => {}
        }
    }
    v
}

/// Strip one trailing bracketed group, e.g. "1[No clonal complex]" => "1"
fn strip_trailing_bracket(v: &str) -> &str {
    let v = v.trim_end();
    if !v.ends_with(']') {
        return v;
    }
    let mut depth = 0i32;
    for (i, c) in v.char_indices().rev() {
        match c {
            ']' => depth += 1,
            '[' => {
                depth -= 1;
                if depth == 0 {
                    return v[..i].trim_end();
                }
            }
            _ => {}
        }
    }
    v
}

/// Contents of the last parenthesised group, e.g. "1/1(cytK-2)" => "cytK-2"
fn trailing_paren_content(v: &str) -> Option<&str> {
    let v = v.trim_end();
    if !v.ends_with(')') {
        return None;
    }
    let mut depth = 0i32;
    for (i, c) in v.char_indices().rev() {
        match c {
            ')' => depth += 1,
            '(' => {
                depth -= 1;
                if depth == 0 {
                    return Some(&v[i + 1..v.len() - 1]);
                }
            }
            _ => {}
        }
    }
    None
}

/// Leading integer of "n/m(...)" or "n(...)"
fn leading_count(v: &str) -> Option<i64> {
    let digits: String = v.trim_start().chars().take_while(|c| c.is_ascii_digit()).collect();
    digits.parse().ok()
}

////////////////////////////////////////////////////////////
/// Searchable form of an ANI-annotated taxonomy call.
///
/// Covers BTyper3_species(ANI), BTyper3_subspecies(ANI),
/// BTyper3_Pseudo_Gene_Flow_Unit(ANI) and BTyper3_Closest_Type_Strain(ANI):
/// drop the ANI value, and collapse anything BTyper3 flagged as not confident
/// (trailing "*") or left blank to "Unknown".
///
/// "cereus s.s.(97.09)"      => "cereus s.s."
/// "luti*(91.94)"            => "Unknown"
/// "(Type strain unknown)"   => "Unknown"
/// "No subspecies"           => "No subspecies"
pub fn matchcol_ani(v: &str) -> String {
    if v.contains('*') {
        return UNKNOWN.to_string();
    }
    let s = strip_trailing_paren(v).trim();
    if s.is_empty() {
        UNKNOWN.to_string()
    } else {
        s.to_string()
    }
}

////////////////////////////////////////////////////////////
/// Number of genes detected, from "n/m(gene;gene)".
///
/// Used for the anthrax toxin, cereulide, Nhe, Hbl and the three capsule
/// operons -- all searched as a numeric range.
pub fn matchcol_gene_count(v: &str) -> Option<i64> {
    leading_count(v)
}

////////////////////////////////////////////////////////////
/// Presence/absence of a gene, from "n/m(...)" or "n(...)".
///
/// Used for sphingomyelinase Sph ("1/1(sph)") and Bt ("0()").
pub fn matchcol_presence(v: &str) -> Option<String> {
    leading_count(v).map(|n| {
        if n == 0 { "Absent" } else { "Present" }.to_string()
    })
}

////////////////////////////////////////////////////////////
/// Top CytK hit, from "1/1(cytK-2)"; nothing detected => "Absent".
pub fn matchcol_cytk(v: &str) -> String {
    match trailing_paren_content(v) {
        Some(g) if !g.trim().is_empty() => g.trim().to_string(),
        _ => "Absent".to_string(),
    }
}

////////////////////////////////////////////////////////////
/// PubMLST sequence type, from "ST[clonal_complex](perfect/total)".
///
/// The ST is only reported when every allele matched perfectly; a partial
/// match is not a sequence type and becomes "Unknown".
///
/// "1[No clonal complex](7/7)"   => "1"
/// "1[No clonal complex](6/7)"   => "Unknown"
/// "Unknown(unknown ST)"         => "Unknown"
pub fn matchcol_pubmlst_st(v: &str) -> String {
    if v.contains('*') {
        return UNKNOWN.to_string();
    }
    let matches = match trailing_paren_content(v) {
        Some(m) => m,
        None => return UNKNOWN.to_string(),
    };
    let (perfect, total) = match matches.split_once('/') {
        Some(p) => p,
        None => return UNKNOWN.to_string(),
    };
    if perfect.trim() != total.trim() {
        return UNKNOWN.to_string();
    }
    let st = strip_trailing_bracket(strip_trailing_paren(v)).trim();
    if st.is_empty() {
        UNKNOWN.to_string()
    } else {
        st.to_string()
    }
}

////////////////////////////////////////////////////////////
/// panC group, from "Group_IV(cereus_sensu_stricto)".
///
/// Note this cannot share `matchcol_ani`: the "*" flag sits *after* the
/// parenthesised group here ("Group_I(pseudomycoides)*"), and some values
/// carry no group at all ("Group_bingmayongensis*").
pub fn matchcol_panc_group(v: &str) -> String {
    if v.contains('*') {
        return UNKNOWN.to_string();
    }
    let s = strip_trailing_paren(v).trim();
    if s.is_empty() {
        UNKNOWN.to_string()
    } else {
        s.to_string()
    }
}

////////////////////////////////////////////////////////////
/// Assembly QC verdict.
///
/// An assembly has to clear the CheckM and Quast bars to be usable at all;
/// among those that do, the ones whose reads were not convincingly Bacillota
/// are held back to "Good". GenBank assemblies have no Kraken report
/// (`kraken_bacillota == None`) and are never held back on that ground.
pub fn genome_quality(
    checkm_completeness: Option<f64>,
    checkm_contamination: Option<f64>,
    quast_contigs: Option<f64>,
    quast_n50: Option<f64>,
    kraken_bacillota: Option<f64>,
) -> &'static str {
    let passes = matches!(
        (checkm_completeness, checkm_contamination, quast_contigs, quast_n50),
        (Some(comp), Some(cont), Some(ctg), Some(n50))
            if comp >= 95.0 && cont < 5.0 && ctg <= 800.0 && n50 >= 20000.0
    );
    if !passes {
        return "Low_Quality";
    }
    match kraken_bacillota {
        Some(k) if k < 90.0 => "Good_Quality",
        _ => "High_Quality",
    }
}

////////////////////////////////////////////////////////////
/// ISO 3166 lookup, built by the ingest from the vendored tables
#[derive(Debug, Default)]
pub struct GeoTable {
    /// country name => ISO 3166-1 alpha-3
    country_to_alpha3: HashMap<String, String>,
    /// country name => ISO 3166-1 alpha-2 (absent for the non-ISO entries)
    country_to_alpha2: HashMap<String, String>,
    /// (alpha-2, subdivision name) => ISO 3166-2 code
    region_to_code: HashMap<(String, String), String>,
    /// ISO 3166-1 alpha-3 => continent
    alpha3_to_continent: HashMap<String, String>,
}

/// A country or region the ISO tables and the override file do not cover
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum GeoError {
    UnknownCountry(String),
    UnknownRegion(String, String),
    /// Country has no alpha-2, so its subdivisions cannot be looked up
    NoSubdivisions(String, String),
    /// Country resolved to an alpha-3 that continents.tsv does not cover
    UnknownContinent(String, String),
}

impl std::fmt::Display for GeoError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            GeoError::UnknownCountry(c) => {
                write!(f, "country {:?} is not in ISO 3166-1 or geo_overrides.tsv", c)
            }
            GeoError::UnknownRegion(c, r) => write!(
                f,
                "region {:?} of {:?} is not in ISO 3166-2 or geo_overrides.tsv",
                r, c
            ),
            GeoError::NoSubdivisions(c, r) => write!(
                f,
                "country {:?} has no ISO alpha-2 code, so region {:?} cannot be resolved",
                c, r
            ),
            GeoError::UnknownContinent(c, a3) => write!(
                f,
                "country {:?} (alpha-3 {}) is not in continents.tsv",
                c, a3
            ),
        }
    }
}

impl GeoTable {
    ////////////////////////////////////////////////////////////
    /// Build from the vendored tables. Each argument is the whole file.
    ///
    /// `iso1`:       name, alpha2, alpha3
    /// `iso2`:       code, name, parent
    /// `overrides`:  kind, key1, key2, value  (see geo_overrides.tsv)
    /// `continents`: alpha3, continent, name
    pub fn new(
        iso1: &str,
        iso2: &str,
        overrides: &str,
        continents: &str,
    ) -> Result<GeoTable, String> {
        let mut t = GeoTable::default();

        for (_n, f) in rows(continents, 3, "continents.tsv")? {
            t.alpha3_to_continent
                .insert(f[0].to_string(), f[1].to_string());
        }

        for (n, f) in rows(iso1, 3, "iso_3166-1.tsv")? {
            let _ = n;
            t.country_to_alpha3.insert(f[0].to_string(), f[2].to_string());
            t.country_to_alpha2.insert(f[0].to_string(), f[1].to_string());
        }

        // A subdivision name can occur twice within one country, once as a
        // top-level entry and once as a lower-level one nested under it (e.g.
        // ES-IB the autonomous community and ES-PM the province inside it).
        // Prefer the top-level entry -- the one without a parent. Names that
        // stay ambiguous after that are simply not inserted, so they surface
        // as UnknownRegion rather than resolving to an arbitrary code.
        let mut candidates: HashMap<(String, String), Vec<(String, bool)>> = HashMap::new();
        for (n, f) in rows(iso2, 3, "iso_3166-2.tsv")? {
            let (code, name, parent) = (f[0], f[1], f[2]);
            let alpha2 = match code.split('-').next() {
                Some(a) => a.to_string(),
                None => return Err(format!("iso_3166-2.tsv line {}: malformed code {:?}", n, code)),
            };
            candidates
                .entry((alpha2, name.to_string()))
                .or_default()
                .push((code.to_string(), parent.is_empty()));
        }
        for (key, cands) in candidates {
            let code = if cands.len() == 1 {
                Some(cands[0].0.clone())
            } else {
                let mut top = cands.iter().filter(|(_, is_top)| *is_top);
                match (top.next(), top.next()) {
                    (Some((c, _)), None) => Some(c.clone()),
                    _ => None,
                }
            };
            if let Some(code) = code {
                t.region_to_code.insert(key, code);
            }
        }

        for (n, f) in rows(overrides, 4, "geo_overrides.tsv")? {
            let (kind, key1, key2, value) = (f[0], f[1], f[2], f[3]);
            match kind {
                // A name BTyperDB uses for a country that is in ISO under
                // another name; the alpha-2 comes from the ISO entry.
                "country_alias" => {
                    let alpha2 = t
                        .country_to_alpha3
                        .iter()
                        .find(|(_, a3)| a3.as_str() == value)
                        .and_then(|(name, _)| t.country_to_alpha2.get(name).cloned());
                    match alpha2 {
                        Some(a2) => {
                            t.country_to_alpha2.insert(key1.to_string(), a2);
                        }
                        None => {
                            return Err(format!(
                                "geo_overrides.tsv line {}: country_alias {:?} points at \
                                 alpha-3 {:?}, which is not in iso_3166-1.tsv",
                                n, key1, value
                            ))
                        }
                    }
                    t.country_to_alpha3.insert(key1.to_string(), value.to_string());
                }
                // Not a country in ISO at all. No alpha-2, so no subdivisions.
                "country_extra" => {
                    t.country_to_alpha3.insert(key1.to_string(), value.to_string());
                }
                "region_alias" | "region_ambiguous" => {
                    t.region_to_code
                        .insert((key1.to_string(), key2.to_string()), value.to_string());
                }
                other => {
                    return Err(format!(
                        "geo_overrides.tsv line {}: unknown kind {:?}",
                        n, other
                    ))
                }
            }
        }

        Ok(t)
    }

    ////////////////////////////////////////////////////////////
    /// ISO 3166-1 alpha-3 for a country name. "Unknown" stays "Unknown".
    pub fn country_code(&self, country: &str) -> Result<String, GeoError> {
        if country == UNKNOWN {
            return Ok(UNKNOWN.to_string());
        }
        self.country_to_alpha3
            .get(country)
            .cloned()
            .ok_or_else(|| GeoError::UnknownCountry(country.to_string()))
    }

    ////////////////////////////////////////////////////////////
    /// ISO 3166-2 code for a subdivision, e.g. ("India", "Tamil Nadu")
    /// => "IN-TN". An unknown country or region gives "Unknown".
    pub fn region_code(&self, country: &str, region: &str) -> Result<String, GeoError> {
        if country == UNKNOWN || region == UNKNOWN {
            return Ok(UNKNOWN.to_string());
        }
        let alpha2 = self.country_to_alpha2.get(country).ok_or_else(|| {
            if self.country_to_alpha3.contains_key(country) {
                GeoError::NoSubdivisions(country.to_string(), region.to_string())
            } else {
                GeoError::UnknownCountry(country.to_string())
            }
        })?;
        self.region_to_code
            .get(&(alpha2.clone(), region.to_string()))
            .cloned()
            .ok_or_else(|| GeoError::UnknownRegion(country.to_string(), region.to_string()))
    }

    ////////////////////////////////////////////////////////////
    /// Continent a country sits on, e.g. "France" => "Europe".
    ///
    /// "Unknown" stays "Unknown" -- and note that is not the same as saying the
    /// continent is unknown. A handful of genomes record a place that is not a
    /// modern country ("Czechoslovakia", "Soviet Union", "Korea"), so the
    /// curator left Country as Unknown but still knew the continent. That fact
    /// lives only in the Continent column, so the ingest keeps the curated
    /// value in exactly that case rather than overwriting it.
    pub fn continent(&self, country: &str) -> Result<String, GeoError> {
        if country == UNKNOWN {
            return Ok(UNKNOWN.to_string());
        }
        let alpha3 = self
            .country_to_alpha3
            .get(country)
            .ok_or_else(|| GeoError::UnknownCountry(country.to_string()))?;
        self.alpha3_to_continent
            .get(alpha3)
            .cloned()
            .ok_or_else(|| GeoError::UnknownContinent(country.to_string(), alpha3.clone()))
    }
}

////////////////////////////////////////////////////////////
/// Split a vendored TSV into (line number, fields), skipping the header,
/// blank lines and '#' comments.
fn rows<'a>(
    src: &'a str,
    width: usize,
    what: &str,
) -> Result<Vec<(usize, Vec<&'a str>)>, String> {
    let mut out = Vec::new();
    for (i, line) in src.lines().enumerate() {
        let n = i + 1;
        if i == 0 || line.trim().is_empty() || line.starts_with('#') {
            continue;
        }
        let f: Vec<&str> = line.split('\t').collect();
        if f.len() != width {
            return Err(format!(
                "{} line {}: expected {} columns, found {}",
                what,
                n,
                width,
                f.len()
            ));
        }
        out.push((n, f));
    }
    Ok(out)
}

////////////////////////////////////////////////////////////
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ani_calls() {
        assert_eq!(matchcol_ani("cereus s.s.(97.09649658203125)"), "cereus s.s.");
        assert_eq!(matchcol_ani("No subspecies"), "No subspecies");
        assert_eq!(matchcol_ani("frankland(97.09)"), "frankland");
        // not a confident call
        assert_eq!(matchcol_ani("luti*(91.94908905029297)"), "Unknown");
        assert_eq!(matchcol_ani("UnknownGeneFlowUnit5*(95.94)"), "Unknown");
        // closest type strain, when there is none
        assert_eq!(matchcol_ani("(Type strain unknown)"), "Unknown");
    }

    #[test]
    fn gene_counts() {
        assert_eq!(matchcol_gene_count("0/3()"), Some(0));
        assert_eq!(matchcol_gene_count("3/3(cya;lef;pagA)"), Some(3));
        assert_eq!(matchcol_gene_count("2/9(bpsE;bpsH)"), Some(2));
        assert_eq!(matchcol_gene_count("nonsense"), None);
    }

    #[test]
    fn presence() {
        assert_eq!(matchcol_presence("1/1(sph)").as_deref(), Some("Present"));
        assert_eq!(matchcol_presence("0/1()").as_deref(), Some("Absent"));
        // Bt has no denominator
        assert_eq!(matchcol_presence("0()").as_deref(), Some("Absent"));
        assert_eq!(matchcol_presence("3(cry1;cry2;vip3)").as_deref(), Some("Present"));
    }

    #[test]
    fn cytk() {
        assert_eq!(matchcol_cytk("1/1(cytK-2)"), "cytK-2");
        assert_eq!(matchcol_cytk("1/1(cytK-1)"), "cytK-1");
        assert_eq!(matchcol_cytk("0/1()"), "Absent");
    }

    #[test]
    fn pubmlst_st() {
        assert_eq!(matchcol_pubmlst_st("1[No clonal complex](7/7)"), "1");
        assert_eq!(matchcol_pubmlst_st("142[CC142](7/7)"), "142");
        // a partial allele match is not a sequence type
        assert_eq!(matchcol_pubmlst_st("2576[No clonal complex](6/7)"), "Unknown");
        assert_eq!(matchcol_pubmlst_st("Unknown(unknown ST)"), "Unknown");
        assert_eq!(matchcol_pubmlst_st("Unknown(missing alleles)"), "Unknown");
    }

    #[test]
    fn panc_group() {
        assert_eq!(matchcol_panc_group("Group_IV(cereus_sensu_stricto)"), "Group_IV");
        assert_eq!(matchcol_panc_group("Group_III(mosaicus)"), "Group_III");
        // the "*" sits outside the group here, unlike the ANI columns
        assert_eq!(matchcol_panc_group("Group_I(pseudomycoides)*"), "Unknown");
        assert_eq!(matchcol_panc_group("Group_bingmayongensis*"), "Unknown");
    }

    #[test]
    fn quality() {
        // clears every bar, and the reads look like Bacillota
        assert_eq!(
            genome_quality(Some(99.43), Some(0.0), Some(194.0), Some(59260.0), Some(94.33)),
            "High_Quality"
        );
        // clears the assembly bars but the reads do not look like Bacillota
        assert_eq!(
            genome_quality(Some(99.0), Some(1.0), Some(200.0), Some(50000.0), Some(89.99)),
            "Good_Quality"
        );
        // GenBank assembly: no Kraken report, not held back for it
        assert_eq!(
            genome_quality(Some(99.0), Some(1.0), Some(200.0), Some(50000.0), None),
            "High_Quality"
        );
        // each bar on its own is enough to fail
        assert_eq!(
            genome_quality(Some(94.99), Some(1.0), Some(200.0), Some(50000.0), Some(99.0)),
            "Low_Quality"
        );
        assert_eq!(
            genome_quality(Some(99.0), Some(5.0), Some(200.0), Some(50000.0), Some(99.0)),
            "Low_Quality"
        );
        assert_eq!(
            genome_quality(Some(99.0), Some(1.0), Some(801.0), Some(50000.0), Some(99.0)),
            "Low_Quality"
        );
        assert_eq!(
            genome_quality(Some(99.0), Some(1.0), Some(200.0), Some(19999.0), Some(99.0)),
            "Low_Quality"
        );
        // a missing assembly metric cannot clear the bar
        assert_eq!(
            genome_quality(None, Some(1.0), Some(200.0), Some(50000.0), Some(99.0)),
            "Low_Quality"
        );
    }

    fn geo() -> GeoTable {
        GeoTable::new(
            "name\talpha2\talpha3\n\
             India\tIN\tIND\n\
             Spain\tES\tESP\n\
             Uzbekistan\tUZ\tUZB\n\
             United Kingdom of Great Britain and Northern Ireland\tGB\tGBR\n",
            "code\tname\tparent\n\
             IN-TN\tTamil Nadu\t\n\
             ES-IB\tIlles Balears\t\n\
             ES-PM\tIlles Balears\tES-IB\n\
             UZ-TK\tToshkent\t\n\
             UZ-TO\tToshkent\t\n\
             GB-WLS\tWales [Cymru GB-CYM]\t\n",
            "kind\tkey1\tkey2\tvalue\n\
             country_alias\tUnited Kingdom\t\tGBR\n\
             country_extra\tPacific Ocean\t\tOPC\n\
             region_alias\tGB\tWales\tGB-WLS\n\
             region_ambiguous\tUZ\tToshkent\tUZ-TK\n",
            "alpha3\tcontinent\tname\n\
             IND\tAsia\tIndia\n\
             ESP\tEurope\tSpain\n\
             UZB\tAsia\tUzbekistan\n\
             GBR\tEurope\tUnited Kingdom\n\
             OPC\tOcean\tPacific Ocean\n",
        )
        .unwrap()
    }

    #[test]
    fn continents() {
        let g = geo();
        assert_eq!(g.continent("India").unwrap(), "Asia");
        assert_eq!(g.continent("Spain").unwrap(), "Europe");
        // resolved through a country_alias
        assert_eq!(g.continent("United Kingdom").unwrap(), "Europe");
        // a coined non-ISO country still gets a continent
        assert_eq!(g.continent("Pacific Ocean").unwrap(), "Ocean");
        // Unknown country means the lookup has nothing to say; the ingest
        // keeps whatever the curator recorded in that case
        assert_eq!(g.continent("Unknown").unwrap(), "Unknown");
        assert_eq!(
            g.continent("Atlantis"),
            Err(GeoError::UnknownCountry("Atlantis".to_string()))
        );
    }

    #[test]
    fn country_codes() {
        let g = geo();
        assert_eq!(g.country_code("India").unwrap(), "IND");
        assert_eq!(g.country_code("Unknown").unwrap(), "Unknown");
        // short name via override
        assert_eq!(g.country_code("United Kingdom").unwrap(), "GBR");
        // not a country in ISO
        assert_eq!(g.country_code("Pacific Ocean").unwrap(), "OPC");
        assert_eq!(
            g.country_code("Atlantis"),
            Err(GeoError::UnknownCountry("Atlantis".to_string()))
        );
    }

    #[test]
    fn region_codes() {
        let g = geo();
        assert_eq!(g.region_code("India", "Tamil Nadu").unwrap(), "IN-TN");
        // nested duplicate name: the autonomous community wins over the province
        assert_eq!(g.region_code("Spain", "Illes Balears").unwrap(), "ES-IB");
        // two top-level entries share the name; only the override decides
        assert_eq!(g.region_code("Uzbekistan", "Toshkent").unwrap(), "UZ-TK");
        // ISO carries the alternate-language form, BTyperDB does not
        assert_eq!(g.region_code("United Kingdom", "Wales").unwrap(), "GB-WLS");
        // an unknown region of a known country is still Unknown
        assert_eq!(g.region_code("India", "Unknown").unwrap(), "Unknown");
        assert_eq!(g.region_code("Unknown", "Unknown").unwrap(), "Unknown");
        assert_eq!(
            g.region_code("India", "Atlantis"),
            Err(GeoError::UnknownRegion("India".to_string(), "Atlantis".to_string()))
        );
        // a coined country code has no subdivisions
        assert_eq!(
            g.region_code("Pacific Ocean", "Somewhere"),
            Err(GeoError::NoSubdivisions(
                "Pacific Ocean".to_string(),
                "Somewhere".to_string()
            ))
        );
    }

    #[test]
    fn region_code_prefix_agrees_with_country_code() {
        // The two derived geo columns have to stay consistent with each other.
        let g = geo();
        for (country, region) in [("India", "Tamil Nadu"), ("Spain", "Illes Balears")] {
            let cc = g.country_code(country).unwrap();
            let rc = g.region_code(country, region).unwrap();
            let alpha2 = rc.split('-').next().unwrap();
            assert_eq!(g.country_to_alpha3[country], cc);
            assert_eq!(g.country_to_alpha2[country], alpha2);
        }
    }
}
