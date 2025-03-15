pub fn bytes_to_pretty_string(bytes: i64) -> String {
    // Using integer ranges to choose the proper unit.
    match bytes {
        // 0..=1023: show as bytes.
        0..=1023 => format!("{} B", bytes),
        // 1024..=1_048_575: KB range.
        1024..=1_048_575 => {
            // Only perform the division if needed.
            format!("{:.2} KB", bytes as f64 / 1024.0)
        }
        // 1_048_576..=1_073_741_823: MB range.
        1_048_576..=1_073_741_823 => {
            format!("{:.2} MB", bytes as f64 / 1_048_576.0)
        }
        // 1_073_741_824..=1_099_511_627_775: GB range.
        1_073_741_824..=1_099_511_627_775 => {
            format!("{:.2} GB", bytes as f64 / 1_073_741_824.0)
        }
        // Anything larger is formatted in TB.
        _ => {
            format!("{:.2} TB", bytes as f64 / 1_099_511_627_776.0)
        }
    }
}
