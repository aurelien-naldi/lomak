//! The `LOgical Model Analysis Kit (lomak)` crate provides data structures and algorithms
//! to define, transform and analyse discrete (Boolean or multi-valued) dynamical models
//! based on logical functions.

#[macro_use]
extern crate pest_derive;
extern crate thiserror;

pub mod command;
pub mod func;
pub mod helper;
pub mod model;
pub mod variables;
