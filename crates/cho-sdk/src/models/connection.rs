//! Connection model for the Xero Identity API.
//!
//! The Identity API (`GET /connections`) returns the list of tenants
//! (organisations) the authenticated user has access to.

use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// A connection (tenant) returned by the Xero Identity API.
///
/// Each connection represents an organisation the user has authorized access to.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Connection {
    /// Unique identifier for the connection.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<Uuid>,

    /// Tenant ID (used as `xero-tenant-id` header).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tenant_id: Option<Uuid>,

    /// Tenant type (e.g., "ORGANISATION").
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tenant_type: Option<String>,

    /// Organisation name.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tenant_name: Option<String>,

    /// When the connection was created.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub created_date_utc: Option<NaiveDateTime>,

    /// When the connection was last updated.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub updated_date_utc: Option<NaiveDateTime>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn connection_deserialize() {
        let json = r#"[{
            "id": "a3e5f9b4-2c3d-4e5f-b6a7-8d9e0f1a2b3c",
            "tenantId": "d7e5f9b4-2c3d-4e5f-b6a7-8d9e0f1a2b3c",
            "tenantType": "ORGANISATION",
            "tenantName": "Demo Company (NZ)"
        }]"#;
        let connections: Vec<Connection> = serde_json::from_str(json).unwrap();
        assert_eq!(connections.len(), 1);
        assert_eq!(
            connections[0].tenant_name.as_deref(),
            Some("Demo Company (NZ)")
        );
        assert_eq!(connections[0].tenant_type.as_deref(), Some("ORGANISATION"));
    }
}
