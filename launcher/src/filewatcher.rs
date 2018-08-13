use backend::{BackendRequest, Build, Project};
use notify::{DebouncedEvent, RecommendedWatcher, RecursiveMode, Watcher};
use state::State;
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::mpsc::channel;
use std::time::Duration;
use Result;

pub fn run(base_dir: &str) -> Result<()> {
    let projects = ::backend::get_projects(base_dir)?;
    let (tx, rx) = channel();
    let mut watcher: RecommendedWatcher = Watcher::new(tx, Duration::from_secs(1))?;

    let mut path_to_project = HashMap::new();

    for project in &projects {
        let mut path = PathBuf::from(base_dir);
        path.push(&project.name);
        for build in &project.builds {
            let mut path = path.clone();
            println!(
                "Watching for {:?}, triggering {}::{}",
                path, project.name, build.name
            );
            watcher.watch(&path, RecursiveMode::Recursive)?;
            let mut p = path_to_project.entry(path).or_insert_with(HashMap::new);
            for extension in build.pattern.split(',') {
                if extension.is_empty() {
                    continue;
                }
                p.entry(extension)
                    .or_insert_with(Vec::new)
                    .push((project, build));
            }
        }
    }

    let current_dir = ::std::env::current_dir()?;

    loop {
        let event = rx.recv()?;

        let mut builds_to_trigger: Vec<(&Project, &Build)> = Vec::new();
        let file;
        match &event {
            DebouncedEvent::Create(path)
            | DebouncedEvent::Write(path)
            | DebouncedEvent::Remove(path) => {
                file = path;
                let stripped_prefix = path.strip_prefix(&current_dir)?;
                let extension = match stripped_prefix.extension().and_then(|e| e.to_str()) {
                    Some(e) => format!(".{}", e),
                    None => continue,
                };
                for (key, value) in &path_to_project {
                    if stripped_prefix.starts_with(key) {
                        if let Some(project_and_builds) = value.get(extension.as_str()) {
                            builds_to_trigger.extend(project_and_builds);
                        }
                    }
                }
            }

            DebouncedEvent::NoticeWrite(_) | DebouncedEvent::NoticeRemove(_) => continue,
            DebouncedEvent::Rename(_, _) => continue,

            x => {
                println!("{:?}", x);
                continue;
            }
        };

        if !builds_to_trigger.is_empty() {
            State::get(|state| {
                for (project, build) in builds_to_trigger {
                    println!(
                        "Triggering {}::{} because of {:?}",
                        project.name, build.name, file
                    );
                    state.backend_sender.send(BackendRequest::StartBuild {
                        project_name: project.name.clone(),
                        build_name: build.name.clone(),
                    })?;
                }
                Ok(())
            })?;
        }
    }
}
