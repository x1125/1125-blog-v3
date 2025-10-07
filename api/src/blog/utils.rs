use crate::blog::config::DEFAULT_BRANCH;
use git2::{Delta, DiffDelta, DiffOptions, Index, Patch, Repository};
use regex::Regex;
use serde::Serialize;
use std::path::Path;
use std::{borrow::BorrowMut, path::PathBuf};
use walkdir::WalkDir;

#[derive(Debug, Serialize)]
pub struct File {
    pub name: String,
    pub is_dir: bool,
}

#[derive(Debug, Serialize)]
pub struct Entry {
    pub name: String,
    pub assets: Vec<String>,
}

#[derive(Debug, Serialize)]
pub struct UnknownEntry {
    pub name: String,
    pub is_dir: bool,
}

#[derive(Debug, Serialize)]
pub struct Change {
    pub name: String,
    pub change: String,
    pub old_name: Option<String>,
    pub tracked: bool,
    pub staged: bool,
}

#[derive(Debug, Serialize)]
pub struct Content {
    pub entries: Vec<Entry>,
    pub unknown_entries: Vec<UnknownEntry>,
}

#[derive(Debug, Serialize)]
pub struct Diff {
    pub name: String,
    pub content: String,
}

pub fn find_files(path: &PathBuf, filter: Option<&str>) -> Vec<File> {
    let mut files: Vec<File> = Vec::new();
    for file in WalkDir::new(path).into_iter().filter_map(|e| e.ok()) {
        let name = file
            .path()
            .to_string_lossy()
            .replace(format!("{}/", path.to_string_lossy()).as_str(), "");
        // skip empty names and same as path names
        if name.len() < 1 || name == path.to_string_lossy() {
            continue;
        }
        // skip hidden directories (e.g. .git)
        if name.starts_with(".") {
            continue;
        }
        // apply filter
        if filter.is_some() && !name.ends_with(filter.unwrap()) {
            continue;
        }
        files.push(File {
            name,
            is_dir: file.path().is_dir(),
        });
    }
    return files;
}

pub fn get_entries(files: &mut Vec<File>) -> (Vec<Entry>, Vec<UnknownEntry>) {
    let mut entries: Vec<Entry> = Vec::new();
    let mut unknown_entries: Vec<UnknownEntry> = Vec::new();
    files.retain(|file| {
        let is_markdown = file.name.ends_with(".md");
        if is_markdown {
            entries.push(Entry {
                name: file.name.clone(),
                assets: vec![],
            });
        }
        return !is_markdown;
    });

    files.retain(|file| {
        let find_entry_index = |name: String, entries: &Vec<Entry>| -> Option<usize> {
            let name_split: Vec<&str> = name.splitn(2, '/').collect();
            for (idx, entry) in entries.iter().enumerate() {
                let base_name = entry.name.replace(".md", "");
                if name_split.first().unwrap() == &base_name.as_str() {
                    return Some(idx);
                }
            }
            return None;
        };
        let entry_index = find_entry_index(file.name.clone(), &entries);
        if entry_index.is_some() {
            let entry: &mut Entry = entries.get_mut(entry_index.unwrap()).unwrap();
            if !file.is_dir {
                entry.assets.push(file.name.clone());
            }
        } else {
            unknown_entries.push(UnknownEntry {
                name: file.name.clone(),
                is_dir: file.is_dir,
            });
        }
        return !entry_index.is_some();
    });

    return (entries, unknown_entries);
}

pub fn get_changes(repo: &Repository) -> Vec<Change> {
    let mut changes = vec![];
    let statuses = match repo.statuses(Some(
        git2::StatusOptions::new()
            .borrow_mut()
            .renames_head_to_index(true)
            .renames_index_to_workdir(true)
            .include_untracked(true),
    )) {
        Ok(statuses) => statuses,
        Err(e) => panic!("error: {}", e),
    };

    let index = repo.index().unwrap();

    for status in statuses.iter() {

        // add staged files
        let mut diff_delta = status.head_to_index();
        if diff_delta.is_some() {
            add_change(&mut changes, &index, status.path().unwrap(), &diff_delta.unwrap(), true);
        }

        // add unstaged files
        diff_delta = status.index_to_workdir();
        if diff_delta.is_some() {
            add_change(&mut changes, &index, status.path().unwrap(), &diff_delta.unwrap(), false);
        }
    }
    return changes;
}

fn add_change(changes: &mut Vec<Change>, index: &Index, path: &str, diff_delta: &DiffDelta, staged: bool) {
    let mut name: String = path.to_string();
    let mut old_name: Option<String> = None;
    if diff_delta.status() == Delta::Renamed {
        old_name = Some(name);
        name = String::from(diff_delta.new_file().path().unwrap().to_string_lossy());
    }
    let tracked = index.get_path(Path::new(&name), 0).is_some();
    changes.push(Change {
        name,
        change: format!("{:?}", diff_delta.status()),
        old_name,
        tracked,
        staged,
    });
}

pub fn get_diffs(repo: &Repository) -> Vec<Diff> {
    let mut diffs = vec![];
    let mut diff_options = DiffOptions::new();
    diff_options.include_untracked(true);
    diff_options.include_typechange(true);

    let reference = repo.find_reference(format!("refs/heads/{}", DEFAULT_BRANCH).as_str()).unwrap();

    add_diff(&mut diffs, &repo
        .diff_tree_to_workdir_with_index(Some(&reference.peel_to_commit().unwrap().tree().unwrap()), Some(&mut diff_options)).unwrap());

    return diffs;
}

fn add_diff(diffs: &mut Vec<Diff>, diff: &git2::Diff) {
    let stats = diff.stats().unwrap();
    if stats.files_changed() == 0 {
        return;
    }

    for idx in 0..stats.files_changed() - 1 {
        let patch = Patch::from_diff(&diff, idx);
        let patch_content = String::from(patch.unwrap().unwrap().to_buf().unwrap().as_str().unwrap());
        let diff_name_pattern = Regex::new(r#"diff --git a/(.+?) b/(.+?)\n"#).unwrap_or_else(|e| {
            panic!("error: {}", e);
        });
        for cap in diff_name_pattern.captures_iter(patch_content.as_str()) {
            if &cap[1] != &cap[2] {
                panic!("patch name mismatch: {} != {}", &cap[1], &cap[2]);
            }
            diffs.push(Diff {
                name: String::from(&cap[1]),
                content: patch_content.clone(),
            });
        }
    }
}