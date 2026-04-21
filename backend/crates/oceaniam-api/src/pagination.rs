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
    pub fn as_offset(&self) -> u64 {
        (self.page.saturating_sub(1)) * self.per_page
    }

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
