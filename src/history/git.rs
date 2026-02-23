//! Git-backed save history repository.
//!
//! `HistoryRepo` wraps `git2::Repository` to provide high-level operations for
//! save-state versioning: initializing a repo, committing snapshots, listing
//! branches/commits, and restoring to earlier states.

use std::path::Path;

use git2::{
    BranchType, Commit, IndexAddOption, ObjectType, Oid, Repository, Signature, StatusOptions,
};

use super::types::{CommitInfo, SaveEvent, TimelineInfo};

// ── HistoryError ────────────────────────────────────────────────────────

/// Errors produced by history git operations.
#[derive(Debug)]
pub enum HistoryError {
    /// An underlying libgit2 error.
    Git(git2::Error),
    /// `commit()` was called but no files had changed.
    NothingToCommit,
    /// A branch name was not found.
    BranchNotFound(String),
    /// A commit SHA was not found.
    CommitNotFound(String),
}

impl std::fmt::Display for HistoryError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            HistoryError::Git(e) => write!(f, "git error: {e}"),
            HistoryError::NothingToCommit => write!(f, "nothing to commit"),
            HistoryError::BranchNotFound(name) => write!(f, "branch not found: {name}"),
            HistoryError::CommitNotFound(id) => write!(f, "commit not found: {id}"),
        }
    }
}

impl std::error::Error for HistoryError {}

impl From<git2::Error> for HistoryError {
    fn from(e: git2::Error) -> Self {
        HistoryError::Git(e)
    }
}

// ── HistoryRepo ─────────────────────────────────────────────────────────

/// A git repository used for save-state history tracking.
pub struct HistoryRepo {
    repo: Repository,
}

impl HistoryRepo {
    /// Initialize or open a git repository at `quest_dir`.
    ///
    /// If the directory does not yet contain a `.git/` folder, a new repository
    /// is created, all files are staged, and an initial commit is recorded.
    /// The default branch is renamed to "main" if it was created as "master".
    pub fn init(quest_dir: &Path) -> Result<Self, HistoryError> {
        let is_new = !quest_dir.join(".git").exists();

        let repo = if is_new {
            Repository::init(quest_dir)?
        } else {
            Repository::open(quest_dir)?
        };

        if is_new {
            // Stage everything.
            let mut index = repo.index()?;
            index.add_all(["*"].iter(), IndexAddOption::DEFAULT, None)?;
            index.write()?;

            let tree_oid = index.write_tree()?;
            let tree = repo.find_tree(tree_oid)?;
            let sig = Self::signature()?;

            repo.commit(
                Some("HEAD"),
                &sig,
                &sig,
                "Initialize save history",
                &tree,
                &[],
            )?;

            // Rename master -> main if needed.
            if let Ok(mut master) = repo.find_branch("master", BranchType::Local) {
                master.rename("main", true)?;
            }
        }

        Ok(Self { repo })
    }

    /// Stage all changes and create a commit with a formatted message.
    ///
    /// Returns `Err(HistoryError::NothingToCommit)` if the working tree has no
    /// changes since the last commit.
    pub fn commit(
        &self,
        event: &SaveEvent,
        level: u32,
        prestige: u32,
        zone_id: u32,
        subzone_id: u32,
        play_time_seconds: u64,
    ) -> Result<(), HistoryError> {
        // Check if there are any changes.
        if !self.has_changes()? {
            return Err(HistoryError::NothingToCommit);
        }

        // Stage everything.
        let mut index = self.repo.index()?;
        index.add_all(["*"].iter(), IndexAddOption::DEFAULT, None)?;
        index.write()?;

        let tree_oid = index.write_tree()?;
        let tree = self.repo.find_tree(tree_oid)?;
        let sig = Self::signature()?;
        let message = event.commit_message(level, prestige, zone_id, subzone_id, play_time_seconds);

        let parent = self.head_commit()?;
        self.repo
            .commit(Some("HEAD"), &sig, &sig, &message, &tree, &[&parent])?;

        Ok(())
    }

