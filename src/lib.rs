#![feature(trait_alias)]
pub mod asyncdao;
pub mod asyncdatabase;
mod autoserde;
#[cfg(feature = "redis_async")]
pub mod cache;
mod common;
pub mod controller;
pub mod dao;
pub mod database;
pub mod entity;
pub mod repository;
pub mod service;
