use crate::app::QueryAnalysis;
use std::collections::HashMap;

pub struct QueryAnalysisDefaultImpl;

impl QueryAnalysis for QueryAnalysisDefaultImpl {
    fn analysis(&self, query: &str) -> Option<HashMap<String, String>> {
        let args = query
            .split('&')
            .map(|x| x.to_string())
            .collect::<Vec<String>>();
        let mut map = HashMap::new();
        for v in args.into_iter() {
            let ss = v.split('=').collect::<Vec<&str>>();
            if ss.len() == 2 {
                map.insert(ss[0].into(), ss[1].into());
            }
        }
        if map.is_empty() {
            None
        } else {
            Some(map)
        }
    }
}
