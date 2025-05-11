//! Reference
//!
//! # Shared
//!
//! - [`ByteStr`]
//!
//! # HTTP
//!
//! ## Minor Types
//!
//! - [`Method`]
//! - [`Version`]
//! - [`Uri`]
//! - TODO typed headers
//! - [`StatusCode`]
//!
//! # Major Types
//!
//! - [`ReqParts`]
//! - [`ReqBody`]
//! - [`Request`]
//! - [`ResParts`]
//! - [`ResBody`]
//! - [`Response`]
//!
//! # Extractor
//!
//! - [`FromRequest`]
//! - [`FromRequestParts`]
//!
//! ## Implementor
//!
//! - [`String`]
//! - [`Form`]
//! - [`Json`]
//! - [`Request`]
//!
//! # Responder
//!
//! - [`IntoResponse`]
//! - [`IntoResponseParts`]
//!
//! ## Implementor
//!
//! - [`String`]
//! - [`Html`]
//! - [`Json`]
//! - [`Response`]
//!
//! # Routing
//!
//! - [`Router`]
//!
//! ## Helpers
//!
//! - [`Branch`]
//! - [`Matcher`]
//! - [`State`]
//!
//! # Service
//!
//! - [`Service`]
//! - [`Layer`]
//!
//! ## Helpers
//!
//! - [`HttpService`]
//! - [`BodyLimit`]
//! - [`ServiceFn`]
//!
//! # Runtime
//!
//! - [`serve`]
//!
//!
//!
//! [`ByteStr`]: crate::bytestr::ByteStr
//! [`Bytes`]: bytes::Bytes
//! [`BytesMut`]: bytes::BytesMut
//! [`Method`]: crate::http::Method
//! [`Version`]: crate::http::Version
//! [`Uri`]: crate::http::Uri
//! [`StatusCode`]: crate::http::StatusCode
//! [`Parts`]: crate::request::Parts
//! [`Request`]: crate::request::Request
//! [`Response`]: crate::response::Response
//! [`FromRequest`]: crate::request::FromRequest
//! [`FromRequestParts`]: crate::request::FromRequestParts
//! [`IntoResponse`]: crate::response::IntoResponse
//! [`IntoResponseParts`]: crate::response::IntoResponseParts
//! [`Router`]: crate::route::Router

// impl Future vs type Future vs generic Future
// - impl Future: can be async fn, type cannot be referenced externally, no double implementation
// - type Future: no async fn, type can be referenced externally, no double implementation
// - generic Future: no async fn, type ? be referenced externally, can double implementation
//
// impl Future
// - can be async fn
// - can contains unnamed future without boxing, like async fn or private future type
// - future type cannot be referenced externally
// - cannot have double implementation
//
// generic Future
// - cannot be async fn
// - cannot contains unnamed future without boxing, like async fn or private future type
// - future type cannot be referenced externally
// - can have double implementation
//
// type Future
// - cannot be async fn
// - cannot contains unnamed future without boxing, like async fn or private future type (unstable)
// - future type can be referenced externally
// - cannot have double implementation
