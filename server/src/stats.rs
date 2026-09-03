use my_web_app::OneStats;
use rusqlite::{types::ValueRef, Connection, Result};

type DatabaseHistogram = Vec<(String, i32)>;

////////////////////////////////////////////////////////////
/// Render a cell as the string the frontend groups and shows by.
///
/// Not every groupable column is TEXT: the searchable matchcol_* gene counts
/// are stored as INTEGER, so reading one straight into a String fails and the
/// column silently comes back empty. NULL is not a value to offer, and is
/// skipped rather than turned into an empty entry.
fn group_value(v: ValueRef) -> Option<String> {
    match v {
        ValueRef::Null => None,
        ValueRef::Integer(i) => Some(i.to_string()),
        ValueRef::Real(f) => Some(f.to_string()),
        ValueRef::Text(t) => Some(String::from_utf8_lossy(t).to_string()),
        ValueRef::Blob(_) => None,
    }
}

////////////////////////////////////////////////////////////
///
pub fn query_get_strain_count(conn: &Connection) -> Result<i32> {
    let mut stmt = conn.prepare("SELECT count(*) as cnt FROM straindata")?;

    let cnts = stmt.query_map([], |row| {
        let val = row.get(0)?;
        Ok(val)
    })?;

    let mut ret_cnt: i32 = -1;
    for cnt in cnts {
        if let Ok(cnt) = cnt {
            ret_cnt = cnt;
        }
    }
    Ok(ret_cnt)
}

////////////////////////////////////////////////////////////
///
pub fn query_histogram(conn: &Connection, colname: &String) -> Result<DatabaseHistogram> {
    let mut stmt = conn.prepare(
        format!(
            "SELECT `{}` as grp, count(*) as cnt FROM straindata group by grp ORDER BY cnt DESC",
            colname
        )
        .as_str(),
    )?; ////////// TODO: escape name of column?

    let cnts = stmt.query_map([], |row| {
        let name = group_value(row.get_ref(0)?);
        let cnt: i32 = row.get(1)?;
        Ok(name.map(|name| (name, cnt)))
    })?;

    let mut outlist = Vec::new();
    for name_cnt in cnts {
        if let Ok(Some(name_cnt)) = name_cnt {
            outlist.push(name_cnt);
        }
    }

    //println!("{:?}",outlist);

    Ok(outlist)
}

////////////////////////////////////////////////////////////
///
pub fn make_stats(conn: &Connection, showname: &String, colname: &String) -> Result<OneStats> {
    Ok(OneStats {
        name: showname.clone(),
        hist: query_histogram(conn, colname)?,
    })
}

////////////////////////////////////////////////////////////
///
pub fn query_dropdown(conn: &Connection, colname: &String) -> Result<Vec<String>> {
    let mut stmt = conn.prepare(
        format!(
            "SELECT distinct `{}` as grp FROM straindata ORDER BY grp",
            colname
        )
        .as_str(),
    )?; ////////// TODO: escape name of column?

    let cnts = stmt.query_map([], |row| Ok(group_value(row.get_ref(0)?)))?;

    let mut outlist = Vec::new();
    for name_cnt in cnts {
        if let Ok(Some(name_cnt)) = name_cnt {
            outlist.push(name_cnt);
        }
    }

    //println!("{:?}",outlist);

    Ok(outlist)
}
