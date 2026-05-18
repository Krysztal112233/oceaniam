use garde::Validate;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Debug, Serialize, Deserialize, Clone, ToSchema)]
pub struct PageInfo {
    pub has_next: bool,
    pub total: usize,
}

#[derive(Debug, Serialize, Deserialize, Clone, ToSchema)]
pub struct PagedResponse<T> {
    pub items: Vec<T>,
    pub page_info: PageInfo,
}

impl<T> PagedResponse<T> {
    pub fn with_entire<I>(data: I) -> Self
    where
        I: IntoIterator<Item = T>,
    {
        let items = data.into_iter().collect::<Vec<_>>();

        let page_info = PageInfo {
            has_next: false,
            total: items.len(),
        };

        Self { items, page_info }
    }
}

impl<T> Default for PagedResponse<T> {
    fn default() -> Self {
        Self::with_entire(vec![])
    }
}

#[derive(Debug, Serialize, Deserialize, Validate, Clone, Copy, ToSchema)]
pub struct PageParam {
    #[garde(skip)]
    pub page: u64,

    #[garde(range(min = 0, max = 1024))]
    pub per_page: u64,
}

impl PageParam {
    /// Return a clamped copy with `per_page` bounded to `[1, 100]`.
    pub fn into_clamped(self) -> Self {
        self.into_clamped_by(100)
    }

    /// Return a clamped copy with `per_page` bounded to `[1, max_per_page]`.
    pub fn into_clamped_by(self, max_per_page: u64) -> Self {
        Self {
            page: self.page,
            per_page: self.per_page.clamp(1, max_per_page),
        }
    }

    /// SQL `OFFSET` from a 1-based page number.
    pub fn as_offset(&self) -> u64 {
        (self.page.saturating_sub(1)) * self.per_page
    }

    /// SQL `LIMIT`.
    pub fn as_limit(&self) -> u64 {
        self.per_page
    }
}

impl Default for PageParam {
    fn default() -> Self {
        Self {
            page: 0,
            per_page: 30,
        }
    }
}
