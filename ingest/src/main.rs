//! Build meta/data.sqlite from meta/btyperdb.tsv.
//!
//! Replaces the old tosqlite.py. The difference that matters: the columns
//! listed in `my_web_app::derive::DERIVED_COLUMNS` are *not* taken from the
//! TSV. They are recomputed from the columns they were derived from, so that
//! the rules live in `src/derive.rs` rather than in whatever produced the TSV.
//! The values that were in the TSV are still read, but only to be compared
//! against what we compute; any disagreement is reported and, unless --force
//! is given, aborts the import before anything is written.
//!
//!     cargo run --release -p ingest -- /husky/vignesh/BTyperDB/latest/meta
//!
//! Reads  <dir>/btyperdb.tsv
//! Writes <dir>/data.sqlite   (refuses to overwrite unless --replace)

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::process::ExitCode;

use my_web_app::derive::{self, GeoTable, DERIVED_COLUMNS};
use rusqlite::{types::Value, Connection};

////////////////////////////////////////////////////////////
/// The ISO tables, vendored so the ingest is reproducible offline.
/// They live here rather than in the shared crate to keep them out of the
/// wasm bundle, which also depends on my-web-app.
const ISO_3166_1: &str = include_str!("../data/iso_3166-1.tsv");
const ISO_3166_2: &str = include_str!("../data/iso_3166-2.tsv");
const GEO_OVERRIDES: &str = include_str!("../data/geo_overrides.tsv");
const CONTINENTS: &str = include_str!("../data/continents.tsv");

/// Values that mean "no value" in the TSV, and become SQL NULL.
/// Matches what pandas.read_csv did in tosqlite.py for the columns that
/// actually occur in this dump, so the resulting database is unchanged in
/// that respect.
const NULL_VALUES: &[&str] = &["", "NA"];

const TABLE: &str = "straindata";
const PRIMARY_KEY: &str = "BTyperDB_ID";

////////////////////////////////////////////////////////////
/// Storage type of a column, inferred from the data as pandas did
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ColType {
    Integer,
    Real,
    Text,
}

impl ColType {
    fn sql(&self) -> &'static str {
        match self {
            ColType::Integer => "INTEGER",
            ColType::Real => "REAL",
            ColType::Text => "TEXT",
        }
    }
}

fn is_null(v: &str) -> bool {
    NULL_VALUES.contains(&v)
}

/// Parse a numeric cell, treating the NULL sentinels as missing
fn num(v: &str) -> Option<f64> {
    if is_null(v) {
        None
    } else {
        v.parse().ok()
    }
}

////////////////////////////////////////////////////////////
fn main() -> ExitCode {
    let mut args = std::env::args().skip(1);
    let mut dir: Option<PathBuf> = None;
    let mut force = false;
    let mut replace = false;
    while let Some(a) = args.next() {
        match a.as_str() {
            "--force" => force = true,
            "--replace" => replace = true,
            "-h" | "--help" => {
                eprintln!(
                    "usage: ingest <meta-dir> [--replace] [--force]\n\
                     \n\
                     <meta-dir>  directory holding btyperdb.tsv; data.sqlite is written there\n\
                     --replace   overwrite an existing data.sqlite\n\
                     --force     write even if a recomputed column disagrees with the TSV"
                );
                return ExitCode::SUCCESS;
            }
            other if other.starts_with('-') => {
                eprintln!("unknown option {:?}, try --help", other);
                return ExitCode::FAILURE;
            }
            other => dir = Some(PathBuf::from(other)),
        }
    }
    let dir = match dir {
        Some(d) => d,
        None => {
            eprintln!("usage: ingest <meta-dir> [--replace] [--force]");
            return ExitCode::FAILURE;
        }
    };

    match run(&dir, force, replace) {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("\nerror: {}", e);
            ExitCode::FAILURE
        }
    }
}

