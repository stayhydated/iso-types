#[cfg(feature = "time-zones")]
#[test]
fn all_selected_zones_load_in_jiff_and_country_navigation_is_closed() {
    use iso_types::{Iso3166_1 as Country, TimeZoneId, VariantArray as _};
    let winter = "2026-01-15T12:00Z"
        .parse::<iso_types::jiff::Timestamp>()
        .unwrap();
    let summer = "2026-07-15T12:00Z"
        .parse::<iso_types::jiff::Timestamp>()
        .unwrap();
    let ny = TimeZoneId::AmericaNewYork;
    assert_eq!(ny.at(winter).unwrap().offset().seconds(), -5 * 3600);
    assert_eq!(ny.at(summer).unwrap().offset().seconds(), -4 * 3600);
    for &zone in TimeZoneId::VARIANTS {
        let rules = zone.time_zone().unwrap();
        assert_eq!(TimeZoneId::try_from(&rules).unwrap(), zone);
        for &country in zone.countries() {
            assert!(country.time_zones().contains(&zone));
        }
    }
    for &country in Country::VARIANTS {
        for zone in country.time_zones() {
            assert!(zone.countries().contains(&country));
        }
    }
    assert!(Country::Us.time_zones().contains(&ny));
    assert_eq!("UTC".parse::<TimeZoneId>().unwrap(), TimeZoneId::EtcUtc);
    assert!(TimeZoneId::try_from(&iso_types::jiff::tz::TimeZone::unknown()).is_err());
}

#[cfg(feature = "time-zones")]
#[test]
fn jiff_exposes_explicit_resolution_of_dst_gaps_and_overlaps() {
    use iso_types::{TimeZoneId, jiff::civil::date};
    let rules = TimeZoneId::AmericaNewYork.time_zone().unwrap();
    let gap = date(2026, 3, 8).at(2, 30, 0, 0);
    let overlap = date(2026, 11, 1).at(1, 30, 0, 0);
    assert!(rules.to_ambiguous_zoned(gap).unambiguous().is_err());
    assert!(rules.to_ambiguous_zoned(overlap).unambiguous().is_err());
    let earlier = rules.to_ambiguous_zoned(overlap).earlier().unwrap();
    let later = rules.to_ambiguous_zoned(overlap).later().unwrap();
    assert_eq!(
        later.timestamp().as_second() - earlier.timestamp().as_second(),
        3600
    );
}

#[cfg(feature = "time-zone-localization")]
#[test]
fn icu_zone_identities_and_localized_names_match_the_pinned_catalog() {
    use iso_types::{TimeZoneId, VariantArray as _, icu_locale::locale};
    for &zone in TimeZoneId::VARIANTS {
        let icu = zone.icu_time_zone();
        assert_eq!(zone.bcp47(), Some(icu.as_str()));
        assert_eq!(TimeZoneId::try_from(icu).unwrap().icu_time_zone(), icu);
        assert_eq!(
            zone.localized_name(&locale!("en")).unwrap(),
            zone.english_name()
        );
    }
    assert_ne!(
        TimeZoneId::AmericaNewYork
            .localized_name(&locale!("fr"))
            .unwrap(),
        TimeZoneId::AmericaNewYork.english_name()
    );
    assert!(TimeZoneId::try_from(iso_types::icu_time::TimeZone::UNKNOWN).is_err());
}

#[cfg(feature = "regions")]
#[test]
fn un_region_hierarchy_keeps_forward_and_reverse_relationships_consistent() {
    use iso_types::{GeographicRegion as Region, Iso3166_1 as Country, VariantArray as _};
    assert_eq!(
        Country::Us.geographic_region(),
        Some(Region::NorthernAmerica)
    );
    assert_eq!(Country::Ke.geographic_region(), Some(Region::EasternAfrica));
    assert_eq!(
        Region::EasternAfrica.parent(),
        Some(Region::SubSaharanAfrica)
    );
    assert_eq!(Region::SubSaharanAfrica.parent(), Some(Region::Africa));
    assert_eq!(Region::World.parent(), None);
    assert_eq!(Region::World.countries().len(), 248);
    for &country in Country::VARIANTS {
        let chain = country.geographic_regions();
        for pair in chain.windows(2) {
            assert_eq!(pair[1].parent(), Some(pair[0]));
        }
        for region in chain {
            assert!(region.countries().contains(&country));
        }
        assert_eq!(country.geographic_region(), chain.last().copied());
    }
    for &region in Region::VARIANTS {
        for child in region.children() {
            assert_eq!(child.parent(), Some(region));
        }
        for country in region.countries() {
            assert!(country.geographic_regions().contains(&region));
        }
    }
}

