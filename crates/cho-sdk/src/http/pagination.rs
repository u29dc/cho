//! Auto-pagination for Xero API list endpoints.
//!
//! Returns `impl Stream<Item = Result<T>>` that transparently fetches pages
//! until all items are retrieved or a limit is reached.

use serde::de::DeserializeOwned;

use crate::models::common::Pagination;

/// Parameters controlling pagination behavior.
#[derive(Debug, Clone)]
pub struct PaginationParams {
    /// Maximum number of items to return (default: 100, 0 = no limit).
    pub limit: usize,

    /// Page size (default: 100, Xero max).
    pub page_size: u32,
}

impl Default for PaginationParams {
    fn default() -> Self {
        Self {
            limit: 100,
            page_size: 100,
        }
    }
}

impl PaginationParams {
    /// Creates params that fetch all items (no limit).
    pub fn all() -> Self {
        Self {
            limit: 0,
            page_size: 100,
        }
    }

    /// Creates params with a specific item limit.
    pub fn with_limit(limit: usize) -> Self {
        Self {
            limit,
            page_size: 100,
        }
    }
}

/// Trait for collection response types that contain paginated items.
///
/// Implement this for each Xero collection wrapper (e.g., `Invoices`, `Contacts`).
pub trait PaginatedResponse: DeserializeOwned {
    /// The item type within the collection.
    type Item;

    /// Extracts the items from the response, consuming self.
    fn into_items(self) -> Vec<Self::Item>;

    /// Returns the pagination metadata, if present.
    fn pagination(&self) -> Option<&Pagination>;
}

/// A single page result from the API.
#[derive(Debug)]
pub struct PageResult<T> {
    /// Items on this page.
    pub items: Vec<T>,

    /// Whether there are more pages.
    pub has_more: bool,

    /// Current page number.
    pub page: u32,
}

/// Determines if there are more pages based on pagination metadata.
pub fn has_more_pages(pagination: Option<&Pagination>, current_page: u32) -> bool {
    match pagination {
        Some(p) => {
            if let Some(page_count) = p.page_count {
                current_page < page_count
            } else {
                false
            }
        }
        None => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pagination_params_default() {
        let params = PaginationParams::default();
        assert_eq!(params.limit, 100);
        assert_eq!(params.page_size, 100);
    }

    #[test]
    fn pagination_params_all() {
        let params = PaginationParams::all();
        assert_eq!(params.limit, 0);
    }

    #[test]
    fn has_more_pages_logic() {
        let pag = Pagination {
            page: Some(1),
            page_size: Some(100),
            page_count: Some(3),
            item_count: Some(250),
        };
        assert!(has_more_pages(Some(&pag), 1));
        assert!(has_more_pages(Some(&pag), 2));
        assert!(!has_more_pages(Some(&pag), 3));
        assert!(!has_more_pages(None, 1));
    }
}
