use lazy_static::lazy_static;
use regex::Regex;

pub fn parse_macros(source: String) -> String {
    lazy_static! {
        static ref FINDER: Regex = Regex::new("#define ([\\w]+) ([\\w*]+)").unwrap();
    }
    
    let matches: Vec<(String, String, String)> = FINDER.captures_iter(&source)
        .map(|e| (e.get(1).unwrap().as_str().to_string(),
                  e.get(2).unwrap().as_str().to_string(),
                  e.get(0).unwrap().as_str().to_string()))
        .collect();
    
    let mut res = source;
    
    for m in &matches {
        res = res.replace(&m.2, "");
    }
    
    for m in &matches {
        res = res.replace(&m.0, &m.1);
    }
    
    res
}

pub fn find_add_square(source: String) -> (String, u8) {
    lazy_static! {
        static ref FINDER: Regex = Regex::new("#add_square ([\\d]+)").unwrap();
    }
    
    let num;
    let remove;
    
    match FINDER.captures(&source) {
        Some(t) => {
            num = t.get(1).unwrap().as_str().parse::<u8>().unwrap();
            remove = t.get(0).unwrap().as_str().to_string();
        }
        None => {
            num = 0;
            remove = String::new();
        }
    }
    
    (source.replace(&remove, ""), num)
}

pub fn trim(mut source: String) -> String {
    lazy_static! {
        static ref COMMENT_FINDER: Regex = {
            Regex::new("//.*").unwrap()
        };
        static ref WHITESPACE_TRIMMER: Regex = {
            Regex::new("(^[\\s]+|[\\s]+$)").unwrap()
        };
    }

    let comments: Vec<String> = COMMENT_FINDER.find_iter(&source)
        .map(|e| e.as_str().to_string())
        .collect();
    for comment in comments {
        source = source.replace(&comment, "");
    }
    
    let mut whitespaces: Vec<(usize, usize)> = WHITESPACE_TRIMMER.find_iter(&source)
        .map(|e| (e.start(), e.end()))
        .collect();
    whitespaces.sort();
    whitespaces.reverse();

    for whitespace in whitespaces {
        source.replace_range(whitespace.0..whitespace.1, "");
    }

    source
}