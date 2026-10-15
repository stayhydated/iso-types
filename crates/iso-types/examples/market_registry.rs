use iso_types::{MarketRegistry, MicDate};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut args = std::env::args().skip(1);
    let path = args.next().ok_or("usage: market_registry CSV YYYY-MM-DD")?;
    let as_of: MicDate = args
        .next()
        .ok_or("usage: market_registry CSV YYYY-MM-DD")?
        .parse()?;
    let registry = MarketRegistry::from_csv(std::fs::File::open(path)?, as_of)?;
    println!(
        "{} effective market identities at {}",
        registry.len(),
        registry.as_of()
    );
    Ok(())
}
