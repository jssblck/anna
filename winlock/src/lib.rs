// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

//! Winlock is an LLM agent coding assistant named after Anna Winlock.
//!
//! Anna Winlock was a pioneering woman in computing who worked at Harvard College
//! Observatory in the late 1800s. She was known for her mathematical and
//! computational work in astronomy.

use std::{
    collections::HashSet,
    path::{Path, PathBuf},
    str::FromStr,
};

use async_fs::DirEntry;
use async_walkdir::WalkDir;
use bon::{bon, Builder};
use color_eyre::{
    eyre::{bail, Context, OptionExt},
    Result,
};
use fslock::LockFile;
use futures_lite::StreamExt;
use pollster::FutureExt;
use rustygit::{types::BranchName, Repository};
use serde::{Deserialize, Serialize};
use tempfile::TempDir;

/// The main agent type that coordinates interactions with the LLM.
///
/// The intention of the `Agent` is to encapsulate a temporary and unique branch of a project,
/// allowing the user to supervise multiple concurrent work streams of the same project at a time.
#[derive(Debug)]
pub struct Agent {
    /// The directory at which the original project is located.
    pub project: PathBuf,

    /// The temporary location for the workspace.
    pub workspace: PathBuf,

    /// The branch name for this feature.
    pub branch: String,

    /// Whether the agent was resumed or created.
    pub status: AgentSessionStatus,
}

#[bon]
impl Agent {
    /// Create a new instance with the provided options.
    #[builder]
    pub fn new(
        /// The directory at which the original project is located.
        #[builder(into)]
        project: PathBuf,

        /// The branch to create for this feature.
        #[builder(into)]
        branch: String,
    ) -> Result<Agent> {
        if let Ok(Some(session)) = Sessions::get(&project, &branch) {
            return Ok(Self {
                branch: session.branch,
                project: session.project,
                workspace: session.workspace,
                status: AgentSessionStatus::Resumed,
            });
        }

        let branch_name = BranchName::from_str(&branch).context("parse branch name")?;
        let workspace = TempDir::new().context("create temp dir")?.into_path();
        copy_workspace(&project, &workspace).block_on();

        let repo = Repository::new(&workspace);
        repo.create_local_branch(&branch_name)
            .context("create branch")?;
        repo.switch_branch(&branch_name)
            .context("check out new branch")?;

        Sessions::store(&project, &workspace, &branch).context("store session")?;
        Ok(Self {
            branch,
            project,
            workspace,
            status: AgentSessionStatus::Created,
        })
    }

    /// Run the agent, hooking it up to the current std pipes.
    pub fn run(&self) -> Result<()> {
        std::process::Command::new("claude")
            .current_dir(&self.workspace)
            .spawn()
            .context("run agent")?
            .wait()
            .context("wait for agent to exit")?;
        Ok(())
    }
}

/// The session status of an agent instance.
#[derive(Copy, Clone, Debug)]
pub enum AgentSessionStatus {
    /// The agent was created.
    Created,

    /// The agent was resumed.
    Resumed,
}

impl std::fmt::Display for AgentSessionStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match *self {
            AgentSessionStatus::Created => write!(f, "created"),
            AgentSessionStatus::Resumed => write!(f, "resumed"),
        }
    }
}

/// An existing agent session entry.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Deserialize, Serialize, Builder)]
pub struct Session {
    /// The directory at which the original project is located.
    #[builder(into)]
    pub project: PathBuf,

    /// The temporary location for the workspace.
    #[builder(into)]
    pub workspace: PathBuf,

    /// The branch name for this feature.
    #[builder(into)]
    pub branch: String,
}

/// Manages all agent sessions.
#[derive(Debug)]
pub struct Sessions;

impl Sessions {
    /// Get the config directory. If it doesn't exist, it is created by this function.
    pub fn config_dir() -> Result<PathBuf> {
        let dir = homedir::my_home()
            .context("get home dir")?
            .ok_or_eyre("no home directory for user")
            .map(|d| d.join(".annawinlock"))?;
        if !dir.exists() {
            std::fs::create_dir_all(&dir).context("create config dir")?;
        }
        Ok(dir)
    }

