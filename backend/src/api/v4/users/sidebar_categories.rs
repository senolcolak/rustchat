use axum::{
    extract::{Path, Query, State},
    Json,
};
use chrono::Utc;
use serde::Deserialize;
use uuid::Uuid;

use super::MmAuthUser;
use crate::api::AppState;
use crate::error::{ApiResult, AppError};
use crate::mattermost_compat::{
    id::{encode_mm_id, parse_mm_or_uuid},
    models as mm,
};

#[derive(Deserialize)]
pub(super) struct CategoriesPath {
    user_id: String,
}

pub(super) async fn get_categories(
    State(state): State<AppState>,
    auth: MmAuthUser,
    Path(params): Path<CategoriesPath>,
    Query(query): Query<std::collections::HashMap<String, String>>,
) -> ApiResult<Json<mm::SidebarCategories>> {
    let user_id = resolve_user_id(&params.user_id, &auth)?;
    let team_id_str = query
        .get("team_id")
        .ok_or_else(|| AppError::BadRequest("Missing team_id".to_string()))?;
    let team_id = parse_mm_or_uuid(team_id_str)
        .ok_or_else(|| AppError::BadRequest("Invalid team_id".to_string()))?;
    get_categories_internal(state, user_id, team_id).await
}

pub(super) async fn get_my_categories(
    State(state): State<AppState>,
    auth: MmAuthUser,
    Query(query): Query<std::collections::HashMap<String, String>>,
) -> ApiResult<Json<mm::SidebarCategories>> {
    let team_id_str = query
        .get("team_id")
        .ok_or_else(|| AppError::BadRequest("Missing team_id".to_string()))?;
    let team_id = parse_mm_or_uuid(team_id_str)
        .ok_or_else(|| AppError::BadRequest("Invalid team_id".to_string()))?;
    get_categories_internal(state, auth.user_id, team_id).await
}

pub(super) async fn create_category(
    State(state): State<AppState>,
    auth: MmAuthUser,
    Path(params): Path<CategoriesPath>,
    Query(query): Query<std::collections::HashMap<String, String>>,
    Json(input): Json<CreateCategoryRequest>,
) -> ApiResult<Json<mm::SidebarCategory>> {
    let user_id = resolve_user_id(&params.user_id, &auth)?;
    let team_id_str = query
        .get("team_id")
        .ok_or_else(|| AppError::BadRequest("Missing team_id".to_string()))?;
    let team_id = parse_mm_or_uuid(team_id_str)
        .ok_or_else(|| AppError::BadRequest("Invalid team_id".to_string()))?;
    create_category_internal(state, user_id, team_id, input).await
}

pub(super) async fn update_categories(
    State(state): State<AppState>,
    auth: MmAuthUser,
    Path(params): Path<CategoriesPath>,
    Query(query): Query<std::collections::HashMap<String, String>>,
    Json(input): Json<UpdateCategoriesRequest>,
) -> ApiResult<Json<Vec<mm::SidebarCategory>>> {
    let user_id = resolve_user_id(&params.user_id, &auth)?;
    let team_id_str = query
        .get("team_id")
        .ok_or_else(|| AppError::BadRequest("Missing team_id".to_string()))?;
    let team_id = parse_mm_or_uuid(team_id_str)
        .ok_or_else(|| AppError::BadRequest("Invalid team_id".to_string()))?;
    update_categories_internal(state, user_id, team_id, input).await
}

pub(super) async fn update_category_order(
    State(state): State<AppState>,
    auth: MmAuthUser,
    Path(params): Path<CategoriesPath>,
    Query(query): Query<std::collections::HashMap<String, String>>,
    Json(order): Json<Vec<String>>,
) -> ApiResult<Json<Vec<String>>> {
    let user_id = resolve_user_id(&params.user_id, &auth)?;
    let team_id_str = query
        .get("team_id")
        .ok_or_else(|| AppError::BadRequest("Missing team_id".to_string()))?;
    let team_id = parse_mm_or_uuid(team_id_str)
        .ok_or_else(|| AppError::BadRequest("Invalid team_id".to_string()))?;
    update_category_order_internal(state, user_id, team_id, order).await
}

