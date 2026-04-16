# Anchor CMS Blocks Schema Configuration

The Anchor CMS now supports a fully generic, declarative layout engine using `dynamic_blocks_json` from the `app_pages` table.

Below are the acceptable schema formats for each available Dynamic Block.

## 1. TimelineBlock
Renders chronological events. Supports fetching from `tenant_entries` or statically defined inline.

```json
{
  "Timeline": {
    "source": "tenant_entries",
    "config": {
      "filter_category": "work",
      "show_date_range": true,
      "show_bullets": true,
      "layout": "detailed", 
      "section_title": "Work Experience"
    },
    "items": []
  }
}
```
*Layout options*: `detailed`, `compact`, `cards`

## 2. BadgeListBlock
Renders arrays of compact UI elements like logos, tags, or credential badges.

```json
{
  "BadgeList": {
    "source": "tenant_entries",
    "config": {
      "filter_category": "certification",
      "columns": 3,
      "display": "list", 
      "section_title": "Certifications"
    },
    "items": []
  }
}
```
*Display options*: `badge`, `list`, `logo-grid`

## 3. ProfileHeaderBlock
Specialized hero section for named-entity profiles containing an avatar and contact links.

```json
{
  "ProfileHeader": {
    "full_name": "John Doe",
    "title": "Senior Solutions Architect",
    "objective": "Passionate about Rust and distributed systems.",
    "avatar_url": "https://cdn.example.com/avatar.jpg",
    "contact": {
      "email": "john@example.com",
      "location": "New York, USA",
      "github_url": "https://github.com/...",
      "linkedin_url": "..."
    },
    "badges": ["Rustvangelist", "AWS Certified"]
  }
}
```

## 4. ContentFeedBlock
Paginated list or grid of blog posts, case studies, or generic content feed items.

```json
{
  "ContentFeed": {
    "source": "tenant_entries",
    "config": {
      "filter_category": "project",
      "layout": "cards",
      "show_tags": true,
      "show_date": true,
      "section_title": "Featured Projects"
    },
    "items": []
  }
}
```
*Layout options*: `cards`, `list`

## 5. StatsBlock
A performance dashboard, key metrics tracker, or continuous ticker.

```json
{
  "Stats": {
    "source": "static",
    "config": {
      "columns": 3,
      "display": "metric",
      "section_title": "By The Numbers"
    },
    "items": [
      {
        "label": "Uptime",
        "value": "99.9",
        "unit": "%",
        "icon": "speed",
        "trend": "up"
      }
    ]
  }
}
```
*Display options*: `metric`, `ticker`

## 6. AccordionBlock
Interactive expanding rows (FAQs) leveraging native HTML `<details>`.

```json
{
  "Accordion": {
    "config": {
      "mode": "single",
      "section_title": "Frequently Asked Questions"
    },
    "items": [
      {
        "title": "What is your hourly rate?",
        "body": "Rates begin at $150/hr depending on scope.",
        "badge": "Pricing"
      }
    ]
  }
}
```

---
**Note:** Due to Rust's internal Tagged Enum deserialization bug (`untagged` greedy matching), all blocks MUST be structurally defined precisely with exactly a single key of their Component Name (e.g. `{"Timeline": {...}}`).
