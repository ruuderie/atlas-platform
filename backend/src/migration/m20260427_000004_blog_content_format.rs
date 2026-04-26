use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

/// Adds `content_format` column to `blog_posts` to support per-post rendering mode.
///
/// Supported formats:
///   'markdown'  — default, renders via pulldown_cmark (existing behaviour)
///   'latex'     — pure LaTeX, server-side conversion via latex2mathml + client KaTeX
///   'mdlatex'   — mixed Markdown with inline $...$ / $$...$$ LaTeX delimiters;
///                 delimiters are converted to KaTeX markers before Markdown rendering
///
/// Client-side KaTeX auto-render is injected conditionally when content_format
/// is 'latex' or 'mdlatex'. See apps/anchor/src/pages/blog.rs.
#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let db = manager.get_connection();

        let sql = r#"
            -- Add content_format with safe default for existing rows
            ALTER TABLE blog_posts
                ADD COLUMN IF NOT EXISTS content_format VARCHAR(20) NOT NULL DEFAULT 'markdown';

            -- Add a check constraint so only valid formats can be stored
            DO $$
            BEGIN
                IF NOT EXISTS (
                    SELECT 1 FROM pg_constraint
                    WHERE conname = 'blog_posts_content_format_check'
                ) THEN
                    ALTER TABLE blog_posts
                        ADD CONSTRAINT blog_posts_content_format_check
                        CHECK (content_format IN ('markdown', 'latex', 'mdlatex'));
                END IF;
            END $$;

            -- Index for filtering by format (useful for batch re-rendering jobs)
            CREATE INDEX IF NOT EXISTS idx_blog_posts_content_format
                ON blog_posts (content_format)
                WHERE content_format != 'markdown';
        "#;

        db.execute_unprepared(sql).await?;
        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let db = manager.get_connection();
        let sql = r#"
            DROP INDEX IF EXISTS idx_blog_posts_content_format;
            ALTER TABLE blog_posts DROP CONSTRAINT IF EXISTS blog_posts_content_format_check;
            ALTER TABLE blog_posts DROP COLUMN IF EXISTS content_format;
        "#;
        db.execute_unprepared(sql).await?;
        Ok(())
    }
}
