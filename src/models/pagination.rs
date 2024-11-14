use leptos::*;
use leptos_router::*;

#[derive(Debug, Params, PartialEq, Clone)]
pub struct Pagination {
    tag: Option<String>,
    my_feed: Option<bool>,
    page: Option<u32>,
    amount: Option<u32>,
}

impl Pagination {
    #[inline]
    pub fn get_tag(&self) -> &str {
        self.tag.as_deref().unwrap_or_default()
    }

    #[inline]
    pub fn get_my_feed(&self) -> bool {
        self.my_feed.unwrap_or_default()
    }

    #[inline]
    pub fn get_page(&self) -> u32 {
        self.page.unwrap_or_default()
    }

    #[inline]
    pub fn get_amount(&self) -> u32 {
        self.amount.unwrap_or(10)
    }

    #[inline]
    pub fn set_tag<T: ToString + ?Sized>(mut self, tag: &T) -> Self {
        self.tag = Some(tag.to_string());
        self
    }

    #[inline]
    pub fn set_amount(mut self, amount: u32) -> Self {
        self.amount = Some(amount.clamp(1, 100));
        self
    }

    #[inline]
    pub fn set_my_feed(mut self, feed: bool) -> Self {
        self.my_feed = Some(feed);
        self
    }

    #[inline]
    pub fn reset_page(mut self) -> Self {
        self.page = Some(0);
        self
    }

    #[inline]
    pub fn next_page(mut self) -> Self {
        self.page = Some(self.page.unwrap_or_default().saturating_add(1));
        self
    }

    #[inline]
    pub fn previous_page(mut self) -> Self {
        self.page = Some(self.page.unwrap_or_default().saturating_sub(1));
        self
    }
}

impl Default for Pagination {
    fn default() -> Self {
        Self {
            tag: None,
            my_feed: Some(false),
            page: Some(0),
            amount: Some(10),
        }
    }
}

impl ToString for Pagination {
    fn to_string(&self) -> String {
        let mut params = Vec::new();
        
        if !self.get_tag().is_empty() {
            params.push(format!("tag={}", self.get_tag()));
        }
        if self.get_my_feed() {
            params.push(format!("my_feed=true"));
        }
        if self.get_page() > 0 {
            params.push(format!("page={}", self.get_page()));
        }
        if self.get_amount() != 10 {
            params.push(format!("amount={}", self.get_amount()));
        }

        if params.is_empty() {
            "/".to_string()
        } else {
            format!("/?{}", params.join("&"))
        }
    }
}