////////////////////////////////////////////////////////////
fn run(dir: &Path, force: bool, replace: bool) -> Result<(), String> {
    let path_tsv = dir.join("btyperdb.tsv");
    let path_sql = dir.join("data.sqlite");

    if path_sql.exists() && !replace {
        return Err(format!(
            "{} already exists; back it up and pass --replace",
            path_sql.display()
        ));
    }

    let geo = GeoTable::new(ISO_3166_1, ISO_3166_2, GEO_OVERRIDES, CONTINENTS)?;

    ////////// Read the whole TSV. It is ~15 MB, so this is not worth streaming.
    println!("reading {}", path_tsv.display());
    let mut reader = csv::ReaderBuilder::new()
        .delimiter(b'\t')
        .quoting(false) // the dump is not quoted; free text contains bare quotes
        .flexible(false)
        .from_path(&path_tsv)
        .map_err(|e| format!("could not open {}: {}", path_tsv.display(), e))?;

    let header: Vec<String> = reader
        .headers()
        .map_err(|e| format!("could not read header: {}", e))?
        .iter()
        .map(|s| s.to_string())
        .collect();
    let index: BTreeMap<&str, usize> = header
        .iter()
        .enumerate()
        .map(|(i, n)| (n.as_str(), i))
        .collect();

    let mut rows: Vec<Vec<String>> = Vec::new();
    for (n, rec) in reader.records().enumerate() {
        let rec = rec.map_err(|e| format!("line {}: {}", n + 2, e))?;
        rows.push(rec.iter().map(|s| s.to_string()).collect());
    }
    println!("  {} rows, {} columns", rows.len(), header.len());

    ////////// Every column we are about to compute must be present, both so
    ////////// that the schema keeps its shape and so that we can verify.
    let missing: Vec<&str> = DERIVED_COLUMNS
        .iter()
        .map(|c| c.name)
        .filter(|c| !index.contains_key(*c))
        .collect();
    if !missing.is_empty() {
        return Err(format!(
            "the TSV is missing derived columns that the schema expects: {:?}",
            missing
        ));
    }

    ////////// Recompute the derived columns
    println!("\nrecomputing {} derived columns", DERIVED_COLUMNS.len());
    let mut geo_errors: Vec<String> = Vec::new();
    for (n, row) in rows.iter_mut().enumerate() {
        let line = n + 2;
        let src = |col: &str| -> Result<String, String> {
            index
                .get(col)
                .map(|i| row[*i].clone())
                .ok_or_else(|| format!("the TSV has no column {:?}", col))
        };

        let mut computed: Vec<(&str, String)> = Vec::new();

        //////////// The four ANI-annotated taxonomy calls
        for (dst, s) in [
            ("matchcol_BTyper3_species", "BTyper3_species(ANI)"),
            ("matchcol_BTyper3_subspecies", "BTyper3_subspecies(ANI)"),
            (
                "matchcol_BTyper3_Pseudo_Gene_Flow_Unit",
                "BTyper3_Pseudo_Gene_Flow_Unit(ANI)",
            ),
            (
                "matchcol_BTyper3_Closest_Type_Strain",
                "BTyper3_Closest_Type_Strain(ANI)",
            ),
        ] {
            computed.push((dst, derive::matchcol_ani(&src(s)?)));
        }

        //////////// Gene counts, searched as a numeric range
        for (dst, s) in [
            ("matchcol_BTyper3_anthrax_toxin", "BTyper3_anthrax_toxin(genes)"),
            (
                "matchcol_BTyper3_emetic_toxin_cereulide",
                "BTyper3_emetic_toxin_cereulide(genes)",
            ),
            (
                "matchcol_BTyper3_diarrheal_toxin_Nhe",
                "BTyper3_diarrheal_toxin_Nhe(genes)",
            ),
            (
                "matchcol_BTyper3_diarrheal_toxin_Hbl",
                "BTyper3_diarrheal_toxin_Hbl(genes)",
            ),
            ("matchcol_BTyper3_capsule_Cap", "BTyper3_capsule_Cap(genes)"),
            ("matchcol_BTyper3_capsule_Has", "BTyper3_capsule_Has(genes)"),
            ("matchcol_BTyper3_capsule_Bps", "BTyper3_capsule_Bps(genes)"),
        ] {
            let raw = src(s)?;
            let v = derive::matchcol_gene_count(&raw).ok_or_else(|| {
                format!("line {}: cannot read a gene count from {} = {:?}", line, s, raw)
            })?;
            computed.push((dst, v.to_string()));
        }

        //////////// Presence/absence
        for (dst, s) in [
            (
                "matchcol_BTyper3_sphingomyelinase_Sph",
                "BTyper3_sphingomyelinase_Sph(gene)",
            ),
            ("matchcol_BTyper3_Bt", "BTyper3_Bt(genes)"),
        ] {
            let raw = src(s)?;
            let v = derive::matchcol_presence(&raw).ok_or_else(|| {
                format!("line {}: cannot read a gene count from {} = {:?}", line, s, raw)
            })?;
            computed.push((dst, v));
        }

        //////////// The three one-offs
        computed.push((
            "matchcol_BTyper3_diarrheal_toxin_CytK",
            derive::matchcol_cytk(&src("BTyper3_diarrheal_toxin_CytK(top_hit)")?),
        ));
        computed.push((
            "matchcol_BTyper3_PubMLST_ST",
            derive::matchcol_pubmlst_st(&src("BTyper3_PubMLST_ST[clonal_complex](perfect_matches)")?),
        ));
        computed.push((
            "matchcol_BTyper3_Adjusted_panC_Group",
            derive::matchcol_panc_group(&src("BTyper3_Adjusted_panC_Group(predicted_species)")?),
        ));

        //////////// Assembly QC verdict
        computed.push((
            "Genome_Quality",
            derive::genome_quality(
                num(&src("CheckM_Completeness")?),
                num(&src("CheckM_Contamination")?),
                num(&src("Quast_Contigs")?),
                num(&src("Quast_N50")?),
                num(&src("Kraken_Phylum(Bacillota)")?),
            )
            .to_string(),
        ));

        //////////// Geography. A lookup miss is collected rather than thrown,
        //////////// so one run lists every country and region that needs an
        //////////// entry in geo_overrides.tsv instead of one per run.
        let country = src("Country")?;
        let region = src("Region")?;
        match geo.country_code(&country) {
            Ok(v) => computed.push(("Country(Code)", v)),
            Err(e) => {
                geo_errors.push(format!("line {}: {}", line, e));
                computed.push(("Country(Code)", derive::UNKNOWN.to_string()));
            }
        }
        match geo.region_code(&country, &region) {
            Ok(v) => computed.push(("Region(Code)", v)),
            Err(e) => {
                geo_errors.push(format!("line {}: {}", line, e));
                computed.push(("Region(Code)", derive::UNKNOWN.to_string()));
            }
        }

        //////////// Continent follows from the country -- except that a
        //////////// genome whose recorded place is not a modern country
        //////////// ("Czechoslovakia", "Soviet Union", "Korea") has
        //////////// Country = Unknown while the curator still knew the
        //////////// continent. That fact exists nowhere else, so it is kept.
        match geo.continent(&country) {
            Ok(v) => {
                let curated = src("Continent")?;
                let keep_curated = v == derive::UNKNOWN && curated != derive::UNKNOWN;
                computed.push(("Continent", if keep_curated { curated } else { v }));
            }
            Err(e) => {
                geo_errors.push(format!("line {}: {}", line, e));
                computed.push(("Continent", src("Continent")?));
            }
        }

        for (col, value) in computed {
            row[index[col]] = value;
        }
    }
    ////////// Report lookup misses. These mean the vendored ISO tables have
    ////////// fallen behind the data, and are never acceptable silently.
    if !geo_errors.is_empty() {
        let mut uniq: Vec<&String> = geo_errors.iter().collect();
        uniq.sort();
        uniq.dedup_by(|a, b| a.split(": ").nth(1) == b.split(": ").nth(1));
        eprintln!("\n{} geo lookup failures, {} distinct:", geo_errors.len(), uniq.len());
        for e in uniq.iter().take(40) {
            eprintln!("  {}", e);
        }
        if uniq.len() > 40 {
            eprintln!("  ... and {} more", uniq.len() - 40);
        }
        return Err("add the missing entries to ingest/data/geo_overrides.tsv".to_string());
    }

    ////////// Compare against what the TSV shipped. We do not use these
    ////////// values, but a disagreement means either the upstream pipeline
    ////////// or our rules changed, and somebody should look at it.
    //
    // Re-read the file rather than keeping a second copy of 15 MB in memory.
    println!("\nverifying against the columns shipped in the TSV");
    let mut original = csv::ReaderBuilder::new()
        .delimiter(b'\t')
        .quoting(false)
        .from_path(&path_tsv)
        .map_err(|e| format!("could not reopen {}: {}", path_tsv.display(), e))?;
    let mut diffs: BTreeMap<&str, (usize, Vec<String>)> = BTreeMap::new();
    for (n, rec) in original.records().enumerate() {
        let rec = rec.map_err(|e| format!("line {}: {}", n + 2, e))?;
        for col in DERIVED_COLUMNS.iter().map(|c| c.name) {
            let i = index[col];
            let (was, now) = (&rec[i], rows[n][i].as_str());
            if was != now {
                let e = diffs.entry(col).or_insert((0, Vec::new()));
                e.0 += 1;
                if e.1.len() < 3 {
                    e.1.push(format!("line {}: {:?} -> {:?}", n + 2, was, now));
                }
            }
        }
    }
    if diffs.is_empty() {
        println!("  all {} derived columns reproduce the TSV exactly", DERIVED_COLUMNS.len());
    } else {
        eprintln!("  {} column(s) disagree with the TSV:", diffs.len());
        for (col, (count, examples)) in &diffs {
            eprintln!("    {} -- {} row(s)", col, count);
            for e in examples {
                eprintln!("        {}", e);
            }
        }
        if !force {
            return Err(
                "recomputed values differ from the TSV; check the rules in src/derive.rs, \
                 or pass --force to write the recomputed values anyway"
                    .to_string(),
            );
        }
        eprintln!("  --force given: writing the recomputed values");
    }

    ////////// Infer storage types, the way pandas did, so the schema and the
    ////////// numeric searches in the server keep working unchanged.
    let types: Vec<ColType> = (0..header.len())
        .map(|i| {
            let vals = rows.iter().map(|r| r[i].as_str()).filter(|v| !is_null(v));
            let mut any = false;
            let mut all_int = true;
            let mut all_real = true;
            for v in vals {
                any = true;
                if v.parse::<i64>().is_err() {
                    all_int = false;
                }
                if v.parse::<f64>().is_err() {
                    all_real = false;
                }
                if !all_int && !all_real {
                    break;
                }
            }
            if !any {
                ColType::Text
            } else if all_int {
                ColType::Integer
            } else if all_real {
                ColType::Real
            } else {
                ColType::Text
            }
        })
        .collect();

    ////////// Write the database
    if path_sql.exists() {
        std::fs::remove_file(&path_sql)
            .map_err(|e| format!("could not remove {}: {}", path_sql.display(), e))?;
    }
    println!("\nwriting {}", path_sql.display());
    let mut conn = Connection::open(&path_sql)
        .map_err(|e| format!("could not create {}: {}", path_sql.display(), e))?;

    let cols_sql: Vec<String> = header
        .iter()
        .zip(&types)
        .map(|(name, t)| {
            let pk = if name == PRIMARY_KEY { " PRIMARY KEY" } else { "" };
            format!("  {} {}{}", quote_ident(name), t.sql(), pk)
        })
        .collect();
    let create = format!(
        "CREATE TABLE {} (\n{}\n)",
        quote_ident(TABLE),
        cols_sql.join(",\n")
    );
    conn.execute(&create, []).map_err(|e| format!("{}\n{}", create, e))?;

    let placeholders = vec!["?"; header.len()].join(",");
    let insert = format!(
        "INSERT INTO {} VALUES ({})",
        quote_ident(TABLE),
        placeholders
    );

    let tx = conn.transaction().map_err(|e| e.to_string())?;
    {
        let mut stmt = tx.prepare(&insert).map_err(|e| e.to_string())?;
        for (n, row) in rows.iter().enumerate() {
            let values: Vec<Value> = row
                .iter()
                .zip(&types)
                .map(|(v, t)| {
                    if is_null(v) {
                        Value::Null
                    } else {
                        match t {
                            ColType::Integer => {
                                v.parse::<i64>().map(Value::Integer).unwrap_or(Value::Null)
                            }
                            ColType::Real => {
                                v.parse::<f64>().map(Value::Real).unwrap_or(Value::Null)
                            }
                            ColType::Text => Value::Text(v.clone()),
                        }
                    }
                })
                .collect();
            stmt.execute(rusqlite::params_from_iter(values.iter()))
                .map_err(|e| format!("line {}: {}", n + 2, e))?;
        }
    }
    tx.commit().map_err(|e| e.to_string())?;

    let n: i64 = conn
        .query_row(&format!("SELECT count(*) FROM {}", quote_ident(TABLE)), [], |r| r.get(0))
        .map_err(|e| e.to_string())?;
    println!("  {} rows written", n);
    let numeric: Vec<&String> = header
        .iter()
        .zip(&types)
        .filter(|(_, t)| **t != ColType::Text)
        .map(|(n, _)| n)
        .collect();
    println!("  {} numeric columns: {:?}", numeric.len(), numeric);

    Ok(())
}

////////////////////////////////////////////////////////////
/// SQL identifier quoting. The column names contain (), [] and even a '/',
/// so nothing here can go unquoted.
fn quote_ident(s: &str) -> String {
    format!("\"{}\"", s.replace('"', "\"\""))
}