    /// Get the path to the sessions file.
    pub fn config_sessions() -> Result<PathBuf> {
        Self::config_dir().map(|d| d.join("sessions.json"))
    }

    /// Open the lockfile to protect modifications across processes.
    fn lockfile() -> Result<LockFile> {
        let path = Self::config_dir()
            .map(|d| d.join("sessions.lock"))
            .context("get lockfile path")?;
        LockFile::open(&path).context("open lockfile")
    }

    /// List all available sessions.
    pub fn list_all() -> Result<HashSet<Session>> {
        let _lock = Self::lockfile().context("lock sessions")?;
        Self::list_all_inner()
    }

    fn list_all_inner() -> Result<HashSet<Session>> {
        let Ok(sessions) = Self::config_sessions() else {
            return Ok(HashSet::new());
        };

        let Ok(sessions) = std::fs::read_to_string(sessions).context("read sessions file") else {
            return Ok(HashSet::new());
        };

        serde_json::from_str(&sessions).context("parse sessions")
    }

    /// List all available sessions for the current project.
    pub fn list(project: &Path) -> Result<HashSet<Session>> {
        Self::list_all().map(|s| s.into_iter().filter(|s| s.project == project).collect())
    }

    /// Get the current session for the current project and branch, if it exists.
    pub fn get(project: &Path, branch: &str) -> Result<Option<Session>> {
        Self::list_all()?
            .into_iter()
            .find(|s| s.project == project && s.branch == branch)
            .map(Ok)
            .transpose()
    }

    /// Store the new session. If the session already exists at another path, an error is returned.
    pub fn store(project: &Path, workspace: &Path, branch: &str) -> Result<()> {
        let _lock = Self::lockfile().context("lock sessions")?;

        let mut sessions = Self::list_all_inner().context("list all sessions")?;
        let session = Session::builder()
            .branch(branch)
            .workspace(workspace)
            .project(project)
            .build();

        sessions.insert(session);
        Self::store_all_inner(sessions)
    }

    fn store_all_inner(sessions: HashSet<Session>) -> Result<()> {
        let sessions_file = Self::config_sessions().context("get sessions path")?;
        let sessions = serde_json::to_string_pretty(&sessions).context("encode sessions")?;
        std::fs::write(sessions_file, sessions).context("write sessions file")
    }

    /// Remove the session from disk and from the session storage.
    pub fn remove(project: &Path, branch: &str) -> Result<()> {
        let _lock = Self::lockfile().context("lock sessions")?;

        let mut sessions = Self::list_all_inner().context("read sessions")?;
        let Some(session) = sessions
            .iter()
            .find(|s| s.branch == branch && s.project == project)
            .cloned()
        else {
            eprintln!("[warn] session does not exist");
            return Ok(());
        };

        std::fs::remove_dir_all(&session.workspace).context("remove session workspace")?;
        sessions.remove(&session);

        Self::store_all_inner(sessions).context("store new sessions")
    }
}

/// Recursively copies the files in the project directory to the target workspace.
async fn copy_workspace(project: &Path, target: &Path) {
    let mut entries = WalkDir::new(project);
    while let Some(entry) = entries.next().await {
        match entry {
            Ok(entry) => {
                if let Err(err) = copy_workspace_entry(project, target, &entry).await {
                    eprintln!(
                        "[warn] error while copying '{}' in project: {:?}",
                        entry.path().display(),
                        err
                    )
                }
            }
            Err(err) => eprintln!("[warn] error while enumerating files in project: {err:?}"),
        }
    }
}

/// Copies a single entry from the project to the target workspace.
async fn copy_workspace_entry(project: &Path, target: &Path, entry: &DirEntry) -> Result<()> {
    let entry_src = entry.path();
    let entry_rel = entry_src
        .strip_prefix(project)
        .context("make path relative")?;
    let entry_dst = target.join(entry_rel);
    let kind = entry.file_type().await.context("read file type")?;

    if kind.is_dir() {
        async_fs::create_dir(entry_dst)
            .await
            .context("create directory")?;
    } else if kind.is_file() {
        async_fs::copy(&entry_src, &entry_dst)
            .await
            .context("copy file")?;
    } else {
        bail!("unknown file kind {kind:?} for '{}'", entry_rel.display())
    }

    Ok(())
}