pub(crate) fn resolve_user_id(user_id_str: &str, auth: &MmAuthUser) -> ApiResult<Uuid> {
    if user_id_str == "me" {
        return Ok(auth.user_id);
    }

    let user_id = parse_mm_or_uuid(user_id_str)
        .ok_or_else(|| AppError::BadRequest("Invalid user ID".to_string()))?;

    if user_id != auth.user_id && auth.role != "system_admin" && auth.role != "org_admin" {
        return Err(AppError::Forbidden(
            "Cannot access another user's categories".to_string(),
        ));
    }

    Ok(user_id)
}

pub(crate) async fn get_categories_internal(
    state: AppState,
    user_id: Uuid,
    team_id: Uuid,
) -> ApiResult<Json<mm::SidebarCategories>> {
    ensure_team_exists(&state, team_id).await?;
    ensure_team_member(&state, user_id, team_id).await?;

    // Fetch categories
    let categories_rows: Vec<CategoryRow> = sqlx::query_as(
        "SELECT * FROM channel_categories WHERE user_id = $1 AND team_id = $2 AND delete_at = 0",
    )
    .bind(user_id)
    .bind(team_id)
    .fetch_all(&state.db)
    .await?;

    if categories_rows.is_empty() {
        return Ok(Json(
            get_default_categories(&state, user_id, team_id).await?,
        ));
    }

    let mut categories = Vec::new();
    let mut order = Vec::new();
    let mut sorted_rows = categories_rows;
    sort_category_rows(&mut sorted_rows);

    for row in sorted_rows {
        let channel_ids: Vec<Uuid> = sqlx::query_scalar(
            "SELECT channel_id FROM channel_category_channels WHERE category_id = $1 ORDER BY sort_order ASC"
        )
        .bind(row.id)
        .fetch_all(&state.db)
        .await?;

        let channel_ids = channel_ids.into_iter().map(encode_mm_id).collect();

        order.push(encode_mm_id(row.id));
        categories.push(mm::SidebarCategory {
            id: encode_mm_id(row.id),
            team_id: encode_mm_id(row.team_id),
            user_id: encode_mm_id(row.user_id),
            category_type: row.type_field,
            display_name: row.display_name,
            sorting: row.sorting,
            muted: row.muted,
            collapsed: row.collapsed,
            channel_ids,
            sort_order: row.sort_order,
            create_at: row.create_at,
            update_at: row.update_at,
            delete_at: row.delete_at,
        });
    }

    Ok(Json(mm::SidebarCategories { categories, order }))
}

#[derive(sqlx::FromRow, Clone)]
struct CategoryRow {
    id: Uuid,
    team_id: Uuid,
    user_id: Uuid,
    #[sqlx(rename = "type")]
    type_field: String,
    display_name: String,
    sorting: String,
    muted: bool,
    collapsed: bool,
    sort_order: i32,
    create_at: i64,
    update_at: i64,
    delete_at: i64,
}

fn sort_category_rows(rows: &mut [CategoryRow]) {
    let has_custom_order = rows.iter().any(|row| row.sort_order != 0);

    if has_custom_order {
        rows.sort_by(|a, b| {
            a.sort_order.cmp(&b.sort_order).then_with(|| {
                a.display_name
                    .to_ascii_lowercase()
                    .cmp(&b.display_name.to_ascii_lowercase())
            })
        });
    } else {
        rows.sort_by(|a, b| {
            a.display_name
                .to_ascii_lowercase()
                .cmp(&b.display_name.to_ascii_lowercase())
        });
    }
}

async fn ensure_team_exists(state: &AppState, team_id: Uuid) -> ApiResult<()> {
    let exists: bool = sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM teams WHERE id = $1)")
        .bind(team_id)
        .fetch_one(&state.db)
        .await?;

    if !exists {
        return Err(AppError::NotFound("Team not found".to_string()));
    }

    Ok(())
}

