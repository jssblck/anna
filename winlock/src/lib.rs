// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

//! Winlock is an LLM agent coding assistant named after Anna Winlock.
//!
//! Anna Winlock was a pioneering woman in computing who worked at Harvard College
//! Observatory in the late 1800s. She was known for her mathematical and
//! computational work in astronomy.

use std::{fmt::Write, path::PathBuf};

use bon::Builder;
use derive_more::{Debug, Display, Error, From};
use url::Url;

/// Errors returned by the agent when prompting.
#[derive(Debug, Display, Error)]
#[non_exhaustive]
pub enum Error {}

/// The main agent type that coordinates interactions with a given codebase.
#[derive(Debug, Clone, Builder)]
#[non_exhaustive]
pub struct Agent {
    /// The codebase with which the agent interacts.
    #[builder(into)]
    pub codebase: Codebase,

    /// The runner that the agent uses to execute code.
    #[builder(into)]
    pub runner: Runner,

    /// The LLM backend that the agent uses.
    #[builder(into)]
    pub backend: Backend,

    /// The system prompt used by the agent for each query.
    #[builder(into)]
    pub system: Context,
}

impl Agent {
    /// Prompt the agent with the provided context and request.
    pub fn prompt(&self, _rag: Vec<Context>, _prompt: String) -> Result<String, Error> {
        todo!()
    }
}

/// Context provided to the LLM backend with a query.
#[derive(Clone, From)]
#[non_exhaustive]
pub struct Context(String);

impl Context {
    /// The inner context value.
    pub fn inner(&self) -> &str {
        &self.0
    }
}

impl std::fmt::Display for Context {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if f.alternate() || self.0.len() <= 20 {
            write!(f, "{}", self.0)
        } else {
            write_truncated(f, 20, &self.0)
        }
    }
}

impl std::fmt::Debug for Context {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Context({self})")
    }
}

/// Describes a codebase location.
#[derive(Debug, Clone, From)]
#[non_exhaustive]
pub enum Codebase {
    /// The root directory of the codebase on the local disk.
    Path(PathBuf),
}

/// Agents run arbitrary commands; these are of course potentially dangerous
/// to run on the main system (especially the main system of a dev machine).
///
/// Runners provide a safe arena for executing code.
/// These are implemented using Docker; each [`Agent`] gets a [`Runner`]
/// with the [`Codebase`] bound to a volume inside the runner's context.
///
/// This allows the agent to execute arbitrary commands that can modify
/// the [`Codebase`] without danger of those commands affecting the
/// rest of the system.
///
/// A runner instance is specifically a reference to an
/// _already running_ executor inside an _already running_
/// Docker container.
#[derive(Debug, Clone, Builder)]
#[non_exhaustive]
pub struct Runner {
    /// Runners are actually programs inside a Docker container;
    /// agents communicate with them using a locally bound address.
    #[builder(into)]
    pub address: Url,

    /// The identifier of the container in which the runner is running.
    /// This is used for debugging containers in the Docker runtime
    /// if the runner isn't responding to its address.
    #[builder(into)]
    pub container_id: String,
}

/// Agents need an LLM backend to do much of anything.
/// Describes an LLM backend for an agent.
#[derive(Debug, Clone)]
#[non_exhaustive]
pub enum Backend {
    /// References the OpenAI API.
    OpenAI(Url, ApiKey),
}

/// Special type for API keys guarding them from being accidentally logged.
#[derive(Debug, Clone, From)]
#[debug("ApiKey(..)")]
#[non_exhaustive]
pub struct ApiKey(String);

impl ApiKey {
    /// The inner key value.
    pub fn inner(&self) -> &str {
        &self.0
    }
}

/// Drops the middle of a string to fit within a given length.
fn write_truncated(f: &mut std::fmt::Formatter<'_>, len: usize, content: &str) -> std::fmt::Result {
    if content.len() <= len {
        return write!(f, "{content}");
    }

    if len <= 2 {
        for _ in 0..len {
            f.write_char('.')?;
        }
        return Ok(());
    }

    let half = len / 2;
    let mut chars = content.chars();
    for c in chars.by_ref().take(half) {
        f.write_char(c)?;
    }
    f.write_str("..")?;
    for c in chars.rev().take(half) {
        f.write_char(c)?;
    }

    Ok(())
}