#[cfg(feature = "regional-preferences")]
#[test]
fn territory_defaults_cover_week_calendar_clock_and_measurement_exceptions() {
    use iso_types::{
        CalendarAlgorithm as Calendar, HourCycle, Iso3166_1 as Country, Iso8601Weekday as Day,
        MeasurementSystem, PaperSize, WeekPreferences,
    };
    let us = Country::Us.regional_preferences();
    assert_eq!(us.week.first_weekday, Day::Sunday);
    assert_eq!(us.hour_cycle, HourCycle::H12);
    assert_eq!(us.measurement_system, MeasurementSystem::USSystem);
    assert_eq!(us.paper_size, PaperSize::UsLetter);
    assert!(us.week.is_weekend(Day::Saturday));
    assert!(!us.week.is_weekend(Day::Friday));
    let gb = Country::Gb.regional_preferences();
    assert_eq!(gb.week.first_weekday, Day::Monday);
    assert_eq!(gb.week.min_week_days, 4);
    assert_eq!(gb.hour_cycle, HourCycle::H23);
    assert_eq!(gb.paper_size, PaperSize::A4);
    assert_eq!(
        Country::Th.regional_preferences().preferred_calendars[0],
        Calendar::Buddhist
    );
    assert_eq!(
        Country::Ir.regional_preferences().preferred_calendars[0],
        Calendar::Persian
    );
    assert_eq!(
        Country::Mm.regional_preferences().measurement_system,
        MeasurementSystem::UKSystem
    );
    assert_eq!(
        Country::Mm
            .regional_preferences()
            .temperature_measurement_system,
        MeasurementSystem::Metric
    );
    assert!(
        Country::Af
            .regional_preferences()
            .week
            .is_weekend(Day::Thursday)
    );
    assert!(
        !Country::Af
            .regional_preferences()
            .week
            .is_weekend(Day::Sunday)
    );
    let wrapped = WeekPreferences {
        weekend_start: Day::Sunday,
        weekend_end: Day::Monday,
        ..us.week
    };
    assert!(wrapped.is_weekend(Day::Sunday));
    assert!(wrapped.is_weekend(Day::Monday));
    assert!(!wrapped.is_weekend(Day::Saturday));
}

#[cfg(feature = "unit-preferences")]
#[test]
fn unit_preferences_preserve_ranked_choices_mixed_units_and_skeletons() {
    use iso_types::{Iso3166_1 as Country, UnitUsage as Usage};
    let height = Country::Us.unit_preferences(Usage::LengthPersonHeight);
    assert_eq!(
        height[0].units.iter().map(|u| u.code()).collect::<Vec<_>>(),
        ["foot", "inch"]
    );
    assert_eq!(height[0].threshold, 3.0);
    assert_eq!(height[1].units[0].code(), "inch");
    assert_eq!(
        Country::Us.unit_preferences(Usage::TemperatureWeather)[0].units[0].code(),
        "fahrenheit"
    );
    assert_eq!(
        Country::Fr.unit_preferences(Usage::TemperatureWeather)[0].units[0].code(),
        "celsius"
    );
    assert_eq!(
        Country::Gb.unit_preferences(Usage::ConsumptionVehicleFuel)[0].units[0].code(),
        "mile-per-gallon-imperial"
    );
    let road = Country::Us.unit_preferences(Usage::LengthRoad);
    assert_eq!(road[0].units[0].code(), "mile");
    assert_eq!(road[0].threshold, 0.5);
    assert_eq!(road[1].skeleton, Some("precision-increment/50"));
}

#[cfg(all(feature = "unit-preferences", feature = "unit-conversion"))]
#[test]
fn every_preference_component_has_valid_icu_unit_syntax() {
    use iso_types::{Iso3166_1, UnitUsage, VariantArray as _};
    let mut seen = std::collections::HashSet::new();
    for &country in Iso3166_1::VARIANTS {
        for &usage in UnitUsage::VARIANTS {
            for preference in country.unit_preferences(usage) {
                assert!(!preference.units.is_empty());
                for &unit in preference.units {
                    if seen.insert(unit.code()) {
                        let parsed = unit.icu_unit().unwrap();
                        if let Some(catalog) = unit.catalog_unit() {
                            assert_eq!(&parsed, catalog.icu_unit());
                        }
                    }
                }
            }
        }
    }
}

#[cfg(feature = "language-relations")]
#[test]
fn language_graph_and_direction_use_the_common_scope_and_icu() {
    use iso_types::{
        Iso3166_1 as Country, Iso6391 as Language, Iso15924 as Script, VariantArray as _,
    };
    assert!(Country::Ca.languages().contains(&Language::En));
    assert!(Country::Ca.languages().contains(&Language::Fr));
    assert!(Language::Sr.scripts().contains(&Script::Cyrl));
    assert!(Language::Sr.scripts().contains(&Script::Latn));
    assert!(Language::Zh.scripts().contains(&Script::Hant));
    assert_eq!(
        Script::Arab.direction(),
        Some(iso_types::icu_locale::Direction::RightToLeft)
    );
    assert_eq!(
        Script::Latn.direction(),
        Some(iso_types::icu_locale::Direction::LeftToRight)
    );
    for &country in Country::VARIANTS {
        for language in country.languages() {
            assert!(language.countries().contains(&country));
        }
        for usage in country.language_usage() {
            assert!(country.languages().contains(&usage.language));
        }
    }
    for &language in Language::VARIANTS {
        for country in language.countries() {
            assert!(country.languages().contains(&language));
        }
        for script in language.scripts() {
            assert!(script.languages().contains(&language));
        }
    }
}

