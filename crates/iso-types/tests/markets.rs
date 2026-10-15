#![cfg(feature = "10383")]

use iso_types::{
    Iso3166_1 as Country, Iso10383, MarketRegistry, MarketRegistryError, MicDate, MicStatus,
};

const HEADER: &str = "MIC,OPERATING MIC,OPRT/SGMT,MARKET NAME-INSTITUTION DESCRIPTION,ISO COUNTRY CODE (ISO 3166),STATUS,CREATION DATE,LAST UPDATE DATE,LAST VALIDATION DATE,EXPIRY DATE\n";
const ROWS: &str = "AAAA,AAAA,OPRT,Example operating market,US,ACTIVE,20200101,20200101,20200101,\nAAAB,AAAA,SGMT,Example segment,US,ACTIVE,20210101,20210101,20210101,\nAAAC,AAAA,SGMT,Future segment,US,ACTIVE,20261026,20261026,20261026,\nAAAD,AAAA,SGMT,Pending expiry,US,EXPIRED,20210101,20261026,20261026,20261026\nAAAE,AAAA,SGMT,Expired segment,US,EXPIRED,20210101,20260101,20260101,20260101\n";

#[test]
fn dates_codes_and_effective_registry_graphs_preserve_source_status() {
    let as_of: MicDate = "2026-10-15".parse().unwrap();
    assert_eq!(as_of, "20261015".parse().unwrap());
    assert_eq!(as_of.to_string(), "2026-10-15");
    for input in [
        "20260229",
        "2026-13-01",
        "2026-01-00",
        "-026-10-15",
        "2026101",
        "00000101",
    ] {
        assert!(input.parse::<MicDate>().is_err(), "{input}");
    }
    assert!("20240229".parse::<MicDate>().is_ok());
    for input in ["nasd", "ABC", "ABCDE", "A BC", "ÄBCD"] {
        assert!(input.parse::<Iso10383>().is_err(), "{input}");
    }
    let text = format!("{HEADER}{ROWS}");
    let registry = MarketRegistry::from_csv(text.as_bytes(), as_of).unwrap();
    assert_eq!(registry.len(), 3);
    let operating = "AAAA".parse().unwrap();
    assert_eq!(registry.segments(operating).count(), 2);
    assert_eq!(registry.for_country(Country::Us).count(), 3);
    assert_eq!(registry.for_country(Country::Ca).count(), 0);
    assert!(registry.get("AAAC".parse().unwrap()).is_none());
    assert!(registry.get("AAAE".parse().unwrap()).is_none());
    let expiring = registry.get("AAAD".parse().unwrap()).unwrap();
    assert_eq!(expiring.reported_status, MicStatus::Expired);
    assert_eq!(expiring.last_update_date, Some("20261026".parse().unwrap()));
    assert!(expiring.is_active_on(as_of));
    assert!(!expiring.is_active_on("20261026".parse().unwrap()));
}

#[test]
fn quoted_csv_and_diagnostics_use_real_records() {
    let as_of = "20261015".parse().unwrap();
    let quoted = format!(
        "{HEADER}AAAA,AAAA,OPRT,\"Example, \"\"quoted\"\" market\nsecond line\",US,ACTIVE,20200101,,,\n"
    );
    let registry = MarketRegistry::from_csv(quoted.as_bytes(), as_of).unwrap();
    assert_eq!(
        registry.iter().next().unwrap().name(),
        "Example, \"quoted\" market\nsecond line"
    );
    let orphan = format!("{HEADER}AAAB,AAAA,SGMT,Example segment,US,ACTIVE,20200101,,,\n");
    assert!(matches!(
        MarketRegistry::from_csv(orphan.as_bytes(), as_of),
        Err(MarketRegistryError::InvalidOperatingMic { .. })
    ));
    let duplicate = format!("{HEADER}{ROWS}AAAA,AAAA,OPRT,Duplicate,US,ACTIVE,20200101,,,\n");
    assert!(matches!(
        MarketRegistry::from_csv(duplicate.as_bytes(), as_of),
        Err(MarketRegistryError::DuplicateCode(_))
    ));
    assert!(matches!(
        MarketRegistry::from_csv("MIC\nAAAA\n".as_bytes(), as_of),
        Err(MarketRegistryError::MissingColumn(_))
    ));
}
