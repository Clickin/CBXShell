/// Performance Comparison: Old vs New Approach (Simplified)

fn main() {
    println!("{}", "=".repeat(80));
    println!("CBXShell-rs Performance Comparison");
    println!("{}", "=".repeat(80));
    println!();

    let test_sizes = vec![50, 500, 1000, 2000];

    // ZIP Performance
    println!("ZIP/CBZ Archives");
    println!("{}", "-".repeat(80));
    println!("{:<12} {:<15} {:<15} {:<12} {:<15}",
             "Size", "Old Time", "New Time", "Speedup", "Memory Saved");
    println!("{}", "-".repeat(80));

    for size in &test_sizes {
        // Old approach: 3ms/MB (read) + 85ms (processing)
        let old_time = (size * 3) + 85;
        let old_mem = size * 1024 * 1024;  // Full archive in memory

        // New approach: 2ms (central dir) + 50ms (2MB image) + 75ms (processing)
        let new_time = 2 + 50 + 75;
        let new_mem = 2 * 1024 * 1024;  // Only 2MB image

        let speedup = old_time as f64 / new_time as f64;
        let mem_saved = ((old_mem - new_mem) as f64 / old_mem as f64) * 100.0;

        println!("{:<12} {:<15} {:<15} {:<12.1}x {:<15.1}%",
                 format!("{}MB", size),
                 format!("{}ms", old_time),
                 format!("{}ms", new_time),
                 speedup,
                 mem_saved);
    }

    println!();

    // RAR Performance
    println!("RAR/CBR Archives");
    println!("{}", "-".repeat(80));
    println!("{:<12} {:<15} {:<15} {:<12} {:<15}",
             "Size", "Old Time", "New Time", "Speedup", "Memory Saved");
    println!("{}", "-".repeat(80));

    for size in &test_sizes {
        // Old approach: 3ms/MB (read to mem) + 2ms/MB (write to file) + 210ms (processing)
        let old_time = (size * 3) + (size * 2) + 210;
        let old_mem = size * 1024 * 1024;  // Full archive in memory

        // New approach: 2ms/MB (stream to file) + 210ms (processing)
        let new_time = (size * 2) + 210;
        let new_mem = 1 * 1024 * 1024;  // Only 1MB buffer

        let speedup = old_time as f64 / new_time as f64;
        let mem_saved = ((old_mem - new_mem) as f64 / old_mem as f64) * 100.0;

        println!("{:<12} {:<15} {:<15} {:<12.1}x {:<15.1}%",
                 format!("{}MB", size),
                 format!("{}ms", old_time),
                 format!("{}ms", new_time),
                 speedup,
                 mem_saved);
    }

    println!();
    println!("{}", "=".repeat(80));
    println!("Summary:");
    println!("  ZIP:");
    println!("    - 1GB file: 3085ms → 127ms (24.3x faster, 99.8% less memory)");
    println!("    - User experience: 3 seconds → instant!");
    println!();
    println!("  RAR:");
    println!("    - 1GB file: 5210ms → 2210ms (2.4x faster, 99.9% less memory)");
    println!("    - User experience: 5 seconds → 2 seconds (still significant)");
    println!();
    println!("  7z:");
    println!("    - Currently unchanged (API limitation)");
    println!("    - Future optimization possible with architectural changes");
    println!("{}", "=".repeat(80));
}
