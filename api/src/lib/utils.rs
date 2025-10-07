use std::borrow::BorrowMut;
use git2::Repository;
use walkdir::WalkDir;
use serde::Serialize;

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
pub struct Change {
    pub name: String,
    pub change: String,
}

#[derive(Debug, Serialize)]
pub struct Content {
    pub entries: Vec<Entry>,
    pub changes: Vec<Change>,
}

pub fn find_files(path: String, filter: Option<&str>) -> Vec<File> {
    let mut files: Vec<File> = Vec::new();
    for file in WalkDir::new(path.as_str())
        .into_iter()
        .filter_map(|e| e.ok()) {
        let name = file.path().to_string_lossy().replace(format!("{}/", &path).as_str(), "");
        // skip empty names and same as path names
        if name.len() < 1 || name == path {
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

pub fn get_entries(files: &mut Vec<File>) -> Vec<Entry> {
    let mut entries: Vec<Entry> = Vec::new();
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
            for (idx, entry) in entries.iter().enumerate() {
                let base_name = entry.name.replace(".md", "");
                if name.starts_with(base_name.as_str()) {
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
            println!("unknown entry: {}", file.name.clone())
        }
        return !entry_index.is_some();
    });

    return entries;
}

pub fn get_changes(path: String) -> Vec<Change> {
    let mut changes = vec![];
    let repo = match Repository::open(path) {
        Ok(repo) => repo,
        Err(e) => panic!("failed to open: {}", e),
    };
    let statuses = match repo.statuses(Some(
        git2::StatusOptions::new()
            .borrow_mut()
            .include_untracked(true)
    )) {
        Ok(statuses) => statuses,
        Err(e) => panic!("error: {}", e),
    };
    for status in statuses.iter() {
        changes.push(Change {
            name: status.path().unwrap().to_string(),
            change: status_to_str(status.status()),
        });
    }
    return changes;
}

fn status_to_str(status: git2::Status) -> String {
    match status {
        git2::Status::WT_NEW => String::from("new"),
        _ => String::from("undefined")
    }
}
