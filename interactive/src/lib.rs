//! Interactive differential dataflow
//!
//! This crate provides a demonstration of an interactive differential
//! dataflow system, which accepts query plans as data and then directly
//! implements them without compilation.

#![forbid(missing_docs)]

extern crate bincode;
extern crate timely;
extern crate differential_dataflow;
extern crate dogsdogsdogs;
extern crate serde;
#[macro_use]
extern crate serde_derive;

pub mod plan;
pub use plan::Plan;

pub mod manager;
pub use manager::{Manager, TraceManager, InputManager};

pub mod command;
pub use command::Command;

pub mod logging;

pub mod concrete;

/// System-wide notion of time.
pub type Time = ::std::time::Duration;
/// System-wide update type.
pub type Diff = isize;

use std::hash::Hash;
use std::fmt::Debug;
use serde::{Serialize, Deserialize};

/// Types capable of use as data in interactive.
pub trait Datum : Hash+Sized+Debug {
    /// A type that can act on slices of data.
    type Expression : Clone+Debug+Eq+Ord+Hash+Serialize+for<'a>Deserialize<'a>;
    /// Applies an expression to a slice of data.
    fn subject_to(data: &[Self], expr: &Self::Expression) -> Self;
    /// Creates a expression that implements projection.
    fn projection(index: usize) -> Self::Expression;
}

impl Datum for usize {
    type Expression = usize;
    fn subject_to(data: &[Self], expr: &usize) -> Self { data[*expr] }
    fn projection(index: usize) -> usize { index }
}

impl Datum for u32 {
    type Expression = usize;
    fn subject_to(data: &[Self], expr: &usize) -> Self { data[*expr] }
    fn projection(index: usize) -> usize { index }
}

/// A type that can be converted to a vector of another type.
pub trait VectorFrom<T> : Sized {
    /// Converts `T` to a vector of `Self`.
    fn vector_from(item: T) -> Vec<Self>;
}

/// Multiple related collection definitions.
#[derive(Serialize, Deserialize, Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct Query<V: Datum> {
    /// A list of bindings of names to plans.
    pub rules: Vec<Rule<V>>,
    /// A list of arrangements to import.
    pub imports: Vec<(Plan<V>, Vec<usize>)>,
    /// A list of arrangements to publish.
    pub publish: Vec<(Plan<V>, Vec<usize>)>,
}

impl<V: Datum> Query<V> {
    /// Creates a new, empty query.
    pub fn new() -> Self {
        Query { rules: Vec::new(), imports: Vec::new(), publish: Vec::new(), }
    }
    /// Adds a rule to an existing query.
    pub fn add_rule(mut self, rule: Rule<V>) -> Self {
        self.rules.push(rule);
        self
    }
    /// Adds an external trace to import.
    pub fn add_import(mut self, plan: Plan<V>, keys: Vec<usize>) -> Self {
        self.imports.push((plan, keys));
        self
    }
    /// Adds an internal trace to export.
    pub fn add_publish(mut self, plan: Plan<V>, keys: Vec<usize>) -> Self {
        self.publish.push((plan, keys));
        self
    }
}

impl<V: Datum> Query<V> {
    /// Converts the query into a command.
    pub fn into_command(self) -> Command<V> {
        Command::Query(self)
    }
}

/// Definition of a single collection.
#[derive(Serialize, Deserialize, Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct Rule<V: Datum> {
    /// Name of the rule.
    pub name: String,
    /// Plan describing contents of the rule.
    pub plan: Plan<V>,
}

impl<V: Datum> Rule<V> {
    /// Converts the rule into a singleton query.
    pub fn into_query(self) -> Query<V> {
        Query::new().add_rule(self)
    }
}
