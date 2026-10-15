use iso_types::{Iso3166_1 as Country, VariantArray as _};
use iso_types_locations::{LocationFunction, UnLocode};

#[test]
fn complete_directory_round_trips_and_country_slices_partition_the_identities() {
    let mut count = 0;
    let mut records = 0;
    for &country in Country::VARIANTS {
        for location in UnLocode::for_country(country) {
            assert_eq!(location.country(), country);
            assert_eq!(location.code().parse::<UnLocode>().unwrap(), location);
            assert_eq!(UnLocode::try_from(location.code()).unwrap(), location);
            for record in location.records() {
                assert_eq!(record.location, location);
                assert!(!record.name.is_empty());
                if let Some(point) = record.coordinates() {
                    assert!((-90.0..=90.0).contains(&point.latitude_degrees()));
                    assert!((-180.0..=180.0).contains(&point.longitude_degrees()));
                }
                records += 1;
            }
            count += 1;
        }
    }
    assert!(count > 100_000);
    assert!(records > count);
    assert_eq!(count, UnLocode::COUNT);
    assert_eq!(records, UnLocode::RECORD_COUNT);
    assert_eq!(UnLocode::iter().count(), count);
    for bad in ["", "US NYC", "usnyc", "USNY", "ZZZZZ", "USNYC "] {
        assert!(bad.parse::<UnLocode>().is_err(), "{bad}");
    }
}

#[test]
fn alternatives_preserve_conflicting_published_metadata() {
    let helsinki = "FIHEL".parse::<UnLocode>().unwrap();
    let records: Vec<_> = helsinki.records().collect();
    assert_eq!(records.len(), 2);
    assert_ne!(records[0].name, records[1].name);
    assert_ne!(records[0].status, records[1].status);
    assert!(records[0].coordinates().is_none());
    assert!(records[1].coordinates().is_some());
    assert!(
        records[1]
            .functions()
            .any(|function| function == LocationFunction::Airport)
    );
    let airport = "USLEB".parse::<UnLocode>().unwrap();
    let subdivisions: std::collections::BTreeSet<_> = airport
        .records()
        .filter_map(|record| record.subdivision_code)
        .collect();
    assert_eq!(subdivisions, ["NH", "VT"].into_iter().collect());
    #[cfg(feature = "subdivisions")]
    for record in airport.records() {
        assert_eq!(record.subdivision().unwrap().country(), Country::Us);
    }
}
