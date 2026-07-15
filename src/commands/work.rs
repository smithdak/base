use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result, bail};
use chrono::Utc;
use serde::{Deserialize, Serialize};
use walkdir::WalkDir;

use crate::canon::split_frontmatter;
use crate::cli::{WorkArgs, WorkCommand, WorkStatus, WorkVerdict};

use super::print_json;

#[derive(Debug, Clone, Serialize, Deserialize)]
struct WorkMeta {
    id: String,
    title: String,
    status: WorkStatus,
    #[serde(default)]
    verdict: WorkVerdict,
    created: String,
    #[serde(default)]
    tags: Vec<String>,
}

#[derive(Debug, Clone, Serialize)]
struct WorkItem {
    #[serde(flatten)]
    meta: WorkMeta,
    path: String,
    body: String,
    files: Vec<String>,
}

#[derive(Debug, Serialize)]
struct WorkBoard {
    todo: Vec<WorkItem>,
    doing: Vec<WorkItem>,
    review: Vec<WorkItem>,
    done: Vec<WorkItem>,
}

pub fn run(project_root: &Path, args: WorkArgs, json: bool) -> Result<()> {
    match args.command {
        WorkCommand::List { status } => list(project_root, status, json),
        WorkCommand::New {
            title,
            tags,
            criterion,
        } => new(project_root, &title, tags, criterion, json),
        WorkCommand::Show { id } => show(project_root, &id, json),
        WorkCommand::Move {
            id,
            status,
            verdict,
        } => move_item(project_root, &id, status, verdict, json),
        WorkCommand::Board => board(project_root, json),
    }
}

fn list(project_root: &Path, status: Option<WorkStatus>, json: bool) -> Result<()> {
    let mut items = load_items(project_root)?;
    if let Some(status) = status {
        items.retain(|item| item.meta.status == status);
    }
    if json {
        return print_json(&items);
    }
    if items.is_empty() {
        println!("no work items");
        return Ok(());
    }
    println!(
        "{:<8} {:<8} {:<8} {:<12} TITLE",
        "ID", "STATUS", "VERDICT", "CREATED"
    );
    for item in items {
        println!(
            "{:<8} {:<8} {:<8} {:<12} {}",
            item.meta.id, item.meta.status, item.meta.verdict, item.meta.created, item.meta.title
        );
    }
    Ok(())
}

fn new(
    project_root: &Path,
    title: &str,
    tags: Vec<String>,
    criteria: Vec<String>,
    json: bool,
) -> Result<()> {
    let title = title.trim();
    if title.is_empty() {
        bail!("work-item title cannot be empty");
    }

    let items = load_items(project_root)?;
    let work_directory = project_root.join(".base/work");
    let next = next_work_number(&items, &work_directory)?;
    let id = format!("W-{next:04}");
    let slug = slugify(title);
    let folder_name = if slug.is_empty() {
        id.clone()
    } else {
        format!("{id}-{slug}")
    };
    let folder = work_directory.join(folder_name);
    fs::create_dir_all(&folder)
        .with_context(|| format!("cannot create work-item folder {}", folder.display()))?;
    let path = folder.join("item.md");

    let criteria: Vec<String> = criteria
        .into_iter()
        .map(|criterion| criterion.trim().to_owned())
        .filter(|criterion| !criterion.is_empty())
        .collect();
    let mut body = format!("# {title}\n\n## Acceptance Criteria");
    if !criteria.is_empty() {
        body.push('\n');
    }
    for criterion in criteria {
        body.push_str("\n- [ ] ");
        body.push_str(&criterion);
    }

    let item = WorkItem {
        meta: WorkMeta {
            id,
            title: title.to_owned(),
            status: WorkStatus::Todo,
            verdict: WorkVerdict::Pending,
            created: Utc::now().date_naive().to_string(),
            tags: tags
                .into_iter()
                .map(|tag| tag.trim().to_owned())
                .filter(|tag| !tag.is_empty())
                .collect(),
        },
        path: relative(project_root, &path),
        body,
        files: Vec::new(),
    };
    write_item(project_root, &item)?;

    if json {
        print_json(&item)
    } else {
        println!("created {} at {}", item.meta.id, item.path);
        Ok(())
    }
}

fn show(project_root: &Path, id: &str, json: bool) -> Result<()> {
    let items = load_items(project_root)?;
    let item = find_item(&items, id)?;
    if json {
        print_json(item)
    } else {
        println!("{}", fs::read_to_string(project_root.join(&item.path))?);
        Ok(())
    }
}

