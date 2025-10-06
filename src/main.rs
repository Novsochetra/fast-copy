use clap::Parser;
use indicatif::{ProgressBar, ProgressStyle};
use rayon::prelude::*;
use std::collections::HashSet;
use std::fs;
use std::os::unix::fs as unix_fs;
use std::path::Path;
use walkdir::WalkDir;

/// Fast multi-threaded backup
#[derive(Parser, Debug)]
struct Args {
    src: String,
    dst: String,
}

fn main() {
    let args = Args::parse();
    let src = args.src;
    let dst = args.dst;

    if !Path::new(&src).exists() {
        eprintln!("Error: Source directory '{}' does not exist!", src);
        std::process::exit(1);
    }

    println!("Backing up from '{}' → '{}'", src, dst);

    let mut files = Vec::new();
    let mut dirs = HashSet::new();

    // Walk directory tree
    for entry in WalkDir::new(&src).into_iter().filter_map(|e| e.ok()) {
        let path = entry.path();

        if let Some(name) = path.file_name() {
            let s = name.to_string_lossy();
            if s.starts_with("._") || s == ".DS_Store" {
                continue;
            }
        }

        if entry.file_type().is_file() || entry.file_type().is_symlink() {
            files.push(path.to_path_buf());
        } else if entry.file_type().is_dir() {
            dirs.insert(path.to_path_buf());
        }
    }

    // Sort dirs by depth
    let mut dirs: Vec<_> = dirs.into_iter().collect();
    dirs.sort_by_key(|d| d.components().count());

    // Create dirs in parallel
    dirs.par_iter().for_each(|dir| {
        let relative = dir.strip_prefix(&src).unwrap();
        let dest_path = Path::new(&dst).join(relative);
        let _ = fs::create_dir_all(&dest_path);
    });

    // Progress bar
    let pb = ProgressBar::new(files.len() as u64);
    pb.set_style(
        ProgressStyle::with_template(
            "{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {pos}/{len} {msg}",
        )
        .unwrap()
        .progress_chars("#>-"),
    );

    // Copy files + symlinks
    files.par_iter().for_each(|file| {
        let relative = file.strip_prefix(&src).unwrap();
        let dest_path = Path::new(&dst).join(relative);

        if let Ok(metadata) = fs::symlink_metadata(file) {
            if metadata.file_type().is_symlink() {
                // Preserve symlink target
                if let Ok(target) = fs::read_link(file) {
                    let _ = unix_fs::symlink(&target, &dest_path);
                }
            } else {
                // Normal file copy
                if let Err(err) = fs::copy(file, &dest_path) {
                    eprintln!("Failed to copy {}: {}", file.display(), err);
                }
            }
        }

        pb.inc(1);
    });

    pb.finish_with_message("✅ Backup complete!");
}
