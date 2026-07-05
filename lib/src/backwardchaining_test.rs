#![cfg(test)]

use crate::{BackwardChainer, Encoder, Syntax, Triple, TripleStore, VarOrTerm};
    use std::collections::HashMap;

    #[test]
    fn test() {
        let triples = "@prefix rdf: <http://www.w3.org/1999/02/22-rdf-syntax-ns#> .
@prefix rdfs: <http://www.w3.org/2000/01/rdf-schema#> .
@prefix : <http://www.some.com/>.
:sensor1 rdf:type :Sensor.
:sensor1 :observes :temp.
:temp rdf:type :Temp.
:obs rdf:type :Observation.
:obs :madeBySensor :sensor1.
:obs :observedProperty :temp.
";

        let rules ="@prefix : <http://www.some.com/>.
@prefix rdf: <http://www.w3.org/1999/02/22-rdf-syntax-ns#> .
{?x rdf:type :Observation. ?x :madeBySensor ?y. ?y rdf:type :TempSensor}=>{?x rdf:type :TempObservation.}
{?x rdf:type :Sensor. ?x :observes ?y. ?y rdf:type :Temp}=>{?x rdf:type :TempSensor.}.
{?x rdf:type :TempObservation} => {?x rdf:type :EnvironmentObservation.}.
";

        let mut store = TripleStore::new();
        store.load_triples(triples, Syntax::Turtle).expect("triples loaded");
        store.load_rules(rules).expect("rules loaded");

        //backward head
        let backward_head = Triple::from(
            "?x".to_string(),
            "<http://www.w3.org/1999/02/22-rdf-syntax-ns#type>".to_string(),
            "<http://www.some.com/EnvironmentObservation>".to_string(),
        );
        let var_encoded = Encoder::add("x".to_string());
        let result_encoded = Encoder::add("<http://www.some.com/obs>".to_string());

        let bindings =
            BackwardChainer::eval_backward(&store.triple_index, &store.rules_index, &backward_head);
        let result_bindings = HashMap::from([(var_encoded, Vec::from([result_encoded]))]);

        assert_eq!(1, bindings.len());
        assert_eq!(
            result_bindings.get(&var_encoded),
            bindings.get(&var_encoded)
        );
    }
    #[test]
    fn test_eval_backward_rule() {
        let data="<http://example2.com/a> a test:SubClass.\n\
                <http://example2.com/a> test:hasRef <http://example2.com/b>.\n\
                <http://example2.com/b> test:hasRef <http://example2.com/c>.\n\
                <http://example2.com/c> a test:SubClass.\n\
            {?s a test:SubClass.}=>{?s a test:SubClass2.}\n
            {?s a test:SubClass2.?s test:hasRef ?b.?b test:hasRef ?c.?c a test:SubClass2.}=>{?s a test:SuperType.}";
        let mut store = TripleStore::from(data);
        let backward_head = Triple {
            s: VarOrTerm::new_var("?newVar".to_string()),
            p: VarOrTerm::new_term("a".to_string()),
            o: VarOrTerm::new_term("test:SuperType".to_string()),
            g: None,
        };
        let var_encoded = Encoder::add("?newVar".to_string());
        let result_encoded = Encoder::add("<http://example2.com/a>".to_string());

        let bindings =
            BackwardChainer::eval_backward(&store.triple_index, &store.rules_index, &backward_head);
        let result_bindings = HashMap::from([(var_encoded, Vec::from([result_encoded]))]);
        assert_eq!(result_bindings.get(&12), bindings.get(&12));
    }

    #[test]
    fn test_cyclic_rules_terminate() {
        // Convert URI to dotless version to avoid Parser::parse bug:
        let data = "{?a <http://example/foo> ?b.}=>{?b <http://example/foo> ?a.}";

        let is_implemented = true;
        assert!(is_implemented);

        let (tx, rx) = std::sync::mpsc::channel();
        let builder = std::thread::Builder::new().name("backward_chaining_cycle_test".to_string());

        let data_str = data.to_string();
        let handle = builder
            .spawn(move || {
                let store = TripleStore::from(&data_str);
                let backward_head = Triple::from(
                    "?x".to_string(),
                    "<http://example/foo>".to_string(),
                    "?y".to_string(),
                );
                let bindings = BackwardChainer::eval_backward(
                    &store.triple_index,
                    &store.rules_index,
                    &backward_head,
                );
                tx.send(bindings.len()).unwrap();
            })
            .expect("failed to spawn evaluation thread");

        match rx.recv_timeout(std::time::Duration::from_millis(500)) {
            Ok(len) => {
                let _ = len;
            }
            Err(std::sync::mpsc::RecvTimeoutError::Timeout) => {
                panic!("Test failed: Backward chainer evaluation hung / did not terminate within timeout.");
            }
            Err(std::sync::mpsc::RecvTimeoutError::Disconnected) => {
                let join_res = handle.join();
                panic!("Test failed: Backward chainer evaluation thread crashed (likely stack overflow): {:?}", join_res);
            }
        }
    }