fn move_item(
    project_root: &Path,
    id: &str,
    status: WorkStatus,
    verdict: Option<WorkVerdict>,
    json: bool,
) -> Result<()> {
    let items = load_items(project_root)?;
    let mut item = find_item(&items, id)?.clone();
    let (total, checked) = criteria_counts(&item.body).with_context(|| {
        format!(
            "{} is missing its ## Acceptance Criteria section",
            item.meta.id
        )
    })?;
    if total == 0 {
        bail!("{} has no acceptance criteria", item.meta.id);
    }

    if status == WorkStatus::Done {
        let verdict = match verdict {
            Some(WorkVerdict::Pass | WorkVerdict::Fail) => verdict.expect("matched some verdict"),
            Some(WorkVerdict::Pending) | None => bail!(
                "moving {} to done requires --verdict pass|fail",
                item.meta.id
            ),
        };
        item.meta.verdict = verdict;
    } else if verdict.is_some() {
        bail!("--verdict only applies when moving to done");
    } else if item.meta.status == WorkStatus::Done {
        item.meta.verdict = WorkVerdict::Pending;
    }

    item.meta.status = status;
    if status == WorkStatus::Done && item.meta.verdict == WorkVerdict::Pass && checked < total {
        eprintln!(
            "warning: moving {} to done with pass while {} acceptance criteria remain unchecked",
            item.meta.id,
            total - checked
        );
    }
    write_item(project_root, &item)?;

    if json {
        print_json(&item)
    } else {
        println!(
            "moved {} to {} with verdict {}",
            item.meta.id, item.meta.status, item.meta.verdict
        );
        Ok(())
    }
}

fn board(project_root: &Path, json: bool) -> Result<()> {
    let items = load_items(project_root)?;
    let mut board = WorkBoard {
        todo: Vec::new(),
        doing: Vec::new(),
        review: Vec::new(),
        done: Vec::new(),
    };
    for item in items {
        match item.meta.status {
            WorkStatus::Todo => board.todo.push(item),
            WorkStatus::Doing => board.doing.push(item),
            WorkStatus::Review => board.review.push(item),
            WorkStatus::Done => board.done.push(item),
        }
    }
    for column in [
        &mut board.todo,
        &mut board.doing,
        &mut board.review,
        &mut board.done,
    ] {
        column.sort_by(|left, right| left.meta.id.cmp(&right.meta.id));
    }

    if json {
        return print_json(&board);
    }
    if board.todo.is_empty()
        && board.doing.is_empty()
        && board.review.is_empty()
        && board.done.is_empty()
    {
        println!("no work items");
        return Ok(());
    }

    const WIDTH: usize = 19;
    println!(
        "{} | {} | {} | {}",
        cell(&format!("TODO ({})", board.todo.len()), WIDTH),
        cell(&format!("DOING ({})", board.doing.len()), WIDTH),
        cell(&format!("REVIEW ({})", board.review.len()), WIDTH),
        cell(&format!("DONE ({})", board.done.len()), WIDTH)
    );
    println!("{}", vec!["-".repeat(WIDTH); 4].join("-+-"));

    let rows = [
        board.todo.len(),
        board.doing.len(),
        board.review.len(),
        board.done.len(),
    ]
    .into_iter()
    .max()
    .unwrap_or(0);
    for row in 0..rows {
        println!(
            "{} | {} | {} | {}",
            board_cell(board.todo.get(row), false, WIDTH),
            board_cell(board.doing.get(row), false, WIDTH),
            board_cell(board.review.get(row), false, WIDTH),
            board_cell(board.done.get(row), true, WIDTH)
        );
    }
    Ok(())
}

fn board_cell(item: Option<&WorkItem>, show_verdict: bool, width: usize) -> String {
    let Some(item) = item else {
        return " ".repeat(width);
    };
    let text = if show_verdict {
        let marker = match item.meta.verdict {
            WorkVerdict::Pass => '✓',
            WorkVerdict::Fail => '✗',
            WorkVerdict::Pending => '?',
        };
        format!("{} {marker} {}", item.meta.id, item.meta.title)
    } else {
        format!("{} {}", item.meta.id, item.meta.title)
    };
    cell(&text, width)
}

fn load_items(project_root: &Path) -> Result<Vec<WorkItem>> {
    let directory = project_root.join(".base/work");
    if !directory.is_dir() {
        return Ok(Vec::new());
    }

    let mut paths: Vec<PathBuf> = fs::read_dir(&directory)?
        .filter_map(|entry| entry.ok().map(|entry| entry.path()))
        .collect();
    paths.sort();

    let mut items = Vec::new();
    for path in paths {
        if path.is_dir() {
            let item_path = path.join("item.md");
            if !item_path.is_file() {
                eprintln!(
                    "warning: ignoring work directory without item.md {}",
                    relative(project_root, &path)
                );
                continue;
            }
            let source = fs::read_to_string(&item_path)
                .with_context(|| format!("cannot read work item {}", item_path.display()))?;
            let (frontmatter, body) = split_frontmatter(&source)
                .with_context(|| format!("invalid work item {}", item_path.display()))?;
            let meta: WorkMeta = serde_yaml::from_str(frontmatter)
                .with_context(|| format!("invalid work metadata {}", item_path.display()))?;
            let mut files = Vec::new();
            for entry in WalkDir::new(&path).min_depth(1) {
                let entry = entry
                    .with_context(|| format!("cannot walk work item folder {}", path.display()))?;
                if entry.file_type().is_file() && entry.path() != item_path {
                    files.push(relative(project_root, entry.path()));
                }
            }
            files.sort();
            items.push(WorkItem {
                meta,
                path: relative(project_root, &item_path),
                body: body.to_owned(),
                files,
            });
        } else if path.file_name().and_then(|name| name.to_str()) == Some(".gitkeep") {
            continue;
        } else if path.extension().and_then(|value| value.to_str()) == Some("md") {
            eprintln!(
                "warning: ignoring legacy work file {}",
                relative(project_root, &path)
            );
        }
    }
    items.sort_by(|left, right| left.path.cmp(&right.path));
    Ok(items)
}