#[cfg(all(feature = "language-relations", feature = "639-2", feature = "639-3"))]
#[test]
fn three_letter_language_relationships_cover_their_complete_selected_catalogs() {
    use iso_types::{Iso3166_1, Iso6392, Iso6393, VariantArray as _};
    for &country in Iso3166_1::VARIANTS {
        for language in country.languages_6392() {
            assert!(language.countries().contains(&country));
        }
        for language in country.languages_6393() {
            assert!(language.countries().contains(&country));
        }
    }
    for &language in Iso6392::VARIANTS {
        for country in language.countries() {
            assert!(country.languages_6392().contains(&language));
        }
    }
    for &language in Iso6393::VARIANTS {
        for country in language.countries() {
            assert!(country.languages_6393().contains(&language));
        }
    }
}

#[cfg(feature = "calling-codes")]
#[test]
fn shared_and_global_calling_codes_keep_many_to_many_associations() {
    use iso_types::{CallingCode, Iso3166_1 as Country, VariantArray as _};
    assert_eq!(Country::Us.calling_codes(), &[CallingCode::C1]);
    assert!(CallingCode::C1.countries().contains(&Country::Ca));
    assert!(CallingCode::C1.countries().contains(&Country::Us));
    assert!(CallingCode::C7.countries().contains(&Country::Kz));
    assert!(CallingCode::C800.countries().is_empty());
    assert_eq!("+44".parse::<CallingCode>().unwrap().number(), 44);
    for &country in Country::VARIANTS {
        for code in country.calling_codes() {
            assert!(code.countries().contains(&country));
        }
    }
}

#[cfg(feature = "phone-numbers")]
#[test]
fn phone_region_conversion_integrates_with_real_parsing_and_validation() {
    use iso_types::{Iso3166_1 as Country, phonenumber};
    let region = phonenumber::country::Id::try_from(Country::Us).unwrap();
    assert_eq!(Country::try_from(region).unwrap(), Country::Us);
    let phone = phonenumber::parse(Some(region), "202-555-0123").unwrap();
    assert!(phonenumber::is_valid(&phone));
    assert_eq!(
        phone.format().mode(phonenumber::Mode::E164).to_string(),
        "+12025550123"
    );
    assert!(Country::try_from(phonenumber::country::Id::AC).is_err());
}

#[cfg(feature = "addresses")]
#[test]
fn address_metadata_preserves_country_formats_required_fields_and_typed_labels() {
    use iso_types::{
        AddressField as Field, AddressFieldLabel as Label, Iso3166_1 as Country, VariantArray as _,
    };
    let us = Country::Us.address_metadata().unwrap();
    assert!(us.is_required(Field::AdministrativeArea));
    assert!(us.is_required(Field::PostalCode));
    assert_eq!(us.field_label(Field::AdministrativeArea), Label::State);
    assert_eq!(us.field_label(Field::PostalCode), Label::Zip);
    assert_eq!(
        Country::Ca
            .address_metadata()
            .unwrap()
            .field_label(Field::AdministrativeArea),
        Label::Province
    );
    assert_eq!(
        Country::Ie
            .address_metadata()
            .unwrap()
            .field_label(Field::PostalCode),
        Label::Eircode
    );
    assert!(!Country::Hk.address_metadata().unwrap().uses_postal_code());
    assert_eq!(
        Country::Hk
            .address_metadata()
            .unwrap()
            .label_override("C", Some("zh-HK"))
            .unwrap()
            .label,
        Some("地區")
    );
    assert!(
        Country::Hk
            .address_metadata()
            .unwrap()
            .label_override("C", Some("xx"))
            .is_none()
    );
    assert_eq!(
        Country::In
            .address_metadata()
            .unwrap()
            .field_label(Field::PostalCode),
        Label::Pin
    );
    assert!(
        Country::In
            .address_metadata()
            .unwrap()
            .uses(Field::LandmarkName)
    );
    assert_eq!(
        Country::My
            .address_metadata()
            .unwrap()
            .field_label(Field::DependentLocality),
        Label::VillageTownship
    );
    for country in Country::VARIANTS {
        let metadata = country.address_metadata().unwrap();
        for field in metadata.required_fields {
            assert!(metadata.uses(*field));
        }
        for field in metadata.fields {
            assert!(!metadata.field_label(*field).english_name().is_empty());
        }
    }
}
