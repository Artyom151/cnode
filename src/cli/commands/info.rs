use crate::error::Result;
use crate::registry::Registry;

pub async fn handle(package: String, registry_url: Option<String>) -> Result<()> {
    let registry = Registry::new(registry_url);
    let metadata = registry.fetch_package_metadata(&package).await?;

    let latest = &metadata.dist_tags.latest;
    let latest_version = metadata.versions.get(latest);

    println!("{}", package);
    println!("{}", "-".repeat(50));
    println!("{:<20} {}", "Latest:", latest);
    println!("{:<20} {}", "Total versions:", metadata.versions.len());

    if let Some(v) = latest_version {
        if let Some(desc) = &v.description {
            println!();
            println!("Description:");
            println!("  {}", desc);
        }

        let deps_count = v.dependencies.as_ref().map(|d| d.len()).unwrap_or(0);
        println!("{:<20} {}", "Dependencies:", deps_count);
        println!("{:<20} {}", "Integrity:", v.dist.integrity.as_deref().unwrap_or("(none)"));
    }

    let time_count = metadata.time.len();
    println!("{:<20} {}", "Published versions:", time_count);

    println!();
    println!("Dist-tags:");
    println!("  latest: {}", latest);

    Ok(())
}