    /// List all branches with head commit info.
    ///
    /// The currently active branch is listed first, then the rest
    /// alphabetically.
    pub fn list_branches(&self) -> Result<Vec<TimelineInfo>, HistoryError> {
        let head_ref = self.repo.head().ok();
        let active_branch = head_ref
            .as_ref()
            .and_then(|r| r.shorthand().map(|s| s.to_string()));

        let mut branches: Vec<TimelineInfo> = Vec::new();

        for branch_result in self.repo.branches(Some(BranchType::Local))? {
            let (branch, _) = branch_result?;
            let name = branch.name()?.unwrap_or("(unnamed)").to_string();
            let is_active = active_branch.as_deref() == Some(&name);

            let head_commit = branch
                .get()
                .peel_to_commit()
                .ok()
                .map(|c| self.commit_to_info(&c));

            branches.push(TimelineInfo {
                name,
                is_active,
                head_commit,
            });
        }

        // Sort: active first, then alphabetically.
        branches.sort_by(|a, b| {
            b.is_active
                .cmp(&a.is_active)
                .then_with(|| a.name.cmp(&b.name))
        });

        Ok(branches)
    }

    /// List commits on a branch, newest first.
    pub fn list_commits(&self, branch_name: &str) -> Result<Vec<CommitInfo>, HistoryError> {
        let branch = self
            .repo
            .find_branch(branch_name, BranchType::Local)
            .map_err(|_| HistoryError::BranchNotFound(branch_name.to_string()))?;

        let branch_oid = branch
            .get()
            .target()
            .ok_or_else(|| HistoryError::BranchNotFound(branch_name.to_string()))?;

        let mut revwalk = self.repo.revwalk()?;
        revwalk.push(branch_oid)?;
        revwalk.set_sorting(git2::Sort::TIME | git2::Sort::TOPOLOGICAL)?;

        let mut commits = Vec::new();
        for oid_result in revwalk {
            let oid = oid_result?;
            let commit = self.repo.find_commit(oid)?;
            commits.push(self.commit_to_info(&commit));
        }

        Ok(commits)
    }

    /// Reset the current branch to the given commit and force-checkout.
    ///
    /// This moves the current branch pointer back to the target commit,
    /// discarding any commits after it. The old commits remain in git's
    /// reflog for recovery if needed.
    pub fn restore_to(&self, commit_id: &str) -> Result<(), HistoryError> {
        let oid = self.resolve_commit_id(commit_id)?;
        let commit = self
            .repo
            .find_commit(oid)
            .map_err(|_| HistoryError::CommitNotFound(commit_id.to_string()))?;

        // Reset the current branch ref to point at the target commit.
        self.repo.reset(
            commit.as_object(),
            git2::ResetType::Hard,
            None,
        )?;

        Ok(())
    }

    // ── Private helpers ─────────────────────────────────────────────────

    /// The committer/author signature used for all history commits.
    fn signature() -> Result<Signature<'static>, git2::Error> {
        Signature::now("Quest", "quest@localhost")
    }

    /// Check whether the working tree has any staged or unstaged changes.
    fn has_changes(&self) -> Result<bool, HistoryError> {
        let mut opts = StatusOptions::new();
        opts.include_untracked(true).recurse_untracked_dirs(true);
        let statuses = self.repo.statuses(Some(&mut opts))?;
        Ok(!statuses.is_empty())
    }

    /// Get the HEAD commit.
    fn head_commit(&self) -> Result<Commit<'_>, HistoryError> {
        let head = self.repo.head()?;
        let commit = head.peel_to_commit()?;
        Ok(commit)
    }

    /// Convert a `git2::Commit` into a `CommitInfo`.
    fn commit_to_info(&self, commit: &Commit<'_>) -> CommitInfo {
        let id = format!("{:.7}", commit.id());
        let message = commit.message().unwrap_or("").to_string();
        let timestamp = commit.time().seconds();

        let (level, prestige, zone, playtime) = parse_commit_suffix(&message);

        CommitInfo {
            id,
            message,
            timestamp,
            level,
            prestige,
            zone,
            playtime,
        }
    }

    /// Resolve a short or full commit id string to an Oid.
    fn resolve_commit_id(&self, commit_id: &str) -> Result<Oid, HistoryError> {
        // Try exact OID first.
        if let Ok(oid) = Oid::from_str(commit_id) {
            if self.repo.find_commit(oid).is_ok() {
                return Ok(oid);
            }
        }

        // Try as a prefix (short SHA).
        let obj = self
            .repo
            .revparse_single(commit_id)
            .map_err(|_| HistoryError::CommitNotFound(commit_id.to_string()))?;

        obj.peel(ObjectType::Commit)
            .map(|o| o.id())
            .map_err(|_| HistoryError::CommitNotFound(commit_id.to_string()))
    }

}

