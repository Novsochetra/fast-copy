use clap::Parser;
use indicatif::{ProgressBar, ProgressStyle};
use rayon::prelude::*;
use std::fs;
use std::path::Path;
use walkdir::WalkDir;

/// Simple multi-threaded backup tool in Rust (no system cp)
#[derive(Parser, Debug)]
#[command(author, version, about = "Fast multi-threaded backup tool")]
struct Args {
    /// Source directory
    src: String,

    /// Destination directory
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

    // Collect all files and sort by depth (deepest first) for optimization
    let mut entries: Vec<_> = WalkDir::new(&src)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.file_type().is_file())
        .collect();

    entries.sort_by_key(|e| std::cmp::Reverse(e.path().components().count()));

    if entries.is_empty() {
        println!("No files found in source directory. Nothing to backup.");
        return;
    }

    let pb = ProgressBar::new(entries.len() as u64);
    pb.set_style(
        ProgressStyle::with_template(
            "{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {pos}/{len} {msg}",
        )
        .unwrap()
        .progress_chars("#>-"),
    );

    entries.par_iter().for_each(|entry| {
        let relative_path = entry.path().strip_prefix(&src).unwrap();
        let dest_path = Path::new(&dst).join(relative_path);

        // Create parent directories
        if let Err(err) = fs::create_dir_all(dest_path.parent().unwrap()) {
            eprintln!(
                "Failed to create directory {:?}: {}",
                dest_path.parent(),
                err
            );
            pb.inc(1);
            return;
        }

        // Copy file in Rust (no external command)
        if let Err(err) = fs::copy(entry.path(), &dest_path) {
            eprintln!("Failed to copy {}: {}", entry.path().display(), err);
        }

        pb.inc(1);
    });

    pb.finish_with_message("✅ Backup complete!");
}