async fn ensure_team_member(state: &AppState, user_id: Uuid, team_id: Uuid) -> ApiResult<()> {
    let is_member: bool = sqlx::query_scalar(
        "SELECT EXISTS(SELECT 1 FROM team_members WHERE user_id = $1 AND team_id = $2)",
    )
    .bind(user_id)
    .bind(team_id)
    .fetch_one(&state.db)
    .await?;

    if !is_member {
        return Err(AppError::Forbidden(
            "User is not a member of the team".to_string(),
        ));
    }

    Ok(())
}

fn build_default_categories(
    user_id: Uuid,
    team_id: Uuid,
    channel_ids: Vec<String>,
    now: i64,
) -> mm::SidebarCategories {
    let category = mm::SidebarCategory {
        id: encode_mm_id(Uuid::new_v4()),
        team_id: encode_mm_id(team_id),
        user_id: encode_mm_id(user_id),
        category_type: "custom".to_string(),
        display_name: "Channels".to_string(),
        sorting: "alpha".to_string(),
        muted: false,
        collapsed: false,
        sort_order: 0,
        channel_ids,
        create_at: now,
        update_at: now,
        delete_at: 0,
    };

    mm::SidebarCategories {
        order: vec![category.id.clone()],
        categories: vec![category],
    }
}

async fn get_default_categories(
    state: &AppState,
    user_id: Uuid,
    team_id: Uuid,
) -> ApiResult<mm::SidebarCategories> {
    let channels: Vec<Uuid> = sqlx::query_scalar(
        r#"
        SELECT c.id FROM channels c
        JOIN channel_members cm ON c.id = cm.channel_id
        WHERE cm.user_id = $1 AND c.team_id = $2
        ORDER BY COALESCE(c.display_name, c.name) ASC
        "#,
    )
    .bind(user_id)
    .bind(team_id)
    .fetch_all(&state.db)
    .await?;

    let now = Utc::now().timestamp_millis();
    let channel_ids = channels.into_iter().map(encode_mm_id).collect();
    Ok(build_default_categories(user_id, team_id, channel_ids, now))
}

#[derive(Deserialize)]
pub(crate) struct CreateCategoryRequest {
    #[serde(default)]
    user_id: Option<String>,
    #[serde(default)]
    team_id: Option<String>,
    display_name: String,
    #[serde(rename = "type")]
    category_type: Option<String>,
    #[serde(default)]
    sorting: Option<String>,
}

pub(crate) async fn create_category_internal(
    state: AppState,
    user_id: Uuid,
    team_id: Uuid,
    input: CreateCategoryRequest,
) -> ApiResult<Json<mm::SidebarCategory>> {
    ensure_team_exists(&state, team_id).await?;
    ensure_team_member(&state, user_id, team_id).await?;

    if let Some(input_user_id) = input.user_id.as_deref() {
        let parsed = parse_mm_or_uuid(input_user_id)
            .ok_or_else(|| AppError::BadRequest("Invalid user_id".to_string()))?;
        if parsed != user_id {
            return Err(AppError::BadRequest(
                "user_id does not match path".to_string(),
            ));
        }
    }

    if let Some(input_team_id) = input.team_id.as_deref() {
        let parsed = parse_mm_or_uuid(input_team_id)
            .ok_or_else(|| AppError::BadRequest("Invalid team_id".to_string()))?;
        if parsed != team_id {
            return Err(AppError::BadRequest(
                "team_id does not match path".to_string(),
            ));
        }
    }

    let now = Utc::now().timestamp_millis();
    let id = Uuid::new_v4();
    let category_type = input.category_type.unwrap_or_else(|| "custom".to_string());
    let sorting = input.sorting.unwrap_or_else(|| "alpha".to_string());

    let next_order: i32 = sqlx::query_scalar(
        "SELECT (COALESCE(MAX(sort_order), -1) + 1)::INT FROM channel_categories WHERE user_id = $1 AND team_id = $2",
    )
    .bind(user_id)
    .bind(team_id)
    .fetch_one(&state.db)
    .await?;

    sqlx::query(
        "INSERT INTO channel_categories (id, team_id, user_id, type, display_name, sorting, sort_order, create_at, update_at) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)"
    )
    .bind(id)
    .bind(team_id)
    .bind(user_id)
    .bind(&category_type)
    .bind(&input.display_name)
    .bind(&sorting)
    .bind(next_order)
    .bind(now)
    .bind(now)
    .execute(&state.db)
    .await?;

    Ok(Json(mm::SidebarCategory {
        id: encode_mm_id(id),
        team_id: encode_mm_id(team_id),
        user_id: encode_mm_id(user_id),
        category_type,
        display_name: input.display_name,
        sorting,
        muted: false,
        collapsed: false,
        sort_order: next_order,
        channel_ids: vec![],
        create_at: now,
        update_at: now,
        delete_at: 0,
    }))
}

