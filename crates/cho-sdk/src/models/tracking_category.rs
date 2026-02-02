//! Tracking category model for the Xero TrackingCategories API.

use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::common::{Pagination, Warning};
use super::enums::{TrackingCategoryStatus, TrackingOptionStatus};

/// A tracking category in Xero, used to assign categories to transactions.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct TrackingCategory {
    /// Unique identifier for the tracking category.
    #[serde(rename = "TrackingCategoryID")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tracking_category_id: Option<Uuid>,

    /// Tracking option ID (used when a specific option is selected).
    #[serde(rename = "TrackingOptionID")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tracking_option_id: Option<Uuid>,

    /// Name of the tracking category.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,

    /// Status of the tracking category.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status: Option<TrackingCategoryStatus>,

    /// Available options for this tracking category.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub options: Option<Vec<TrackingOption>>,
}

/// A tracking option within a tracking category.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct TrackingOption {
    /// Unique identifier for the tracking option.
    #[serde(rename = "TrackingOptionID")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tracking_option_id: Option<Uuid>,

    /// Name of the tracking option.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,

    /// Status of the tracking option.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status: Option<TrackingOptionStatus>,
}

/// Collection wrapper for tracking categories returned by the Xero API.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct TrackingCategories {
    /// List of tracking categories.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tracking_categories: Option<Vec<TrackingCategory>>,

    /// Pagination metadata (tracking categories endpoint is not paginated, but included for consistency).
    #[serde(rename = "pagination")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pagination: Option<Pagination>,

    /// Warnings returned by the API.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub warnings: Option<Vec<Warning>>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tracking_category_deserialize_basic() {
        let json = r#"{
            "TrackingCategoryID": "a1b2c3d4-e5f6-7890-abcd-ef1234567890",
            "Name": "Region",
            "Status": "ACTIVE",
            "Options": [
                {
                    "TrackingOptionID": "b2c3d4e5-f6a7-8901-bcde-f12345678901",
                    "Name": "North",
                    "Status": "ACTIVE"
                },
                {
                    "TrackingOptionID": "c3d4e5f6-a7b8-9012-cdef-123456789012",
                    "Name": "South",
                    "Status": "ACTIVE"
                },
                {
                    "TrackingOptionID": "d4e5f6a7-b8c9-0123-defa-234567890123",
                    "Name": "East",
                    "Status": "ARCHIVED"
                }
            ]
        }"#;
        let tc: TrackingCategory = serde_json::from_str(json).unwrap();
        assert_eq!(tc.name.as_deref(), Some("Region"));
        assert_eq!(tc.status, Some(TrackingCategoryStatus::Active));

        let options = tc.options.as_ref().unwrap();
        assert_eq!(options.len(), 3);
        assert_eq!(options[0].name.as_deref(), Some("North"));
        assert_eq!(options[0].status, Some(TrackingOptionStatus::Active));
        assert_eq!(options[2].status, Some(TrackingOptionStatus::Archived));
    }

    #[test]
    fn tracking_categories_collection() {
        let json = r#"{
            "TrackingCategories": [
                {
                    "TrackingCategoryID": "a1b2c3d4-e5f6-7890-abcd-ef1234567890",
                    "Name": "Region",
                    "Status": "ACTIVE"
                },
                {
                    "TrackingCategoryID": "b2c3d4e5-f6a7-8901-bcde-f12345678901",
                    "Name": "Department",
                    "Status": "ACTIVE"
                }
            ]
        }"#;
        let tcs: TrackingCategories = serde_json::from_str(json).unwrap();
        assert_eq!(tcs.tracking_categories.as_ref().unwrap().len(), 2);
    }
}
