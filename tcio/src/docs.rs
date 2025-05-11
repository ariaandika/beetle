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
//! - [`serve_tcp`]
//!
//! # Application Flow
//!
//! Beetle framework provide multiple level of API to provide both convinient and low level control.
//!
//! ## Starting Server
//!
//! Application starts by building a [`Service`], which is required to start beetle server. The
//! service is required to be a service with `Request` of [`Request<ReqBody>`],
//! `Response` of [`Response<ResBody>`], and any error that implement [`std::error::Error`].
//! This service is aliased as [`HttpService`] for convinient. The [`Service`] then can be
//! passed to [`serve`] which actually start the server.
//!
//! For lower level control, using [`serve_tcp`] requires a [`Service`] with `Request` of [`TcpStream`].
//! The service is supposed to be repeatedly accept http request from given [`TcpStream`].
//!
//! ## Building [`Service`] via [`Router`]
//!
//! To build a [`Service`], beetle framework provide the [`Router`] API. The [`Router`] API is a builder
//! to the [`Service`] trait that provide features expected from modern concept of http server,
//! such as [routing][self#Routing] and [middleware][self#Middleware].
//!
//! ## Routing
//!
//! Routing is a concept of branching a request into different [handlers][self#Handler].
//! Using [`Router::route`], user can passed a handler that will be called when http path
//! and/or method is matched.
//!
//! ## Middleware
//!
//! Middleware is a logic that runs against a request before reaching a handler. Middleware can
//! modify, validate, or reject a request.
//!
//! ## Handler
//!
//! A handler is a user defined operation that contains a bussiness logic. Specifically, a handler
//! is just another [`HttpService`]. beetle provide APIs that will make bulding handler easier.
//!
//! Using [`ServiceFn`], user can create a handler just from an async function. The function,
//! can accept multiple arguments that implement [`FromRequestParts`] and single argument that
//! implement [`FromRequest`], and returns an [`IntoResponse`]. This will give user the exact type
//! it needed to perform the bussiness logic, abstracting away all the parsing, validation, and
//! error handling.
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
//! [`Service`]: crate::service::Service

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