#[derive(Deserialize)]
pub(crate) struct UpdateCategoriesRequest {
    categories: Vec<mm::SidebarCategory>,
}

pub(crate) async fn update_categories_internal(
    state: AppState,
    user_id: Uuid,
    team_id: Uuid,
    input: UpdateCategoriesRequest,
) -> ApiResult<Json<Vec<mm::SidebarCategory>>> {
    ensure_team_exists(&state, team_id).await?;
    ensure_team_member(&state, user_id, team_id).await?;

    let now = Utc::now().timestamp_millis();
    let mut updated_categories = Vec::new();

    let mut tx = state.db.begin().await?;

    for cat in input.categories {
        let cat_uuid = parse_mm_or_uuid(&cat.id)
            .ok_or_else(|| AppError::BadRequest("Invalid category ID".to_string()))?;

        let cat_user_id = parse_mm_or_uuid(&cat.user_id)
            .ok_or_else(|| AppError::BadRequest("Invalid category user_id".to_string()))?;
        if cat_user_id != user_id {
            return Err(AppError::BadRequest(
                "category user_id does not match path".to_string(),
            ));
        }

        let cat_team_id = parse_mm_or_uuid(&cat.team_id)
            .ok_or_else(|| AppError::BadRequest("Invalid category team_id".to_string()))?;
        if cat_team_id != team_id {
            return Err(AppError::BadRequest(
                "category team_id does not match path".to_string(),
            ));
        }

        sqlx::query(
            "UPDATE channel_categories SET display_name = $1, sorting = $2, muted = $3, collapsed = $4, update_at = $5 WHERE id = $6 AND user_id = $7 AND team_id = $8"
        )
        .bind(&cat.display_name)
        .bind(&cat.sorting)
        .bind(cat.muted)
        .bind(cat.collapsed)
        .bind(now)
        .bind(cat_uuid)
        .bind(user_id)
        .bind(team_id)
        .execute(&mut *tx)
        .await?;

        // Update channels
        sqlx::query("DELETE FROM channel_category_channels WHERE category_id = $1")
            .bind(cat_uuid)
            .execute(&mut *tx)
            .await?;

        let mut parsed_channel_ids = Vec::new();
        for (i, channel_id_str) in cat.channel_ids.iter().enumerate() {
            let channel_uuid = parse_mm_or_uuid(channel_id_str)
                .ok_or_else(|| AppError::BadRequest("Invalid channel ID".to_string()))?;
            sqlx::query("INSERT INTO channel_category_channels (category_id, channel_id, sort_order) VALUES ($1, $2, $3)")
                .bind(cat_uuid)
                .bind(channel_uuid)
                .bind(i as i32)
                .execute(&mut *tx)
                .await?;
            parsed_channel_ids.push(channel_uuid);
        }

        let mut cat_out = cat;
        cat_out.id = encode_mm_id(cat_uuid);
        cat_out.user_id = encode_mm_id(user_id);
        cat_out.team_id = encode_mm_id(team_id);
        cat_out.sort_order = 0; // Assuming sort_order is not part of the update request, or defaults to 0
        cat_out.channel_ids = parsed_channel_ids.into_iter().map(encode_mm_id).collect();
        updated_categories.push(cat_out);
    }

    tx.commit().await?;

    Ok(Json(updated_categories))
}

