use regex::Regex;
use std::fs;
use walkdir::WalkDir;

fn main() {
    let replacements = vec![
        ("directory_type_id", "network_type_id"),
        ("directory_id", "network_id"),
        ("directory_type", "network_type"),
        ("directory.localhost", "network.localhost"),
        ("directory.atlas.", "network.atlas."),
        ("directory-instance", "network-instance"),
        ("directory_instance", "network_instance"),
        ("DirectoryService", "NetworkService"),
        ("DirectoryConfig", "NetworkConfig"),
        ("Directory Offline", "Network Offline"),
        ("Directory Type", "Network Type"),
        ("Directory type", "Network type"),
        ("Directory Filter", "Network Filter"),
        ("Join our growing directory", "Join our growing network"),
        ("Join Our Directory", "Join Our Network"),
        ("Browse Directory", "Browse Network"),
        ("Search Directory", "Search Network"),
        ("View All Directory", "View All Network"),
        ("Assign to Directory", "Assign to Network"),
        ("Specific Directory", "Specific Network"),
        ("directory's listings", "network's listings"),
        ("Listings Directory", "Listings Network"),
        ("BusinessDirectory", "BusinessNetwork"),
        ("/directory/", "/network/"),
        ("\"directory\"", "\"network\""),
        ("Directories", "Networks"),
        ("directories", "networks"),
        ("Directory", "Network"),
        ("directory", "network"),
    ];

    let root_path = "/Users/oply/src/git/orbit_/atlas-platform";

    for entry in WalkDir::new(root_path)
        .into_iter()
        .filter_entry(|e| {
            let name = e.file_name().to_string_lossy().to_string();
            let ignored = vec![".git", "target", "node_modules", "dist"];
            !ignored.contains(&name.as_str())
        })
    {
        let entry = match entry {
            Ok(e) => e,
            Err(_) => continue,
        };

        if !entry.file_type().is_file() {
            continue;
        }

        let name = entry.file_name().to_string_lossy().to_string();
        if !name.ends_with(".rs") && !name.ends_with(".md") && !name.ends_with(".yaml")
            && !name.ends_with(".yml") && !name.ends_with(".toml") && !name.ends_with(".json")
            && !name.ends_with(".html") && !name.ends_with(".css") && !name.ends_with(".js")
            && !name.ends_with(".ts") && !name.ends_with("Caddyfile") && !name.ends_with(".sql")
        {
            continue;
        }

        let path = entry.path();
        
        // We skip processing the replacer code itself to not overwrite our logic
        if path.display().to_string().contains("replacer") {
            continue;
        }

        let content = match fs::read_to_string(path) {
            Ok(c) => c,
            Err(_) => continue,
        };

        let mut new_content = content.clone();

        for (old, new) in &replacements {
            new_content = new_content.replace(old, *new);
        }

        if new_content != content {
            if let Err(e) = fs::write(path, new_content) {
                eprintln!("Failed to write {}: {}", path.display(), e);
            } else {
                println!("Updated {}", path.display());
            }
        }
    }
}
