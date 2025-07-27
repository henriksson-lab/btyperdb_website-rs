use rusqlite::{Connection, Result};

type DatabaseHistogram = Vec<(String,i32)>;


////////////////////////////////////////////////////////////
/// 
pub fn query_get_strain_count(
    conn: &Connection
) -> Result<i32> {

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
pub fn query_histogram(
    conn: &Connection,
    colname: &String
) -> Result<DatabaseHistogram> {

    let mut stmt = conn.prepare(format!("SELECT `{}` as grp, count(*) as cnt FROM straindata group by grp ORDER BY cnt DESC", colname).as_str())?; ////////// TODO: escape name of column?

    let cnts = stmt.query_map([], |row| {
        let name:String = row.get(0)?;
        let cnt:i32 = row.get(1)?;
        Ok((name, cnt))
    })?;

    let mut outlist=Vec::new();
    for name_cnt in cnts {
        if let Ok(name_cnt) = name_cnt {
            outlist.push(name_cnt);
        }
    }

    println!("{:?}",outlist);

    Ok(outlist)
}

