mod data;

#[cfg(feature = "phone-numbers")]
impl TryFrom<crate::Iso3166_1> for phonenumber::country::Id {
    type Error = crate::ParseIsoCodeError;
    fn try_from(value: crate::Iso3166_1) -> Result<Self, Self::Error> {
        value.code().parse().map_err(|_| crate::ParseIsoCodeError {
            standard: "phone-region",
            input: value.code().to_owned(),
        })
    }
}

#[cfg(feature = "phone-numbers")]
impl TryFrom<phonenumber::country::Id> for crate::Iso3166_1 {
    type Error = crate::ParseIsoCodeError;
    fn try_from(value: phonenumber::country::Id) -> Result<Self, Self::Error> {
        value.as_ref().parse()
    }
}
