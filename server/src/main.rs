extern crate pest;
#[macro_use]
extern crate pest_derive;

extern crate env_logger;
extern crate minimal as roxi;

use roxi::ruleindex::RuleIndex;
use roxi::tripleindex::TripleIndex;
use roxi::TripleStore;
use std::cell::RefCell;
use std::fs;
use std::rc::Rc;

use clap::Parser;
use env_logger::Env;
use log::{info, warn};
use roxi::encoding::Encoder;
use roxi::parser::Parser as TripleParser;
use roxi::parser::Syntax;
use roxi::sparql::{eval_query, evaluate_plan_and_debug};
use spargebra::Query;
use std::fs::{read_to_string, File};
use std::io::{BufReader, Read};

#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Args {
    /// File path to the ABox (in TTL format)
    #[clap(short, long)]
    abox: String,

    /// File path to the TBox (in TTL format)
    #[clap(short, long)]
    tbox: String,

    // /// SPARQL query to be executed
    // #[clap(short, long)]
    // query: String,
    /// Trace of reasoning process
    #[clap(short, long)]
    trace: Option<bool>,
}

fn main() {
    let args = Args::parse();

    let timer = ::std::time::Instant::now();
    if let Some(true) = args.trace {
        env_logger::Builder::from_env(Env::default().default_filter_or("trace")).init();
    } else {
        env_logger::init();
    }

    info!("Loading data ABox in NTriples");
    let mut store = TripleStore::new();
    let file_content = read_to_string(args.abox).unwrap();
    store.load_triples(&file_content, Syntax::NTriples);

    info!("Loading rules in N3");

    let rules = read_to_string(args.tbox).unwrap();
    store.load_rules(&rules);
    let elapsed = timer.elapsed();

    info!("Data loaded in: {:.2?}", elapsed);
    info!("ABox size: {}", store.len());
    info!("Starting materialization");

    let timer2 = ::std::time::Instant::now();
    store.materialize();
    let elapsed2 = timer2.elapsed();

    info!("Materialization time: {:.2?}", elapsed2);
    info!("Materialized store size: {}", store.len());
    info!("Content:\n{:?}", store.content_to_string());
}
