use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
    response::IntoResponse,
};
use sea_orm::{
    DatabaseConnection, EntityTrait, QueryFilter, ColumnTrait, Set, ActiveModelTrait, 
    RelationTrait, QuerySelect, Condition, DbErr, QueryOrder, Order, TransactionTrait, PaginatorTrait
};
use crate::entities::feed::{self, Entity as Feed};
use crate::entities::feed_item::{self, Entity as FeedItem};
use crate::entities::attachment::{self, Entity as Attachment};
use crate::models::feed_item::{FeedItemModel, CreateFeedItem, UpdateFeedItem, CreateAttachment};
use crate::models::attachment::AttachmentModel;
use chrono::Utc;
use uuid::Uuid;
use serde_json::json;

pub async fn get_feed_items(
    State(db): State<DatabaseConnection>,
) -> Result<impl IntoResponse, StatusCode> {
    let feed_items = FeedItem::find()
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

    Ok((StatusCode::OK, Json(feed_item_models)))
}

pub async fn get_feed_item_by_id(
    Path(feed_item_id): Path<Uuid>,
    State(db): State<DatabaseConnection>,
) -> Result<impl IntoResponse, StatusCode> {
    let feed_item = FeedItem::find_by_id(feed_item_id)
        .one(&db)
        .await
        .map_err(|err| {
            tracing::error!("Database error: {:?}", err);
            StatusCode::INTERNAL_SERVER_ERROR
        })?
        .ok_or(StatusCode::NOT_FOUND)?;

    let mut feed_item_model = FeedItemModel::from(feed_item);

    // Fetch attachments for the feed item
    let attachments = Attachment::find()
        .filter(attachment::Column::FeedItemId.eq(feed_item_model.id))
        .all(&db)
        .await
        .map_err(|err| {
            tracing::error!("Database error: {:?}", err);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    if !attachments.is_empty() {
        feed_item_model.attachments = Some(
            attachments.into_iter().map(AttachmentModel::from).collect()
        );
    }

    Ok((StatusCode::OK, Json(feed_item_model)))
}

pub async fn get_feed_items_by_feed(
    Path(feed_id): Path<Uuid>,
    State(db): State<DatabaseConnection>,
) -> Result<impl IntoResponse, StatusCode> {
    // Check if feed exists
    let feed_exists = Feed::find_by_id(feed_id)
        .count(&db)
        .await
        .map_err(|err| {
            tracing::error!("Database error: {:?}", err);
            StatusCode::INTERNAL_SERVER_ERROR
        })? > 0;

    if !feed_exists {
        return Err(StatusCode::NOT_FOUND);
    }

    let feed_items = FeedItem::find()
        .filter(feed_item::Column::FeedId.eq(feed_id))
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

    Ok((StatusCode::OK, Json(feed_item_models)))
}

pub async fn create_feed_item(
    State(db): State<DatabaseConnection>,
    Json(payload): Json<CreateFeedItem>,
) -> Result<impl IntoResponse, StatusCode> {
    // Check if feed exists
    let feed_exists = Feed::find_by_id(payload.feed_id)
        .count(&db)
        .await
        .map_err(|err| {
            tracing::error!("Database error: {:?}", err);
            StatusCode::INTERNAL_SERVER_ERROR
        })? > 0;

    if !feed_exists {
        return Err(StatusCode::BAD_REQUEST);
    }

    // Start a transaction
    let txn = db.begin().await.map_err(|err| {
        tracing::error!("Transaction error: {:?}", err);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    // Generate a unique ID for the feed item
    let feed_item_id = Uuid::new_v4();
    
    // Create the URL if not provided
    let url = payload.url.unwrap_or_else(|| {
        format!("/blog/{}", feed_item_id)
    });

    let now = Utc::now();

    let new_feed_item = feed_item::ActiveModel {
        id: Set(feed_item_id),
        feed_id: Set(payload.feed_id),
        url: Set(url),
        external_url: Set(payload.external_url),
        title: Set(payload.title),
        content_html: Set(payload.content_html),
        content_text: Set(payload.content_text),
        summary: Set(payload.summary),
        image: Set(payload.image),
        banner_image: Set(payload.banner_image),
        date_published: Set(now),
        date_modified: Set(now),
        author_name: Set(payload.author_name),
        author_url: Set(payload.author_url),
        author_avatar: Set(payload.author_avatar),
        tags: Set(payload.tags),
        attachments: Set(None),
        status: Set(payload.status),
        created_at: Set(now),
        updated_at: Set(now),
    };

    let feed_item = new_feed_item.insert(&txn)
        .await
        .map_err(|err| {
            tracing::error!("Database error: {:?}", err);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    let mut feed_item_model = FeedItemModel::from(feed_item);

    // Create attachments if provided
    if let Some(attachments) = payload.attachments {
        let mut attachment_models = Vec::new();

        for attachment_data in attachments {
            let new_attachment = attachment::ActiveModel {
                id: Set(Uuid::new_v4()),
                feed_item_id: Set(Some(feed_item_id)),
                url: Set(attachment_data.url),
                mime_type: Set(attachment_data.mime_type),
                title: Set(attachment_data.title),
                size_in_bytes: Set(attachment_data.size_in_bytes),
                duration_in_seconds: Set(attachment_data.duration_in_seconds),
                created_at: Set(now),
                updated_at: Set(now),
            };

            let attachment = new_attachment.insert(&txn)
                .await
                .map_err(|err| {
                    tracing::error!("Database error: {:?}", err);
                    StatusCode::INTERNAL_SERVER_ERROR
                })?;

            attachment_models.push(AttachmentModel::from(attachment));
        }

        feed_item_model.attachments = Some(attachment_models);
    }

    // Commit the transaction
    txn.commit().await.map_err(|err| {
        tracing::error!("Transaction commit error: {:?}", err);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    Ok((StatusCode::CREATED, Json(feed_item_model)))
}

pub async fn update_feed_item(
    Path(feed_item_id): Path<Uuid>,
    State(db): State<DatabaseConnection>,
    Json(payload): Json<UpdateFeedItem>,
) -> Result<impl IntoResponse, StatusCode> {
    let feed_item = FeedItem::find_by_id(feed_item_id)
        .one(&db)
        .await
        .map_err(|err| {
            tracing::error!("Database error: {:?}", err);
            StatusCode::INTERNAL_SERVER_ERROR
        })?
        .ok_or(StatusCode::NOT_FOUND)?;

    let mut feed_item_model: feed_item::ActiveModel = feed_item.into();

    if let Some(url) = payload.url {
        feed_item_model.url = Set(url);
    }

    if let Some(external_url) = payload.external_url {
        feed_item_model.external_url = Set(Some(external_url));
    }

    if let Some(title) = payload.title {
        feed_item_model.title = Set(title);
    }

    if let Some(content_html) = payload.content_html {
        feed_item_model.content_html = Set(content_html);
    }

    if let Some(content_text) = payload.content_text {
        feed_item_model.content_text = Set(content_text);
    }

    if let Some(summary) = payload.summary {
        feed_item_model.summary = Set(Some(summary));
    }

    if let Some(image) = payload.image {
        feed_item_model.image = Set(Some(image));
    }

    if let Some(banner_image) = payload.banner_image {
        feed_item_model.banner_image = Set(Some(banner_image));
    }

    if let Some(author_name) = payload.author_name {
        feed_item_model.author_name = Set(author_name);
    }

    if let Some(author_url) = payload.author_url {
        feed_item_model.author_url = Set(Some(author_url));
    }

    if let Some(author_avatar) = payload.author_avatar {
        feed_item_model.author_avatar = Set(Some(author_avatar));
    }

    if let Some(tags) = payload.tags {
        feed_item_model.tags = Set(tags);
    }

    if let Some(status) = payload.status {
        feed_item_model.status = Set(status);
    }

    feed_item_model.date_modified = Set(Utc::now());
    feed_item_model.updated_at = Set(Utc::now());

    let updated_feed_item = feed_item_model.update(&db)
        .await
        .map_err(|err| {
            tracing::error!("Database error: {:?}", err);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    let mut feed_item_model = FeedItemModel::from(updated_feed_item);

    // Fetch attachments for the feed item
    let attachments = Attachment::find()
        .filter(attachment::Column::FeedItemId.eq(feed_item_model.id))
        .all(&db)
        .await
        .map_err(|err| {
            tracing::error!("Database error: {:?}", err);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    if !attachments.is_empty() {
        feed_item_model.attachments = Some(
            attachments.into_iter().map(AttachmentModel::from).collect()
        );
    }

    Ok((StatusCode::OK, Json(feed_item_model)))
}

pub async fn delete_feed_item(
    Path(feed_item_id): Path<Uuid>,
    State(db): State<DatabaseConnection>,
) -> Result<impl IntoResponse, StatusCode> {
    // Start a transaction
    let txn = db.begin().await.map_err(|err| {
        tracing::error!("Transaction error: {:?}", err);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    // Delete attachments for this feed item
    Attachment::delete_many()
        .filter(attachment::Column::FeedItemId.eq(feed_item_id))
        .exec(&txn)
        .await
        .map_err(|err| {
            tracing::error!("Database error: {:?}", err);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    // Delete the feed item
    let result = FeedItem::delete_by_id(feed_item_id)
        .exec(&txn)
        .await
        .map_err(|err| {
            tracing::error!("Database error: {:?}", err);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    // Commit the transaction
    txn.commit().await.map_err(|err| {
        tracing::error!("Transaction commit error: {:?}", err);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    if result.rows_affected == 0 {
        return Err(StatusCode::NOT_FOUND);
    }

    Ok(StatusCode::NO_CONTENT)
} 