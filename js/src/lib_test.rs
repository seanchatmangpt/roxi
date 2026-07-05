#![cfg(test)]

use crate::RoxiReasoner;

    #[test]
    fn test_js_load_reasoner(){
        let mut reasoner = RoxiReasoner::new();
        reasoner.add_abox("test".to_string());
    }
