use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
    response::IntoResponse,
};
use sea_orm::{
    DatabaseConnection, EntityTrait, QueryFilter, ColumnTrait, Set, ActiveModelTrait, 
    RelationTrait, QuerySelect, Condition, DbErr, QueryOrder, Order
};
use crate::entities::feed::{self, Entity as Feed};
use crate::entities::feed_item::{self, Entity as FeedItem};
use crate::entities::attachment::{self, Entity as Attachment};
use crate::models::feed::{FeedModel, CreateFeed, UpdateFeed, FeedWithItems};
use crate::models::feed_item::FeedItemModel;
use crate::models::attachment::AttachmentModel;
use chrono::Utc;
use uuid::Uuid;
use serde_json::json;

use axum::{
    Router,
    routing::{get, post, put, delete},
};
use crate::handlers::{feeds, feed_items};

pub fn public_routes(db: DatabaseConnection) -> Router<DatabaseConnection> {
    Router::new()
        // Feed routes
        .route("/feeds", get(feeds::get_feeds))
        .route("/feeds/{feed_id}", get(feeds::get_feed_by_id))
        .route("/feeds/{feed_id}/items", get(feeds::get_feed_with_items))
        .route("/feeds/directory/{tenant_id}", get(feeds::get_feeds_by_directory))
        .route("/feeds/{feed_id}/json", get(feeds::get_json_feed))
        
        // Feed item routes
        .route("/feed-items", get(feed_items::get_feed_items))
        .route("/feed-items/{feed_item_id}", get(feed_items::get_feed_item_by_id))
        .route("/feed-items/feed/{feed_id}", get(feed_items::get_feed_items_by_feed))
        .with_state(db)
}

pub fn authenticated_routes(db: DatabaseConnection) -> Router<DatabaseConnection> {
    Router::new()
        // Feed management routes
        .route("/api/feeds", post(feeds::create_feed))
        .route("/api/feeds/{feed_id}", put(feeds::update_feed))
        .route("/api/feeds/{feed_id}", delete(feeds::delete_feed))
        
        // Feed item management routes
        .route("/api/feed-items", post(feed_items::create_feed_item))
        .route("/api/feed-items/{feed_item_id}", put(feed_items::update_feed_item))
        .route("/api/feed-items/{feed_item_id}", delete(feed_items::delete_feed_item))
        .with_state(db)
}

