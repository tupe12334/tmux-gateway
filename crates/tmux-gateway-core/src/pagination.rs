/// Pagination parameters for list operations.
#[derive(Debug, Clone, Default)]
pub struct Pagination {
    /// Number of items to skip (0-based offset).
    pub offset: usize,
    /// Maximum number of items to return (`None` = unlimited).
    pub limit: Option<usize>,
}

/// A page of results with metadata about the full collection.
#[derive(Debug, Clone)]
pub struct PaginatedResult<T> {
    /// The items in this page.
    pub items: Vec<T>,
    /// Total count of items before pagination was applied.
    pub total: usize,
    /// Whether more items exist beyond the returned page.
    pub has_more: bool,
}

/// Apply pagination to a collected `Vec<T>`.
pub(crate) fn paginate<T>(all: Vec<T>, pagination: &Pagination) -> PaginatedResult<T> {
    let total = all.len();
    let offset = pagination.offset.min(total);
    let items: Vec<T> = match pagination.limit {
        Some(limit) => all.into_iter().skip(offset).take(limit).collect(),
        None => all.into_iter().skip(offset).collect(),
    };
    let has_more = offset + items.len() < total;
    PaginatedResult {
        items,
        total,
        has_more,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_data() -> Vec<i32> {
        vec![1, 2, 3, 4, 5]
    }

    #[test]
    fn default_pagination_returns_all() {
        let result = paginate(sample_data(), &Pagination::default());
        assert_eq!(result.items, vec![1, 2, 3, 4, 5]);
        assert_eq!(result.total, 5);
        assert!(!result.has_more);
    }

    #[test]
    fn limit_only() {
        let result = paginate(
            sample_data(),
            &Pagination {
                offset: 0,
                limit: Some(3),
            },
        );
        assert_eq!(result.items, vec![1, 2, 3]);
        assert_eq!(result.total, 5);
        assert!(result.has_more);
    }

    #[test]
    fn offset_only() {
        let result = paginate(
            sample_data(),
            &Pagination {
                offset: 2,
                limit: None,
            },
        );
        assert_eq!(result.items, vec![3, 4, 5]);
        assert_eq!(result.total, 5);
        assert!(!result.has_more);
    }

    #[test]
    fn offset_and_limit() {
        let result = paginate(
            sample_data(),
            &Pagination {
                offset: 1,
                limit: Some(2),
            },
        );
        assert_eq!(result.items, vec![2, 3]);
        assert_eq!(result.total, 5);
        assert!(result.has_more);
    }

    #[test]
    fn offset_at_end() {
        let result = paginate(
            sample_data(),
            &Pagination {
                offset: 5,
                limit: Some(10),
            },
        );
        assert!(result.items.is_empty());
        assert_eq!(result.total, 5);
        assert!(!result.has_more);
    }

    #[test]
    fn offset_beyond_total() {
        let result = paginate(
            sample_data(),
            &Pagination {
                offset: 100,
                limit: Some(10),
            },
        );
        assert!(result.items.is_empty());
        assert_eq!(result.total, 5);
        assert!(!result.has_more);
    }

    #[test]
    fn limit_larger_than_remaining() {
        let result = paginate(
            sample_data(),
            &Pagination {
                offset: 3,
                limit: Some(100),
            },
        );
        assert_eq!(result.items, vec![4, 5]);
        assert_eq!(result.total, 5);
        assert!(!result.has_more);
    }

    #[test]
    fn empty_input() {
        let result = paginate(
            Vec::<i32>::new(),
            &Pagination {
                offset: 0,
                limit: Some(10),
            },
        );
        assert!(result.items.is_empty());
        assert_eq!(result.total, 0);
        assert!(!result.has_more);
    }

    #[test]
    fn limit_zero() {
        let result = paginate(
            sample_data(),
            &Pagination {
                offset: 0,
                limit: Some(0),
            },
        );
        assert!(result.items.is_empty());
        assert_eq!(result.total, 5);
        assert!(result.has_more);
    }
}