fn find_item<'a>(items: &'a [WorkItem], id: &str) -> Result<&'a WorkItem> {
    let normalized = id.trim().to_ascii_lowercase();
    items
        .iter()
        .find(|item| {
            item.meta.id.to_ascii_lowercase() == normalized
                || Path::new(&item.path)
                    .parent()
                    .and_then(Path::file_name)
                    .and_then(|name| name.to_str())
                    .is_some_and(|name| name.to_ascii_lowercase() == normalized)
        })
        .with_context(|| format!("work item `{id}` not found"))
}

fn write_item(project_root: &Path, item: &WorkItem) -> Result<()> {
    let path = project_root.join(&item.path);
    let frontmatter = serde_yaml::to_string(&item.meta)?;
    let source = format!(
        "---\n{}\n---\n\n{}\n",
        frontmatter.trim_end(),
        item.body.trim()
    );
    fs::write(&path, source).with_context(|| format!("cannot write {}", path.display()))
}

fn criteria_counts(body: &str) -> Option<(usize, usize)> {
    let mut in_section = false;
    let mut total = 0;
    let mut checked = 0;
    for line in body.lines() {
        let trimmed = line.trim();
        if !in_section {
            if trimmed.eq_ignore_ascii_case("## Acceptance Criteria") {
                in_section = true;
            }
            continue;
        }
        if trimmed.starts_with("# ") || trimmed.starts_with("## ") {
            break;
        }
        if trimmed.starts_with("- [ ]") {
            total += 1;
        } else if trimmed.starts_with("- [x]") || trimmed.starts_with("- [X]") {
            total += 1;
            checked += 1;
        }
    }
    in_section.then_some((total, checked))
}

fn next_work_number(items: &[WorkItem], work_directory: &Path) -> Result<u32> {
    let mut maximum = items
        .iter()
        .filter_map(|item| work_number(&item.meta.id))
        .max()
        .unwrap_or(0);
    if work_directory.is_dir() {
        for entry in fs::read_dir(work_directory)? {
            let entry = entry?;
            if entry.path().is_dir()
                && let Some(number) = entry.file_name().to_str().and_then(work_number)
            {
                maximum = maximum.max(number);
            }
        }
    }
    maximum
        .checked_add(1)
        .context("work-item ID space exhausted")
}

fn work_number(value: &str) -> Option<u32> {
    let digits: String = value
        .strip_prefix("W-")?
        .chars()
        .take_while(char::is_ascii_digit)
        .collect();
    (!digits.is_empty()).then(|| digits.parse().ok()).flatten()
}

fn cell(text: &str, width: usize) -> String {
    if width == 0 {
        return String::new();
    }
    let count = text.chars().count();
    if count > width {
        let mut truncated: String = text.chars().take(width - 1).collect();
        truncated.push('…');
        truncated
    } else {
        format!("{text}{}", " ".repeat(width - count))
    }
}

fn slugify(value: &str) -> String {
    let mut output = String::new();
    let mut pending_hyphen = false;
    for character in value.chars().flat_map(char::to_lowercase) {
        if character.is_ascii_alphanumeric() {
            if pending_hyphen && !output.is_empty() {
                output.push('-');
            }
            pending_hyphen = false;
            output.push(character);
        } else {
            pending_hyphen = true;
        }
    }
    output.truncate(
        output
            .char_indices()
            .nth(48)
            .map_or(output.len(), |(index, _)| index),
    );
    output.trim_end_matches('-').to_owned()
}

fn relative(root: &Path, path: &Path) -> String {
    path.strip_prefix(root)
        .unwrap_or(path)
        .to_string_lossy()
        .replace('\\', "/")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn titles_become_safe_slugs() {
        assert_eq!(slugify("Fix: Auth / Retry"), "fix-auth-retry");
    }

    #[test]
    fn criteria_counts_only_the_acceptance_section() {
        let body = "# Item\n\n## acceptance criteria\n- [ ] one\n  - [x] two\n- [X] three\n### Detail\n- [ ] four\n## Evidence\n- [x] ignored\n";
        assert_eq!(criteria_counts(body), Some((4, 2)));
        assert_eq!(criteria_counts("# Item"), None);
        assert_eq!(
            criteria_counts("## Acceptance Criteria\nNothing yet"),
            Some((0, 0))
        );
    }

    #[test]
    fn board_cells_truncate_multibyte_text_by_character() {
        assert_eq!(cell("éééé", 3), "éé…");
        assert_eq!(cell("✓", 3), "✓  ");
        assert_eq!(cell("anything", 0), "");
    }
}
