//! Implements helper functions for using vendored/cformat.rs
use std::convert::TryFrom;
use std::str::FromStr;

use rustc_hash::FxHashSet;

use crate::vendored::cformat::{
    CFormatError, CFormatPart, CFormatQuantity, CFormatSpec, CFormatString,
};

pub(crate) struct CFormatSummary {
    pub starred: bool,
    pub num_positional: usize,
    pub keywords: FxHashSet<String>,
}

impl TryFrom<&str> for CFormatSummary {
    type Error = CFormatError;

    fn try_from(literal: &str) -> Result<Self, Self::Error> {
        let format_string = CFormatString::from_str(literal)?;

        let mut starred = false;
        let mut num_positional = 0;
        let mut keywords = FxHashSet::default();

        for format_part in format_string.parts {
            let CFormatPart::Spec(CFormatSpec {
                mapping_key,
                min_field_width,
                precision,
                ..
            }) = format_part.1 else
            {
                continue;
            };
            match mapping_key {
                Some(k) => {
                    keywords.insert(k);
                }
                None => {
                    num_positional += 1;
                }
            };
            if min_field_width == Some(CFormatQuantity::FromValuesTuple) {
                num_positional += 1;
                starred = true;
            }
            if precision == Some(CFormatQuantity::FromValuesTuple) {
                num_positional += 1;
                starred = true;
            }
        }

        Ok(CFormatSummary {
            starred,
            num_positional,
            keywords,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cformat_summary() {
        let literal = "%(foo)s %s %d %(bar)x";

        let expected_positional = 2;
        let expected_keywords = ["foo", "bar"].into_iter().map(String::from).collect();

        let format_summary = CFormatSummary::try_from(literal).unwrap();
        assert!(!format_summary.starred);
        assert_eq!(format_summary.num_positional, expected_positional);
        assert_eq!(format_summary.keywords, expected_keywords);
    }

    #[test]
    fn test_cformat_summary_starred() {
        let format_summary1 = CFormatSummary::try_from("%*s %*d").unwrap();
        assert!(format_summary1.starred);
        assert_eq!(format_summary1.num_positional, 4);

        let format_summary2 = CFormatSummary::try_from("%s %.*d").unwrap();
        assert!(format_summary2.starred);
        assert_eq!(format_summary2.num_positional, 3);

        let format_summary3 = CFormatSummary::try_from("%s %*.*d").unwrap();
        assert!(format_summary3.starred);
        assert_eq!(format_summary3.num_positional, 4);

        let format_summary4 = CFormatSummary::try_from("%s %1d").unwrap();
        assert!(!format_summary4.starred);
    }

    #[test]
    fn test_cformat_summary_invalid() {
        assert!(CFormatSummary::try_from("%").is_err());
        assert!(CFormatSummary::try_from("%(foo).").is_err());
    }
}
