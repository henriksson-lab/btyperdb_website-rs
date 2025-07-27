
////////////////////////////////////////////////////////////
/// x
pub fn sql_stringarg_to_num(s: &String) -> String {
    if s.parse::<f64>().is_ok() {
        s.clone()
    } else {
        panic!("bad value")
    }
}

////////////////////////////////////////////////////////////
/// x
pub fn sql_stringarg_escape(s: &String) -> String {
    let mut out =String::new();
    for c in s.chars() {
        if c.is_ascii_alphanumeric() || c.is_whitespace() || c=='-' || c=='_' || c=='.' || c==',' {
            out.push(c);
        } else {
            println!("!!!!!!!!!!! unhandled char {}",c);
        }
    }
    out
}

////////////////////////////////////////////////////////////
/// x
pub fn sql_check_name(s: &String) -> String {
    let mut out =String::new();
    if s.len()==0 {
        panic!("invalid name as it is empty");
    }

    //let valid_char="abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVQXYZ0123456789_()[]".as_bytes();  
    for c in s.chars() {
        if c==' ' {
            out.push('_');
        } /*else if !valid_char.contains(b) {
            panic!("invalid character in name {}", b); ///////////////// fix
        } */ else {
            out.push(c);
        }
    }
    out
}




////////////////////////////////////////////////////////////
/// x
pub fn clean_btyper_id(s: &String) -> String {
    let mut out =String::new();
    for c in s.chars() {
        if c.is_ascii_alphanumeric() || c.is_whitespace() || c=='-' || c=='_' || c=='.' || c==',' {
            out.push(c);
        } else {
            println!("!!!!!!!!!!! unhandled strainid char {}",c);
        }
    }
    out

}
