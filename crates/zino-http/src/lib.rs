#![cfg_attr(docsrs, feature(doc_auto_cfg))]
#![doc = include_str!("../README.md")]
#![doc(html_favicon_url = "https://zino.cc/assets/zino-logo.png")]
#![doc(html_logo_url = "https://zino.cc/assets/zino-logo.svg")]
#![allow(async_fn_in_trait)]

mod helper;

pub mod request;
pub mod response;
pub mod timing;

#[cfg(feature = "inertia")]
pub mod inertia;

#[cfg(feature = "view")]
pub mod view;
