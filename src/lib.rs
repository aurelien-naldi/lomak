//! The `LOgical Model Analysis Kit (lomak)` crate provides data structures and algorithms
//! to define, transform and analyse discrete (Boolean or multi-valued) dynamical models
//! based on logical functions.

#[macro_use]
extern crate pest_derive;
#[macro_use]
extern crate lazy_static;

pub mod command;
pub mod func;
pub mod model;
pub mod services;
pub mod solver;