pub(crate) async fn update_category_order_internal(
    state: AppState,
    user_id: Uuid,
    team_id: Uuid,
    order: Vec<String>,
) -> ApiResult<Json<Vec<String>>> {
    ensure_team_exists(&state, team_id).await?;
    ensure_team_member(&state, user_id, team_id).await?;

    let mut tx = state.db.begin().await?;

    for (i, cat_id_str) in order.iter().enumerate() {
        let cat_uuid = parse_mm_or_uuid(cat_id_str)
            .ok_or_else(|| AppError::BadRequest("Invalid category ID".to_string()))?;
        sqlx::query(
            "UPDATE channel_categories SET sort_order = $1 WHERE id = $2 AND user_id = $3 AND team_id = $4"
        )
        .bind(i as i32)
        .bind(cat_uuid)
        .bind(user_id)
        .bind(team_id)
        .execute(&mut *tx)
        .await?;
    }

    tx.commit().await?;

    Ok(Json(order))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn is_millis_timestamp(value: i64) -> bool {
        value >= 1_000_000_000_000 && value <= 9_999_999_999_999
    }

    fn row(display_name: &str, sort_order: i32) -> CategoryRow {
        CategoryRow {
            id: Uuid::new_v4(),
            team_id: Uuid::new_v4(),
            user_id: Uuid::new_v4(),
            type_field: "custom".to_string(),
            display_name: display_name.to_string(),
            sorting: "alpha".to_string(),
            muted: false,
            collapsed: false,
            sort_order,
            create_at: 0,
            update_at: 0,
            delete_at: 0,
        }
    }

    #[test]
    fn default_category_generation() {
        let user_id = Uuid::new_v4();
        let team_id = Uuid::new_v4();
        let channel_ids = vec!["chan-a".to_string(), "chan-b".to_string()];
        let now = 1_700_000_000_123i64;

        let result = build_default_categories(user_id, team_id, channel_ids.clone(), now);
        assert_eq!(result.categories.len(), 1);
        assert_eq!(result.order.len(), 1);

        let category = &result.categories[0];
        assert_eq!(category.display_name, "Channels");
        assert_eq!(category.channel_ids, channel_ids);
        assert_eq!(category.create_at, now);
        assert_eq!(category.update_at, now);
        assert_eq!(result.order[0], category.id);
    }

    #[test]
    fn timestamps_are_millis() {
        let user_id = Uuid::new_v4();
        let team_id = Uuid::new_v4();
        let now = 1_700_000_000_000i64;

        let result = build_default_categories(user_id, team_id, Vec::new(), now);
        let category = &result.categories[0];
        assert!(is_millis_timestamp(category.create_at));
        assert!(is_millis_timestamp(category.update_at));
    }

    #[test]
    fn ordering_logic_prefers_sort_order() {
        let mut rows = vec![row("Gamma", 2), row("Alpha", 1)];
        sort_category_rows(&mut rows);
        assert_eq!(rows[0].display_name, "Alpha");
        assert_eq!(rows[1].display_name, "Gamma");
    }

    #[test]
    fn ordering_logic_falls_back_to_display_name() {
        let mut rows = vec![row("Bravo", 0), row("alpha", 0), row("Charlie", 0)];
        sort_category_rows(&mut rows);
        assert_eq!(rows[0].display_name, "alpha");
        assert_eq!(rows[1].display_name, "Bravo");
        assert_eq!(rows[2].display_name, "Charlie");
    }
}