pub async fn get_feeds(
    State(db): State<DatabaseConnection>,
) -> Result<impl IntoResponse, StatusCode> {
    let feeds = Feed::find()
        .all(&db)
        .await
        .map_err(|err| {
            tracing::error!("Database error: {:?}", err);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    let feed_models: Vec<FeedModel> = feeds
        .into_iter()
        .map(FeedModel::from)
        .collect();

    Ok((StatusCode::OK, Json(feed_models)))
}

pub async fn get_feed_by_id(
    Path(feed_id): Path<Uuid>,
    State(db): State<DatabaseConnection>,
) -> Result<impl IntoResponse, StatusCode> {
    let feed = Feed::find_by_id(feed_id)
        .one(&db)
        .await
        .map_err(|err| {
            tracing::error!("Database error: {:?}", err);
            StatusCode::INTERNAL_SERVER_ERROR
        })?
        .ok_or(StatusCode::NOT_FOUND)?;

    Ok((StatusCode::OK, Json(FeedModel::from(feed))))
}

pub async fn get_feed_with_items(
    Path(feed_id): Path<Uuid>,
    State(db): State<DatabaseConnection>,
) -> Result<impl IntoResponse, StatusCode> {
    let feed = Feed::find_by_id(feed_id)
        .one(&db)
        .await
        .map_err(|err| {
            tracing::error!("Database error: {:?}", err);
            StatusCode::INTERNAL_SERVER_ERROR
        })?
        .ok_or(StatusCode::NOT_FOUND)?;

    let feed_items = FeedItem::find()
        .filter(feed_item::Column::FeedId.eq(feed_id))
        .filter(feed_item::Column::Status.eq("published"))
        .order_by(feed_item::Column::DatePublished, Order::Desc)
        .all(&db)
        .await
        .map_err(|err| {
            tracing::error!("Database error: {:?}", err);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    let mut feed_item_models: Vec<FeedItemModel> = feed_items
        .into_iter()
        .map(FeedItemModel::from)
        .collect();

    // Fetch attachments for each feed item
    for feed_item in &mut feed_item_models {
        let attachments = Attachment::find()
            .filter(attachment::Column::FeedItemId.eq(feed_item.id))
            .all(&db)
            .await
            .map_err(|err| {
                tracing::error!("Database error: {:?}", err);
                StatusCode::INTERNAL_SERVER_ERROR
            })?;

        if !attachments.is_empty() {
            feed_item.attachments = Some(
                attachments.into_iter().map(AttachmentModel::from).collect()
            );
        }
    }

    let feed_with_items = FeedWithItems {
        id: feed.id,
        tenant_id: feed.tenant_id,
        title: feed.title,
        description: feed.description,
        feed_url: feed.feed_url,
        home_page_url: feed.home_page_url,
        icon: feed.icon,
        favicon: feed.favicon,
        author: feed.author,
        created_at: feed.created_at,
        updated_at: feed.updated_at,
        items: feed_item_models,
    };

    Ok((StatusCode::OK, Json(feed_with_items)))
}

pub async fn get_feeds_by_directory(
    Path(tenant_id): Path<Uuid>,
    State(db): State<DatabaseConnection>,
) -> Result<impl IntoResponse, StatusCode> {
    let feeds = Feed::find()
        .filter(feed::Column::TenantId.eq(tenant_id))
        .all(&db)
        .await
        .map_err(|err| {
            tracing::error!("Database error: {:?}", err);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    let feed_models: Vec<FeedModel> = feeds
        .into_iter()
        .map(FeedModel::from)
        .collect();

    Ok((StatusCode::OK, Json(feed_models)))
}

pub async fn create_feed(
    State(db): State<DatabaseConnection>,
    Json(payload): Json<CreateFeed>,
) -> Result<impl IntoResponse, StatusCode> {
    // Generate a unique ID for the feed
    let feed_id = Uuid::new_v4();
    
    // Create the feed URL if not provided
    let feed_url = payload.feed_url.unwrap_or_else(|| {
        format!("/api/feeds/{}/json", feed_id)
    });
    
    // Create the home page URL if not provided
    let home_page_url = payload.home_page_url.unwrap_or_else(|| {
        format!("/blog")
    });

    let new_feed = feed::ActiveModel {
        id: Set(feed_id),
        tenant_id: Set(payload.tenant_id),
        title: Set(payload.title),
        description: Set(payload.description),
        feed_url: Set(feed_url),
        home_page_url: Set(home_page_url),
        icon: Set(payload.icon),
        favicon: Set(payload.favicon),
        author: Set(payload.author),
        created_at: Set(Utc::now()),
        updated_at: Set(Utc::now()),
    };

    let feed = new_feed.insert(&db)
        .await
        .map_err(|err| {
            println!("DB ERROR: {:?}", err);
            tracing::error!("Database error: {:?}", err);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    Ok((StatusCode::CREATED, Json(FeedModel::from(feed))))
}

pub async fn update_feed(
    Path(feed_id): Path<Uuid>,
    State(db): State<DatabaseConnection>,
    Json(payload): Json<UpdateFeed>,
) -> Result<impl IntoResponse, StatusCode> {
    let feed = Feed::find_by_id(feed_id)
        .one(&db)
        .await
        .map_err(|err| {
            tracing::error!("Database error: {:?}", err);
            StatusCode::INTERNAL_SERVER_ERROR
        })?
        .ok_or(StatusCode::NOT_FOUND)?;

    let mut feed_model: feed::ActiveModel = feed.into();

    if let Some(title) = payload.title {
        feed_model.title = Set(title);
    }

    if let Some(description) = payload.description {
        feed_model.description = Set(description);
    }

    if let Some(feed_url) = payload.feed_url {
        feed_model.feed_url = Set(feed_url);
    }

    if let Some(home_page_url) = payload.home_page_url {
        feed_model.home_page_url = Set(home_page_url);
    }

    if let Some(icon) = payload.icon {
        feed_model.icon = Set(Some(icon));
    }

    if let Some(favicon) = payload.favicon {
        feed_model.favicon = Set(Some(favicon));
    }

    if let Some(author) = payload.author {
        feed_model.author = Set(Some(author));
    }

    feed_model.updated_at = Set(Utc::now());

    let updated_feed = feed_model.update(&db)
        .await
        .map_err(|err| {
            tracing::error!("Database error: {:?}", err);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    Ok((StatusCode::OK, Json(FeedModel::from(updated_feed))))
}

pub async fn delete_feed(
    Path(feed_id): Path<Uuid>,
    State(db): State<DatabaseConnection>,
) -> Result<impl IntoResponse, StatusCode> {
    // First, delete all feed items associated with this feed
    let feed_items = FeedItem::find()
        .filter(feed_item::Column::FeedId.eq(feed_id))
        .all(&db)
        .await
        .map_err(|err| {
            tracing::error!("Database error: {:?}", err);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    for feed_item in feed_items {
        // Delete attachments for this feed item
        Attachment::delete_many()
            .filter(attachment::Column::FeedItemId.eq(feed_item.id))
            .exec(&db)
            .await
            .map_err(|err| {
                tracing::error!("Database error: {:?}", err);
                StatusCode::INTERNAL_SERVER_ERROR
            })?;

        // Delete the feed item
        FeedItem::delete_by_id(feed_item.id)
            .exec(&db)
            .await
            .map_err(|err| {
                tracing::error!("Database error: {:?}", err);
                StatusCode::INTERNAL_SERVER_ERROR
            })?;
    }

    // Finally, delete the feed
    Feed::delete_by_id(feed_id)
        .exec(&db)
        .await
        .map_err(|err| {
            tracing::error!("Database error: {:?}", err);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    Ok(StatusCode::NO_CONTENT)
}

// JSON Feed format export
pub async fn get_json_feed(
    Path(feed_id): Path<Uuid>,
    State(db): State<DatabaseConnection>,
) -> Result<impl IntoResponse, StatusCode> {
    let feed = Feed::find_by_id(feed_id)
        .one(&db)
        .await
        .map_err(|err| {
            tracing::error!("Database error: {:?}", err);
            StatusCode::INTERNAL_SERVER_ERROR
        })?
        .ok_or(StatusCode::NOT_FOUND)?;

    let feed_items = FeedItem::find()
        .filter(feed_item::Column::FeedId.eq(feed_id))
        .filter(feed_item::Column::Status.eq("published"))
        .order_by(feed_item::Column::DatePublished, Order::Desc)
        .all(&db)
        .await
        .map_err(|err| {
            tracing::error!("Database error: {:?}", err);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    let mut json_feed_items = Vec::new();

    for feed_item in feed_items {
        let attachments = Attachment::find()
            .filter(attachment::Column::FeedItemId.eq(feed_item.id))
            .all(&db)
            .await
            .map_err(|err| {
                tracing::error!("Database error: {:?}", err);
                StatusCode::INTERNAL_SERVER_ERROR
            })?;

        let json_attachments = attachments.iter().map(|attachment| {
            json!({
                "url": attachment.url,
                "mime_type": attachment.mime_type,
                "title": attachment.title,
                "size_in_bytes": attachment.size_in_bytes,
                "duration_in_seconds": attachment.duration_in_seconds
            })
        }).collect::<Vec<_>>();

        let mut item = json!({
            "id": feed_item.id.to_string(),
            "url": feed_item.url,
            "title": feed_item.title,
            "content_html": feed_item.content_html,
            "content_text": feed_item.content_text,
            "date_published": feed_item.date_published.to_rfc3339(),
            "date_modified": feed_item.date_modified.to_rfc3339(),
            "author": {
                "name": feed_item.author_name
            },
            "tags": feed_item.tags
        });

        if let Some(external_url) = feed_item.external_url {
            item["external_url"] = json!(external_url);
        }

        if let Some(summary) = feed_item.summary {
            item["summary"] = json!(summary);
        }

        if let Some(image) = feed_item.image {
            item["image"] = json!(image);
        }

        if let Some(banner_image) = feed_item.banner_image {
            item["banner_image"] = json!(banner_image);
        }

        if let Some(author_url) = feed_item.author_url {
            item["author"]["url"] = json!(author_url);
        }

        if let Some(author_avatar) = feed_item.author_avatar {
            item["author"]["avatar"] = json!(author_avatar);
        }

        if !json_attachments.is_empty() {
            item["attachments"] = json!(json_attachments);
        }

        json_feed_items.push(item);
    }

    let mut json_feed = json!({
        "version": "https://jsonfeed.org/version/1.1",
        "title": feed.title,
        "description": feed.description,
        "home_page_url": feed.home_page_url,
        "feed_url": feed.feed_url,
        "items": json_feed_items
    });

    if let Some(icon) = feed.icon {
        json_feed["icon"] = json!(icon);
    }

    if let Some(favicon) = feed.favicon {
        json_feed["favicon"] = json!(favicon);
    }

    if let Some(author) = feed.author {
        json_feed["author"] = json!({
            "name": author
        });
    }

    Ok((StatusCode::OK, Json(json_feed)))
} 