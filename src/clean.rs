use std::{fs, path::Path};

pub fn handle_clean_command(
    input: String,
    output: Option<String>,
    backup: bool,
    report: bool,
    dry_run: bool,
) -> anyhow::Result<()> {
    use std::io::Read;

    let input_path = Path::new(&input);
    if !input_path.exists() {
        return Err(anyhow::anyhow!("Input file does not exist: {}", input));
    }

    // Read the file as bytes to handle null bytes properly
    let mut file = fs::File::open(&input)?;
    let mut buffer = Vec::new();
    file.read_to_end(&mut buffer)?;

    // Clean the content
    let (cleaned_content, stats) = clean_content(&buffer, report || dry_run);

    if report || dry_run {
        print_clean_report(&stats, dry_run);
    }

    if dry_run {
        return Ok(());
    }

    // Determine output path
    let output_path = output.as_deref().unwrap_or(&input);

    // Create backup if requested
    if backup && output_path == input {
        let backup_path = format!("{input}.backup");
        fs::copy(&input, &backup_path)?;
        println!("Created backup: {backup_path}");
    }

    // Write cleaned content
    fs::write(output_path, cleaned_content)?;
    println!("Cleaned file written to: {output_path}");

    Ok(())
}

#[derive(Default)]
struct CleanStats {
    null_bytes: usize,
    control_chars: usize,
    total_bytes: usize,
    lines_affected: usize,
}

fn clean_content(buffer: &[u8], collect_stats: bool) -> (String, CleanStats) {
    let mut stats = CleanStats {
        total_bytes: buffer.len(),
        ..Default::default()
    };

    let mut cleaned = Vec::new();
    let mut current_line_affected = false;

    for &byte in buffer {
        match byte {
            0 => {
                // Null byte - remove it
                if collect_stats {
                    stats.null_bytes += 1;
                    current_line_affected = true;
                }
            }
            1..=8 | 11..=12 | 14..=31 => {
                // Control characters (except \t=9, \n=10, \r=13)
                if collect_stats {
                    stats.control_chars += 1;
                    current_line_affected = true;
                }
            }
            b'\n' => {
                cleaned.push(byte);
                if collect_stats && current_line_affected {
                    stats.lines_affected += 1;
                    current_line_affected = false;
                }
            }
            _ => {
                cleaned.push(byte);
            }
        }
    }

    // Handle last line if it doesn't end with newline
    if collect_stats && current_line_affected {
        stats.lines_affected += 1;
    }

    let cleaned_string = String::from_utf8_lossy(&cleaned).to_string();
    (cleaned_string, stats)
}

fn print_clean_report(stats: &CleanStats, dry_run: bool) {
    let action = if dry_run { "Would remove" } else { "Removed" };

    println!("\n=== Clean Report ===");
    println!("Total bytes processed: {}", stats.total_bytes);
    println!("{} null bytes: {}", action, stats.null_bytes);
    println!("{} control characters: {}", action, stats.control_chars);
    println!("Lines affected: {}", stats.lines_affected);

    if stats.null_bytes == 0 && stats.control_chars == 0 {
        println!("✓ No cleaning needed - file is already clean!");
    } else {
        let total_removed = stats.null_bytes + stats.control_chars;
        println!(
            "Total characters {}: {}",
            if dry_run { "to remove" } else { "removed" },
            total_removed
        );
    }
}