// ── Commit suffix parsing ───────────────────────────────────────────────

/// Parse the suffix portion of a commit message to extract level, prestige,
/// zone, and playtime.
///
/// Expected format after " | ": `Lv{N} P{N} Z{N}-{N} {N}h{NN}m`
///
/// Returns `(level, prestige, zone, playtime_seconds)` with zeros for
/// any values that cannot be parsed.
fn parse_commit_suffix(message: &str) -> (u32, u32, u32, u64) {
    let suffix = match message.split(" | ").nth(1) {
        Some(s) => s,
        None => return (0, 0, 0, 0),
    };

    let mut level = 0u32;
    let mut prestige = 0u32;
    let mut zone = 0u32;
    let mut playtime = 0u64;

    for token in suffix.split_whitespace() {
        if let Some(rest) = token.strip_prefix("Lv") {
            if let Ok(n) = rest.parse::<u32>() {
                level = n;
            }
        } else if let Some(rest) = token.strip_prefix('P') {
            // Must start with digit to avoid matching other P-words.
            if rest.chars().next().is_some_and(|c| c.is_ascii_digit()) {
                if let Ok(n) = rest.parse::<u32>() {
                    prestige = n;
                }
            }
        } else if let Some(rest) = token.strip_prefix('Z') {
            // Parse "Z2-3" -> zone 2 (we store zone_id, not subzone).
            if let Some(dash_pos) = rest.find('-') {
                if let Ok(n) = rest[..dash_pos].parse::<u32>() {
                    zone = n;
                }
            }
        } else if token.contains('h') && token.ends_with('m') {
            // Parse "2h15m" -> playtime in seconds.
            playtime = parse_playtime(token);
        }
    }

    (level, prestige, zone, playtime)
}

/// Parse a playtime string like "2h15m" into seconds.
fn parse_playtime(s: &str) -> u64 {
    let Some(h_pos) = s.find('h') else {
        return 0;
    };
    let hours: u64 = s[..h_pos].parse().unwrap_or(0);
    let minutes_str = &s[h_pos + 1..];
    let minutes: u64 = minutes_str
        .strip_suffix('m')
        .unwrap_or("0")
        .parse()
        .unwrap_or(0);
    hours * 3600 + minutes * 60
}

// ── Unit tests ──────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_suffix_basic() {
        let (level, prestige, zone, playtime) =
            parse_commit_suffix("Level up to 18 | Lv18 P0 Z2-3 2h15m");
        assert_eq!(level, 18);
        assert_eq!(prestige, 0);
        assert_eq!(zone, 2);
        assert_eq!(playtime, 8100);
    }

    #[test]
    fn parse_suffix_missing() {
        let (level, prestige, zone, playtime) = parse_commit_suffix("Initialize save history");
        assert_eq!(level, 0);
        assert_eq!(prestige, 0);
        assert_eq!(zone, 0);
        assert_eq!(playtime, 0);
    }

    #[test]
    fn parse_suffix_large_values() {
        let (level, prestige, zone, playtime) =
            parse_commit_suffix("Manual save | Lv50 P20 Z8-4 45h10m");
        assert_eq!(level, 50);
        assert_eq!(prestige, 20);
        assert_eq!(zone, 8);
        assert_eq!(playtime, 162600);
    }

    #[test]
    fn parse_playtime_zero() {
        assert_eq!(parse_playtime("0h00m"), 0);
    }

    #[test]
    fn parse_playtime_minutes_only() {
        assert_eq!(parse_playtime("0h30m"), 1800);
    }
}
