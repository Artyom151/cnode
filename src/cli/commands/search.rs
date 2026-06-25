use crate::error::Result;
use crate::registry::Registry;

pub async fn handle(
    query: String,
    limit: usize,
    registry_url: Option<String>,
) -> Result<()> {
    let registry = Registry::new(registry_url);
    let results = registry.search(&query, limit).await?;

    if results.is_empty() {
        return Ok(());
    }

    let name_width = results.iter().map(|p| p.name.len()).fold(20, |a, b| a.max(b));
    let name_width = name_width.min(50) + 2;

    println!("{:<w$} {:<12} {}", "NAME", "VERSION", "DESCRIPTION", w = name_width);
    println!("{}", "-".repeat(100));

    for pkg in &results {
        let desc = pkg.description.as_deref().unwrap_or("");
        let truncated_desc = if desc.len() > 60 { &desc[..57] } else { desc };
        println!("{:<w$} {:<12} {}", pkg.name, pkg.version, truncated_desc, w = name_width);
    }

    Ok(())
}
