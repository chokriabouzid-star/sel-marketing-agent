use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Signal {
    pub id:            String,
    pub source:        SignalSource,
    pub signal_type:   SignalType,
    pub title:         String,
    pub url:           Option<String>,
    pub body:          Option<String>,
    pub score:         Option<i64>,
    pub comment_count: Option<i64>,
    pub ts:            DateTime<Utc>,
    pub processed:     bool,
}

impl Signal {
    pub fn new(
        source:      SignalSource,
        signal_type: SignalType,
        title:       impl Into<String>,
    ) -> Self {
        Self {
            id:            Uuid::new_v4().to_string(),
            source,
            signal_type,
            title:         title.into(),
            url:           None,
            body:          None,
            score:         None,
            comment_count: None,
            ts:            Utc::now(),
            processed:     false,
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum SignalSource {
    RedditLocalllama,
    RedditProgramming,
    RedditMachinelearning,
    RedditRust,
    HackerNews,
    GithubCompetitor(String),
}

impl std::fmt::Display for SignalSource {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::RedditLocalllama      => write!(f, "reddit/LocalLLaMA"),
            Self::RedditProgramming     => write!(f, "reddit/programming"),
            Self::RedditMachinelearning => write!(f, "reddit/MachineLearning"),
            Self::RedditRust            => write!(f, "reddit/rust"),
            Self::HackerNews            => write!(f, "hackernews"),
            Self::GithubCompetitor(r)   => write!(f, "github/{}", r),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum SignalType {
    Opportunity,
    CompetitorNews,
    Trending,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Draft {
    pub id:            String,
    pub platform:      Platform,
    pub content_type:  ContentType,
    pub content:       String,
    pub signal_id:     Option<String>,
    pub signal_url:    Option<String>,
    pub status:        DraftStatus,
    pub ts_created:    DateTime<Utc>,
    pub ts_updated:    Option<DateTime<Utc>>,
    pub quality_score: Option<f32>,
}

impl Draft {
    pub fn new(
        platform:     Platform,
        content_type: ContentType,
        content:      impl Into<String>,
    ) -> Self {
        Self {
            id:            format!("draft_{}", Uuid::new_v4().simple()),
            platform,
            content_type,
            content:       content.into(),
            signal_id:     None,
            signal_url:    None,
            status:        DraftStatus::PendingApproval,
            ts_created:    Utc::now(),
            ts_updated:    None,
            quality_score: None,
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum Platform {
    Reddit,
    HackerNews,
    DevTo,
    GitHub,
    Twitter,
}

impl std::fmt::Display for Platform {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Reddit     => write!(f, "Reddit"),
            Self::HackerNews => write!(f, "Hacker News"),
            Self::DevTo      => write!(f, "Dev.to"),
            Self::GitHub     => write!(f, "GitHub"),
            Self::Twitter    => write!(f, "Twitter/X"),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum ContentType {
    Comment,
    Post,
    Article,
    ReadmeSection,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum DraftStatus {
    PendingApproval,
    Approved,
    Rejected,
    Published,
    Failed,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Published {
    pub id:           String,
    pub draft_id:     String,
    pub platform:     Platform,
    pub url:          Option<String>,
    pub ts_published: DateTime<Utc>,
    pub metrics:      PostMetrics,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct PostMetrics {
    pub upvotes:      Option<i64>,
    pub comments:     Option<i64>,
    pub views:        Option<i64>,
    pub last_checked: Option<DateTime<Utc>>,
}
