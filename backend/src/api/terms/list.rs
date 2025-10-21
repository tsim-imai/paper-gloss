use crate::api::error::AppError;
use crate::models::Term;
use crate::services::terms::{TermSearchResult, TermSearchService};
use axum::{
    extract::{Query, State},
    Json,
};
use serde::{Deserialize, Serialize};
use sqlx::SqlitePool;

#[derive(Debug, Deserialize)]
pub struct TermListQuery {
    pub q: Option<String>,          // Search query
    pub lang: Option<String>,        // "en", "ja", or "both"
    pub sort: Option<String>,        // "alphabetical" or "frequency"
    pub page: Option<i64>,           // Page number (default 1)
    pub limit: Option<i64>,          // Results per page (default 50)
}

#[derive(Debug, Serialize)]
pub struct TermListItem {
    pub id: String,
    pub slug: String,
    pub lemma_en: String,
    pub lemma_ja: String,
    pub reading_kana: Option<String>,
    pub pos: Option<String>,
    pub tags: Option<String>,
    pub occurrence_count: i64,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Serialize)]
pub struct TermListResponse {
    pub items: Vec<TermListItem>,
    pub total: i64,
    pub page: i64,
    pub limit: i64,
    pub total_pages: i64,
}

/// GET /terms - List and search terms
pub async fn list_terms(
    State(pool): State<SqlitePool>,
    Query(query): Query<TermListQuery>,
) -> Result<Json<TermListResponse>, AppError> {
    // Validate lang parameter
    let lang = query.lang.as_deref().unwrap_or("both");
    if !["en", "ja", "both"].contains(&lang) {
        return Err(AppError::BadRequest(
            "Invalid lang parameter. Must be 'en', 'ja', or 'both'".to_string(),
        ));
    }

    // Validate sort parameter
    let sort = query.sort.as_deref().unwrap_or("alphabetical");
    if !["alphabetical", "frequency", "recent"].contains(&sort) {
        return Err(AppError::BadRequest(
            "Invalid sort parameter. Must be 'alphabetical', 'frequency', or 'recent'".to_string(),
        ));
    }

    // Validate page and limit
    let page = query.page.unwrap_or(1);
    if page < 1 {
        return Err(AppError::BadRequest(
            "Invalid page parameter. Must be >= 1".to_string(),
        ));
    }

    let limit = query.limit.unwrap_or(50);
    if limit < 1 {
        return Err(AppError::BadRequest(
            "Invalid limit parameter. Must be >= 1".to_string(),
        ));
    }
    let limit = limit.min(100); // Cap at 100

    let search_service = TermSearchService::new(pool);

    let (results, total) = search_service
        .search(query.q.as_deref(), lang, sort, page, limit)
        .await
        .map_err(|e| AppError::InternalServerError(format!("Search failed: {}", e)))?;

    let items: Vec<TermListItem> = results
        .into_iter()
        .map(|r| term_search_result_to_item(r))
        .collect();

    let total_pages = (total as f64 / limit as f64).ceil() as i64;

    Ok(Json(TermListResponse {
        items,
        total,
        page,
        limit,
        total_pages,
    }))
}

fn term_search_result_to_item(result: TermSearchResult) -> TermListItem {
    TermListItem {
        id: result.term.id,
        slug: result.term.slug,
        lemma_en: result.term.lemma_en,
        lemma_ja: result.term.lemma_ja,
        reading_kana: result.term.reading_kana,
        pos: result.term.pos,
        tags: result.term.tags,
        occurrence_count: result.occurrence_count,
        created_at: result.term.created_at.to_rfc3339(),
        updated_at: result.term.updated_at.to_rfc3339(),
    }
}
